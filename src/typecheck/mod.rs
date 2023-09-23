use crate::ast::{Ast, Type, Value, Comparison};
use std::collections::HashMap;

pub struct TypeChecker {
    current_fn: String,
    // name, (return type, parameter types)
    functions: HashMap<String, (Type, Vec<(String, Type)>)>,
    variables: HashMap<String, Type>,
}


impl TypeChecker {
    pub fn new() -> TypeChecker {
        return TypeChecker {
            current_fn: String::new(),
            functions: HashMap::new(),
            variables: HashMap::new(),
        };
    }

    fn value_type(&self, value: &Value, loc: (usize, usize)) -> Result<Type, Box<dyn std::error::Error>> {
        return match value {
            Value::BinaryExpr {loc, l_expr, r_expr, op} => {
                if self.value_type(&l_expr, loc.clone())? != self.value_type(&r_expr, loc.clone())?
                || self.value_type(&r_expr, loc.clone())? != Type::Int
                || self.value_type(&l_expr, loc.clone())? != Type::Int
                {
                    return Err(format!("{}:{}: binary expressions can only be applied to integers `{:?}` `{:?}` `{:?}`", loc.0, loc.1, *l_expr, op, *r_expr).into());
                }
                Ok(Type::Int)
            },
            Value::FunctionCall {loc, name, params} => {
                let function = match self.functions.get(name) {
                    Some(func) => func,
                    None => {
                        return Err(format!("{}:{}: unknown function `{:?}`", loc.0, loc.1, name).into());
                    },
                };
                if params.len() != function.1.len() {
                    return Err(format!("{}:{}: expected `{}` parameter(s) but got `{}`", loc.0, loc.1, function.1.len(), params.len()).into());
                }
                for (index, parameter) in params.iter().enumerate() {
                    let val_type = self.value_type(parameter, loc.clone())?;
                    if val_type != function.1[index].1 {
                        return Err(format!("{}:{}: expected `{:?}` but got `{:?}`", loc.0, loc.1, function.1[index].1, val_type).into());
                    }
                }
                Ok(function.0.clone())
            },
            Value::Int(_) => Ok(Type::Int),
            Value::Ident(ident) => {
                if let Some(value_t) = self.variables.get(ident) {
                    Ok(value_t.clone())
                } else {
                    Err(format!("{}:{}: unknown identifier `{}`", loc.0, loc.1, ident).into())
                }
            },
            Value::Deref(value, deref_type) => {
                let val_type = self.value_type(value, loc)?;
                if val_type != Type::Ptr {
                    return Err(format!("{}:{}: cant dereference non pointer type `{:?}`", loc.0, loc.1, val_type).into());
                }
                Ok(deref_type.clone())
            },
            Value::Cast(_, cast_type) => Ok(cast_type.clone()),
            Value::Ref(_) => Ok(Type::Ptr),
            Value::Null => Ok(Type::Void),
        };
    }

    fn comparison_check(&self, comparison: &Comparison, loc: (usize, usize)) -> Result<(), Box<dyn std::error::Error>> {
        let l_type = self.value_type(&comparison.l_expr, loc.clone())?;
        let r_type = self.value_type(&comparison.r_expr, loc.clone())?;
        if l_type != r_type {
            return Err(format!("{}:{}: expected `{:?}` but got `{:?}`", loc.0, loc.1, l_type, r_type).into());
        }
        return Ok(());
    }

    pub fn check(&mut self, ast: &Vec<Ast>, nested: bool) -> Result<(), Box<dyn std::error::Error>> {
        let mut local_vars: Vec<String> = Vec::new();
        let mut index = 0;

        while index < ast.len() {
            match &ast[index] {
                Ast::Function {loc, name, param_t, return_t, body} => {
                    if self.functions.get(name).is_some() {
                        return Err(format!("{}:{}: function `{}` already exists", loc.0, loc.1, name).into());
                    } else if param_t.len() > 6 {
                        return Err(format!("{}:{}: functions can only accept up to 6 parameters", loc.0, loc.1).into());
                    } else if nested {
                        return Err(format!("{}:{}: functions need to be global", loc.0, loc.1).into());
                    }
                    for (var_name, var_type) in param_t {
                        self.variables.insert(var_name.clone(), var_type.clone());
                        local_vars.push(var_name.clone())
                    }
                    self.functions.insert(name.clone(), (return_t.clone(), param_t.clone()));
                    self.current_fn = name.clone();
                    self.check(body, true)?;
                },
                Ast::Return {loc, value} => {
                    let return_t = self.functions.get(&self.current_fn).ok_or(format!("{}:{}: internal compiler error, current_fn not defined correctly", loc.0, loc.1))?;
                    if self.value_type(value, loc.clone())? != return_t.0 {
                        return Err(format!("{}:{}: expected `{:?}` but got `{:?}`", loc.0, loc.1, return_t.0, self.value_type(value, loc.clone())?).into());
                    }
                },
                Ast::Variable {loc, name, var_t, value} => {
                    if var_t.clone() != self.value_type(value, loc.clone())? {
                        return Err(format!("{}:{}: expected `{:?}` but got `{:?}`", loc.0, loc.1, var_t, self.value_type(value, loc.clone())?).into());
                    } else if self.variables.get(name).is_some() {
                        return Err(format!("{}:{}: variable `{}` already exists", loc.0, loc.1, name).into());
                    }
                    self.variables.insert(name.clone(), var_t.clone());
                    local_vars.push(name.clone());
                },
                Ast::MutateVar {loc, name, value} => {
                    let var = self.variables.get(name);
                    let val_type = self.value_type(value, loc.clone())?;
                    if var.is_none() {
                        return Err(format!("{}:{}: cant mutate non existing variable `{}`", loc.0, loc.1, name).into());
                    } else if *var.unwrap() != val_type {
                        return Err(format!("{}:{}: expected `{:?}` but got `{:?}`", loc.0, loc.1, var.unwrap(), val_type).into());
                    }
                },
                Ast::MutatePtr {loc, ptr_type, ptr, value} => {
                    if self.value_type(ptr, loc.clone())? != Type::Ptr {
                        return Err(format!("{}:{}: expected `Ptr` but got `{:?}`", loc.0, loc.1, self.value_type(ptr, loc.clone())?).into());
                    } else if ptr_type != &self.value_type(value, loc.clone())? {
                        return Err(format!("{}:{}: expected `{:?}` but got `{:?}`", loc.0, loc.1, ptr_type, self.value_type(value, loc.clone())?).into());
                    }
                },
                Ast::If {loc, comparison, body, else_body} => {
                    self.comparison_check(&comparison, loc.clone())?;
                    self.check(body, true)?;
                    self.check(else_body, true)?;
                },
                Ast::While {loc, comparison, body} => {
                    self.comparison_check(&comparison, loc.clone())?;
                    self.check(body, true)?;
                },
            }
            index += 1;
        }
        for var in local_vars {
            // non fatal if fail
            self.variables.remove(&var);
        }
        return Ok(());
    }
}

