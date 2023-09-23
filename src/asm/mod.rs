use crate::ast::*;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;
use std::process::Command;
use std::collections::HashMap;

const REGISTERS: [&str; 6] = [
    "rdi",
    "rsi",
    "rdx",
    "rcx",
    "r8",
    "r9",
];

pub struct CodeGen {
    buffer: BufWriter<File>,
    variables: HashMap<String, (usize, usize)>,
    block_count: usize,
    stack_offset: usize,
    filename: String,
    current_fn: String,
}

fn get_filename(file: &str) -> Result<&str, Box<dyn std::error::Error>> {
    return Ok(file.split(".").nth(0).ok_or::<Box<dyn std::error::Error>>("failed to parse filename".into())?);
}

impl CodeGen {
    pub fn new(filename: &str) -> Result<CodeGen, Box<dyn std::error::Error>> {
        let output_filename = format!("{}.asm", get_filename(filename)?);
        return Ok(CodeGen {
            buffer: BufWriter::new(File::create(&output_filename)?),
            variables: HashMap::new(),
            block_count: 1,
            stack_offset: 0,
            filename: output_filename,
            current_fn: String::new(),
        });
    }

    fn entry(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.buffer.write(b"format ELF64 executable 3\nsegment readable executable\n")?;
        return Ok(());
    }

    fn exit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.buffer.write(b"entry start\n")?;
        self.buffer.write(b"start:\n")?;
        self.buffer.write(b"    call main\n")?;
        self.buffer.write(b"    mov rdi, rax\n")?;
        self.buffer.write(b"    mov rax, 60\n")?;
        self.buffer.write(b"    syscall\n")?;
        self.buffer.write(b"segment readable writeable\n")?;
        return Ok(());
    }

    fn value(&mut self, value: &Value) -> Result<(String, String), Box<dyn std::error::Error>> {
        return match value {
            Value::FunctionCall {name, params, ..} => {
                self.buffer.write(b"    ;; -- FUNCTION CALL --\n")?;
                for (index, parameter) in params.iter().enumerate() {
                    let val = self.value(parameter)?;
                    if val.0 != REGISTERS[index] {
                        write!(self.buffer, "    mov {}, {}\n", REGISTERS[index], val.0)?;
                    }
                }
                write!(self.buffer, "    call {}\n", name)?;
                return Ok((format!("rax"), "reg".to_string()));
            },
            Value::BinaryExpr {l_expr, r_expr, op, ..} => {
                let old_stack_offset = self.stack_offset;
                let l_val = self.value(&l_expr)?;
                let r_val = self.value(&r_expr)?;
                self.buffer.write(b"    ;; -- BINARY EXPRESSION --\n")?;
                let l_offset = self.val_is_on_stack(l_val)?;
                let r_offset = self.val_is_on_stack(r_val)?;
                write!(self.buffer, "    mov rax, [rbp-{}]\n", l_offset)?;
                match op {
                    Operator::Plus => {
                        write!(self.buffer, "    add rax, [rbp-{}]\n", r_offset)?;
                    },
                    Operator::Minus => {
                        write!(self.buffer, "    sub rax, [rbp-{}]\n", r_offset)?;
                    },
                    Operator::Multiplication => {
                        write!(self.buffer, "    mov rax, [rbp-{}]\n", r_offset)?;
                        write!(self.buffer, "    mul rbx\n")?;
                    },
                    Operator::Divide => {
                        write!(self.buffer, "    mov rax, [rbp-{}]\n", r_offset)?;
                        write!(self.buffer, "    div rbx\n")?;
                    },
                }
                self.stack_offset = old_stack_offset;
                return Ok((format!("rax"), "reg".to_string()));
            },
            Value::Ref(value) => {
                self.buffer.write(b"    ;; -- REFERENCE --\n")?;
                let value = self.value(&value)?;
                write!(self.buffer, "    lea rax, {}\n", value.0)?;
                return Ok(("rax".to_string(), "reg".to_string()));
            },
            Value::Deref(value, _) => {
                self.buffer.write(b"    ;; -- DEREFERENCE --\n")?;
                let value = self.value(&value)?;
                if &value.0 != "rax" {
                    write!(self.buffer, "    mov rax, {}\n", value.0)?;
                }
                write!(self.buffer, "    mov rbx, [rax]\n")?;
                return Ok(("rbx".to_string(), "reg".to_string()));
            },
            Value::Cast(value, _) => {
                let value = self.value(&value)?;
                Ok(value)
            },
            Value::Int(integer) => Ok((format!("{}", integer), "integer".to_string())),
            Value::Ident(ident) => {
                let var = self.variables.get(ident).unwrap();
                Ok((format!("[rbp-{}]", var.0), format!("{}", var.0)))
            },
            Value::Null => Ok(("0x0".to_string(), "NULL".to_string())),
        };
    }

    fn comparison(&mut self, comp: &Comparison) -> Result<String, Box<dyn std::error::Error>> {
        let old_stack_offset = self.stack_offset;
        let l_val = self.value(&comp.l_expr)?;
        let r_val = self.value(&comp.r_expr)?;
        self.buffer.write(b"    ;; -- COMPARISON --\n")?;
        // make sure value is on the stack
        let l_offset = self.val_is_on_stack(l_val)?;
        let r_offset = self.val_is_on_stack(r_val)?;
        write!(self.buffer, "    mov rax, [rbp-{}]\n", l_offset)?;
        write!(self.buffer, "    cmp rax, [rbp-{}]\n", r_offset)?;
        self.stack_offset = old_stack_offset;
        return match comp.op {
            ComparisonOp::Equal => Ok("jne".to_string()),
            ComparisonOp::NotEqual => Ok("je".to_string()),
            ComparisonOp::Bigger => Ok("jle".to_string()),
            ComparisonOp::Smaller => Ok("jg".to_string()),
        }
    }

    fn val_is_on_stack(&mut self, value: (String, String)) -> Result<usize, Box<dyn std::error::Error>> {
        let offset = if let Ok(integer) = value.1.parse::<usize>() {
            integer
        } else {
            self.stack_offset += 8;
            write!(self.buffer, "    mov qword [rbp-{}], {}\n", self.stack_offset, value.0)?;
            self.stack_offset
        };
        return Ok(offset);
    }

    fn val_is_in_reg(&mut self, value: (String, String)) -> Result<String, Box<dyn std::error::Error>> {
        let reg = if let Ok(addr) = value.1.parse::<usize>() {
            write!(self.buffer, "    rax, [rbp-{}]\n", addr)?;
            String::from("rax")
        } else {
            value.0
        };
        return Ok(reg);
    }

    pub fn generate(&mut self, ast: &Vec<Ast>, entry: bool) -> Result<(), Box<dyn std::error::Error>> {
        let mut local_vars: Vec<String> = Vec::new();

        if entry {
            self.entry()?;
        }

        for instruction in ast {
            match instruction {
                Ast::Function {name, param_t, body, ..} => {
                    let old_fn = self.current_fn.clone();
                    self.current_fn = name.clone();
                    self.buffer.write(b"    ;; -- FUNCTION --\n")?;
                    write!(self.buffer, "{}:\n", name)?;
                    self.buffer.write(b"    push rbp\n")?;
                    self.buffer.write(b"    mov rbp, rsp\n")?;
                    for (index, parameter) in param_t.iter().enumerate() {
                        self.stack_offset += parameter.1.size();
                        write!(self.buffer, "    mov [rbp-{}], {}\n", self.stack_offset, REGISTERS[index])?;
                        self.variables.insert(parameter.0.clone(), (self.stack_offset, parameter.1.size()));
                        local_vars.push(parameter.0.clone());
                    }
                    self.generate(body, false)?;
                    write!(self.buffer, "{}_ret:\n", name)?;
                    self.buffer.write(b"    pop rbp\n")?;
                    self.buffer.write(b"    ret\n")?;
                    self.current_fn = old_fn;
                },
                Ast::Return {value, ..} => {
                    self.buffer.write(b"    ;; -- RETURN --\n")?;
                    let value = self.value(value)?;
                    write!(self.buffer, "    mov rax, {}\n", value.0)?;
                    write!(self.buffer, "    jmp {}_ret\n", self.current_fn)?;
                },
                Ast::Variable {name, var_t, value, ..} => {
                    self.buffer.write(b"    ;; -- VARIABLE --\n")?;
                    self.stack_offset += var_t.size();
                    let value = self.value(value)?;
                    let val_reg = self.val_is_in_reg(value)?;
                    write!(self.buffer, "    mov qword [rbp-{}], {}\n", self.stack_offset, val_reg)?;
                    self.variables.insert(name.clone(), (self.stack_offset, var_t.size()));
                    local_vars.push(name.clone());
                },
                Ast::MutateVar {name, value, ..} => {
                    self.buffer.write(b"    ;; -- MUTATE VARIABLE --\n")?;
                    let value = self.value(value)?;
                    let val_reg = self.val_is_in_reg(value)?;
                    write!(self.buffer, "    mov qword [rbp-{}], {}\n", self.variables.get(name).unwrap().0, val_reg)?;
                },
                Ast::MutatePtr {ptr, value, ..} => {
                    self.buffer.write(b"    ;; -- MUTATE POINTER --\n")?;
                    let value = self.value(value)?;
                    let ptr_val = self.value(ptr)?;

                    let val_loc = self.val_is_on_stack(value)?;
                    let ptr_val_loc = self.val_is_on_stack(ptr_val)?;

                    write!(self.buffer, "    mov rax, [rbp-{}]\n", ptr_val_loc)?;
                    write!(self.buffer, "    mov rbx, [rbp-{}]\n", val_loc)?;
                    write!(self.buffer, "    mov [rax], rbx\n")?;
                },
                Ast::If {comparison, body, else_body, ..} => {
                    self.block_count += 1;
                    self.buffer.write(b"    ;; -- IF --\n")?;
                    let jump = self.comparison(comparison)?;
                    write!(self.buffer, "    {} BB_{}\n", jump, self.block_count)?;
                    self.generate(body, false)?;
                    write!(self.buffer, "    jmp BB_{}\n", self.block_count + 1)?;
                    write!(self.buffer, "BB_{}:\n", self.block_count)?;
                    self.generate(else_body, false)?;
                    self.block_count += 1;
                    write!(self.buffer, "BB_{}:\n", self.block_count)?;
                },
                Ast::While {comparison, body, ..} => {
                    self.buffer.write(b"    ;; -- WHILE --\n")?;
                    write!(self.buffer, "BB_{}:\n", self.block_count + 1)?;
                    let jump = self.comparison(comparison)?;
                    write!(self.buffer, "    {} BB_{}\n", jump, self.block_count + 2)?;
                    self.generate(body, false)?;
                    write!(self.buffer, "    jmp BB_{}\n", self.block_count + 1)?;
                    write!(self.buffer, "BB_{}:\n", self.block_count + 2)?;
                    self.block_count += 2;
                },
            }
        }

        // drop variables created in the current scope
        for var in local_vars {
            let size = self.variables.remove(&var).unwrap().1;
            self.stack_offset -= size;
        }

        if entry {
            self.exit()?;
        }
        return Ok(());
    }

    pub fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.buffer.flush()?;
        return Ok(());
    }

    pub fn assemble(&self) -> Result<String, Box<dyn std::error::Error>> {
        let filename = get_filename(&self.filename)?;
        let output = Command::new("fasm")
            .args([&self.filename, filename])
            .output()?;
        let _ = Command::new("chmod")
            .arg("+x")
            .arg(filename)
            .output()?;
        return Ok(String::from_utf8(output.stdout)?);
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let filename = get_filename(&self.filename)?;
        let _ = Command::new(&format!("./{}", filename)).spawn()?;
        return Ok(());
    }
}

