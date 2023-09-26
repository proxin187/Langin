use lib_lexin::Token;
use crate::{log_color, generate_ast};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Ptr,
    Void,
}

impl Type {
    pub fn size(&self) -> usize {
        return match self {
            Type::Int | Type::Ptr => 8,
            Type::Void => 0,
        };
    }
}

#[derive(Debug)]
pub struct Comparison {
    pub l_expr: Box<Value>,
    pub r_expr: Box<Value>,
    pub op: ComparisonOp,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    Bigger,
    Smaller,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Multiplication,
    Divide,
}

#[derive(Debug)]
pub enum Value {
    BinaryExpr {
        loc: (usize, usize),
        l_expr: Box<Value>,
        r_expr: Box<Value>,
        op: Operator,
    },
    FunctionCall {
        loc: (usize, usize),
        name: String,
        params: Vec<Value>,
    },
    Cast(Box<Value>, Type),
    Deref(Box<Value>, Type),
    Ref(Box<Value>),
    Int(usize),
    Str(String),
    Ident(String),
    Null,
}

#[derive(Debug)]
pub enum Ast {
    Function {
        loc: (usize, usize),
        name: String,
        param_t: Vec<(String, Type)>,
        return_t: Type,
        body: Vec<Ast>,
    },

    Return {
        loc: (usize, usize),
        value: Value,
    },

    Variable {
        loc: (usize, usize),
        name: String,
        var_t: Type,
        value: Value,
    },

    MutateVar {
        loc: (usize, usize),
        name: String,
        value: Value,
    },

    MutatePtr {
        loc: (usize, usize),
        ptr_type: Type,
        ptr: Value,
        value: Value,
    },

    If {
        loc: (usize, usize),
        comparison: Comparison,
        body: Vec<Ast>,
        else_body: Vec<Ast>,
    },

    While {
        loc: (usize, usize),
        comparison: Comparison,
        body: Vec<Ast>,
    },

    InlineAsm {
        loc: (usize, usize),
        asm: String,
    },
}


impl Ast {
    fn bound_check(tokens: &Vec<Token>, index: &mut usize, expected: &str) -> Result<(), Box<dyn std::error::Error>> {
        let loc = if tokens.len() != 0 {
            tokens[tokens.len() - 1].loc()
        } else {
            (0, 0)
        };
        *index += 1;
        if *index >= tokens.len() {
            return Err(format!("{} expected `{}`", log_color(loc), expected).into());
        }
        return Ok(());
    }

    fn is_operator(token: &Token, loc: (usize, usize)) -> Result<Operator, Box<dyn std::error::Error>> {
        return if token.is_symbol("Plus").is_ok() {
            Ok(Operator::Plus)
        } else if token.is_symbol("Minus").is_ok() {
            Ok(Operator::Minus)
        } else if token.is_symbol("Asterisk").is_ok() {
            Ok(Operator::Multiplication)
        } else if token.is_symbol("Slash").is_ok() {
            Ok(Operator::Divide)
        } else {
            Err(format!("{} expected `operator`", log_color(loc)).into())
        }
    }

    fn single_expr(tokens: &Vec<Token>, loc: (usize, usize)) -> Result<Value, Box<dyn std::error::Error>> {
        let mut index = 0;
        if index >= tokens.len() {
            return Err(format!("{} empty expression", log_color(loc)).into());
        }

        if let Ok(integer) = tokens[index].is_integer() {
            return Ok(Value::Int(integer));
        } else if let Ok(ident) = tokens[index].is_ident() {
            return Ok(Value::Ident(ident));
        } else if let Ok(string) = tokens[index].is_section("string") {
            return Ok(Value::Str(string));
        } else if let Ok(deref_type) = Self::is_type(tokens[index].clone()) {
            // DEREFERENCE
            let loc = tokens[index].loc();

            Self::bound_check(tokens, &mut index, "OpenBracket")?;

            let seperator_name = if tokens[index].is_symbol("OpenBracket").is_ok() {
                "CloseBracket"
            } else if tokens[index].is_symbol("OpenParen").is_ok() {
                "CloseParen"
            } else {
                return Err(format!("{} expected `OpenBracket`", log_color(loc)).into());
            };

            Self::bound_check(tokens, &mut index, seperator_name)?;

            let mut value: Vec<Token> = Vec::new();
            while tokens[index].is_symbol(seperator_name).is_err() {
                value.push(tokens[index].clone());
                Self::bound_check(tokens, &mut index, seperator_name)?;
            }

            if seperator_name == "CloseBracket" { // DEREFERENCE
                return Ok(Value::Deref(Box::new(Self::expr(&value, loc)?), Self::str_to_type(deref_type)));
            } else { // CAST
                return Ok(Value::Cast(Box::new(Self::expr(&value, loc)?), Self::str_to_type(deref_type)));
            }

        } else if tokens[index].is_symbol("And").is_ok() {
            // REFERENCE
            let loc = tokens[index].loc();

            Self::bound_check(tokens, &mut index, "Value")?;

            let mut value: Vec<Token> = Vec::new();
            while index < tokens.len() {
                value.push(tokens[index].clone());
                index += 1; // not accident
            }

            return Ok(Value::Ref(Box::new(Self::expr(&value, loc)?)));
        }
        return Ok(Value::Null);
    }

    fn is_binary_expr(tokens: &Vec<Token>) -> bool {
        let mut index = 0;
        while index < tokens.len() {
            if Self::is_operator(&tokens[index].clone(), tokens[index].loc()).is_ok() {
                return true;
            }
            index += 1; // intentional
        }
        return false;
    }

    fn is_function_call(tokens: &Vec<Token>) -> bool {
        if tokens.len() < 2 {
            return false;
        } else if tokens[0].is_ident().is_ok() && tokens[1].is_symbol("OpenParen").is_ok() {
            return true;
        }
        return false;
    }

    fn expr(tokens: &Vec<Token>, loc: (usize, usize)) -> Result<Value, Box<dyn std::error::Error>> {
        let mut index = 0;
        if index >= tokens.len() {
            return Err(format!("{} empty expression", log_color(loc)).into());
        }

        if Self::is_function_call(tokens) {
            // FUNCTION CALL
            let loc = tokens[index].loc();
            let name = tokens[index].is_ident()?;

            Self::bound_check(tokens, &mut index, "OpenParen")?;
            if tokens[index].is_symbol("OpenParen").is_err() {
                return Err(format!("{} expected `(` in function call", log_color(loc)).into());
            }

            Self::bound_check(tokens, &mut index, "CloseParen")?;

            let mut params: Vec<Token> = Vec::new();
            while tokens[index].is_symbol("CloseParen").is_err() {
                params.push(tokens[index].clone());
                Self::bound_check(tokens, &mut index, "CloseParen")?;
            }

            return Ok(Value::FunctionCall {
                loc,
                name,
                params: Self::parse_call_params(&params)?,
            });
        } else if Self::is_binary_expr(tokens) {
            // l_expr [operator] r_expr
            let loc = tokens[index].loc();
            let mut l_expr: Vec<Token> = Vec::new();
            let l_loc = tokens[index].loc();
            while Self::is_operator(&tokens[index], tokens[index].loc()).is_err() {
                l_expr.push(tokens[index].clone());
                Self::bound_check(tokens, &mut index, "operator")?;
            }

            let op = Self::is_operator(&tokens[index], tokens[index].loc())?;

            Self::bound_check(tokens, &mut index, "SemiColon")?;

            let mut r_expr: Vec<Token> = Vec::new();
            let r_loc = tokens[index].loc();
            while index < tokens.len() { // this will always break out if its out of bounds
                r_expr.push(tokens[index].clone());
                index += 1; // intentional because if we bound check here it will always error
                            // because we iterate to the end of the vector
            }

            return Ok(Value::BinaryExpr {
                loc,
                l_expr: Box::new(Self::expr(&l_expr, l_loc)?),
                r_expr: Box::new(Self::expr(&r_expr, r_loc)?),
                op,
            });
        } else {
            // SINGLE EXPR
            return Self::single_expr(tokens, loc);
        }
    }

    fn parse_call_params(tokens: &Vec<Token>) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let mut index = 0;
        let mut params: Vec<Value> = Vec::new();
        let mut param: Vec<Token> = Vec::new();

        while index < tokens.len() {
            if tokens[index].is_symbol("Comma").is_ok() {
                params.push(Self::expr(&param, tokens[index - param.len()].loc())?);
                param = Vec::new();
            } else {
                param.push(tokens[index].clone());
            }
            index += 1; // safe
        }
        if param.len() != 0 {
            params.push(Self::expr(&param, tokens[index - param.len()].loc())?);
        }
        return Ok(params);
    }

    fn double_symbol(sym1: (Token, &str), sym2: (Token, &str)) -> Result<(), Box<dyn std::error::Error>> {
        if sym1.0.is_symbol(sym1.1).is_err()
            ||
           sym2.0.is_symbol(sym2.1).is_err()
        {
            let loc = sym1.0.loc();
            return Err(format!("{} expected `{}{}`", log_color(loc), sym1.1, sym2.1).into());
        }
        return Ok(());
    }

    fn is_type(token: Token) -> Result<&'static str, Box<dyn std::error::Error>> {
        if token.is_keyword("int").is_ok() {
            return Ok("int");
        } else if token.is_keyword("ptr").is_ok() {
            return Ok("ptr");
        }
        let loc = token.loc();
        return Err(format!("{} expected `type`", log_color(loc)).into());
    }

    fn str_to_type(str_t: &str) -> Type {
        return match str_t {
            "int" => Type::Int,
            "ptr" => Type::Ptr,
            _ => Type::Void,
        }
    }

    fn param(tokens: &Vec<Token>, global_loc: (usize, usize)) -> Result<(String, Type), Box<dyn std::error::Error>> {
        let mut index = 0;
        if tokens.len() == 0 {
            return Err(format!("{} expected `ident`", log_color(global_loc)).into());
        }
        // name -> type
        let name = match tokens[index].is_ident() {
            Ok(name) => name,
            Err(_) => {
                let loc = tokens[index].loc();
                return Err(format!("{} expected `ident`", log_color(loc)).into());
            },
        };

        Self::bound_check(tokens, &mut index, "Minus")?;
        Self::bound_check(tokens, &mut index, "Bthen")?;

        Self::double_symbol((tokens[index-1].clone(), "Minus"), (tokens[index].clone(), "BThen"))?;

        Self::bound_check(tokens, &mut index, "type")?;

        let name_t = Self::is_type(tokens[index].clone())?;

        return Ok((name, Self::str_to_type(name_t)));
    }

    fn scope(tokens: &Vec<Token>, index: &mut usize) -> Result<Vec<Token>, Box<dyn std::error::Error>> {
        let mut scope_c = 0;
        let mut scope: Vec<Token> = Vec::new();

        while (tokens[*index].is_symbol("CloseBrace").is_err() && scope_c == 0) || scope_c != 0 {
            if tokens[*index].is_symbol("OpenBrace").is_ok() {
                scope_c += 1;
            } else if tokens[*index].is_symbol("CloseBrace").is_ok() {
                scope_c -= 1;
            }
            scope.push(tokens[*index].clone());

            Self::bound_check(tokens, index, "CloseBrace")?;
        }

        return Ok(scope);
    }

    fn is_comparison_op(token: &Token, loc: (usize, usize)) -> Result<(), Box<dyn std::error::Error>> {
        return if token.is_symbol("Equal").is_ok() {
            Ok(())
        } else if token.is_symbol("Bang").is_ok() {
            Ok(())
        } else if token.is_symbol("BThen").is_ok() {
            Ok(())
        } else if token.is_symbol("SThen").is_ok() {
            Ok(())
        } else {
            Err(format!("{} expected `Comparison Operator`", log_color(loc)).into())
        }
    }

    fn comparison_op(tokens: &Vec<Token>, index: &mut usize) -> Result<ComparisonOp, Box<dyn std::error::Error>> {
        if tokens[*index - 1].is_symbol("Equal").is_ok() && tokens[*index].is_symbol("Equal").is_ok() {
            return Ok(ComparisonOp::Equal);
        } else if tokens[*index - 1].is_symbol("Bang").is_ok() && tokens[*index].is_symbol("Equal").is_ok() {
            return Ok(ComparisonOp::NotEqual);
        } else if tokens[*index].is_symbol("BThen").is_ok() {
            return Ok(ComparisonOp::Bigger);
        } else if tokens[*index].is_symbol("SThen").is_ok() {
            return Ok(ComparisonOp::Smaller);
        } else {
            let loc = tokens[*index].loc();
            return Err(format!("{} expected `Comparison Operator`", log_color(loc)).into());
        }
    }

    fn parse_comparison(tokens: &Vec<Token>, loc: (usize, usize)) -> Result<Comparison, Box<dyn std::error::Error>> {
        if tokens.len() == 0 {
            return Err(format!("{} expected `Comparison`", log_color(loc)).into());
        }

        let mut l_expr: Vec<Token> = Vec::new();
        let mut r_expr: Vec<Token> = Vec::new();
        let mut index = 0;
        let loc = tokens[index].loc();

        while Self::is_comparison_op(&tokens[index], tokens[index].loc()).is_err() {
            l_expr.push(tokens[index].clone());
            Self::bound_check(tokens, &mut index, "Comparison Operator")?;
        }

        // dont add index if the comparison is only 1 token
        if tokens[index].is_symbol("BThen").is_err() && tokens[index].is_symbol("SThen").is_err() {
            Self::bound_check(tokens, &mut index, "Comparison Operator")?;
        }

        let op = Self::comparison_op(tokens, &mut index)?;

        Self::bound_check(tokens, &mut index, "Value")?;

        let r_loc = tokens[index].loc();
        while index < tokens.len() {
            r_expr.push(tokens[index].clone());
            index += 1;
        }

        return Ok(Comparison {
            l_expr: Box::new(Self::expr(&l_expr, loc)?),
            r_expr: Box::new(Self::expr(&r_expr, r_loc)?),
            op,
        });
    }

    pub fn parse(tokens: &Vec<Token>) -> Result<Vec<Ast>, Box<dyn std::error::Error>> {
        let mut ast: Vec<Ast> = Vec::new();

        let mut index = 0;
        while index < tokens.len() {
            if let Ok(name) = tokens[index].is_ident() {
                Self::bound_check(tokens, &mut index, "Equal")?;
                if tokens[index].is_symbol("Equal").is_ok() {
                    // VARIABLE MUTATION
                    let loc = tokens[index - 1].loc();

                    Self::bound_check(tokens, &mut index, "SemiColon")?;

                    let mut value: Vec<Token> = Vec::new();
                    while tokens[index].is_symbol("SemiColon").is_err() {
                        value.push(tokens[index].clone());
                        Self::bound_check(tokens, &mut index, "SemiColon")?;
                    }

                    ast.push(Ast::MutateVar {
                        loc,
                        name,
                        value: Self::expr(&value, loc)?,
                    });
                } else {
                    index -= 1; // because we used bound_check in variable mutation
                    // FUNCTIONS DECLARATIONS
                    let loc = tokens[index].loc();
                    let mut parameters: Vec<(String, Type)> = Vec::new();

                    Self::bound_check(tokens, &mut index, "Colon")?;
                    Self::bound_check(tokens, &mut index, "Colon")?;

                    // ::
                    Self::double_symbol((tokens[index - 1].clone(), "Colon"), (tokens[index].clone(), "Colon"))?;

                    Self::bound_check(tokens, &mut index, "OpenParen")?;

                    // ()
                    if tokens[index].is_symbol("OpenParen").is_ok() {
                        let param_loc = tokens[index].loc();
                        Self::bound_check(tokens, &mut index, "CloseParen")?;
                        let mut param_tokens: Vec<Token> = Vec::new();
                        while tokens[index].is_symbol("CloseParen").is_err() {
                            if tokens[index].is_symbol("Comma").is_ok() {
                                parameters.push(Self::param(&param_tokens, param_loc)?);
                                param_tokens = Vec::new();
                            } else {
                                param_tokens.push(tokens[index].clone());
                            }
                            Self::bound_check(tokens, &mut index, "CloseParen")?;
                        }
                        if param_tokens.len() != 0 {
                            parameters.push(Self::param(&param_tokens, param_loc)?);
                        }
                    } else {
                        let loc = tokens[index].loc();
                        return Err(format!("{} expected `(` in function declaration", log_color(loc)).into());
                    }
                    Self::bound_check(tokens, &mut index, "Minus")?;
                    Self::bound_check(tokens, &mut index, "BThen")?;

                    // ->
                    Self::double_symbol((tokens[index - 1].clone(), "Minus"), (tokens[index].clone(), "BThen"))?;
                    Self::bound_check(tokens, &mut index, "type")?;

                    let return_t = Self::is_type(tokens[index].clone())?;
                    Self::bound_check(tokens, &mut index, "OpenBrace")?;

                    if tokens[index].is_symbol("OpenBrace").is_err() {
                        let loc = tokens[index].loc();
                        return Err(format!("{} expected `{{` to start function body", log_color(loc)).into());
                    }
                    Self::bound_check(tokens, &mut index, "CloseBrace")?;

                    // { }
                    let body = Self::scope(tokens, &mut index)?;

                    ast.push(Ast::Function {
                        loc,
                        name,
                        param_t: parameters,
                        return_t: Self::str_to_type(return_t),
                        body: Self::parse(&body)?,
                    });
                }
            } else if tokens[index].is_keyword("return").is_ok() {
                // RETURN STATEMENTS
                let loc = tokens[index].loc();
                let mut value: Vec<Token> = Vec::new();
                Self::bound_check(tokens, &mut index, "SemiColon")?;

                let value_loc = tokens[index].loc();
                while tokens[index].is_symbol("SemiColon").is_err() {
                    value.push(tokens[index].clone());
                    Self::bound_check(tokens, &mut index, "SemiColon")?;
                }

                ast.push(Ast::Return {
                    loc,
                    value: Self::expr(&value, value_loc)?,
                });
            } else if tokens[index].is_keyword("let").is_ok() {
                // VARIABLE DECLARATION
                let loc = tokens[index].loc();
                Self::bound_check(tokens, &mut index, "ident")?;

                // name
                let name = if let Ok(ident) = tokens[index].is_ident() {
                    ident
                } else {
                    let loc = tokens[index].loc();
                    return Err(format!("{} expected `ident` but got `{:?}`", log_color(loc), tokens[index]).into());
                };
                Self::bound_check(tokens, &mut index, "Minus")?;
                Self::bound_check(tokens, &mut index, "BThen")?;

                // ->
                Self::double_symbol((tokens[index - 1].clone(), "Minus"), (tokens[index].clone(), "BThen"))?;
                Self::bound_check(tokens, &mut index, "type")?;

                // type
                let var_t = Self::is_type(tokens[index].clone())?;
                Self::bound_check(tokens, &mut index, "Equal")?;

                // =
                if tokens[index].is_symbol("Equal").is_err() {
                    let loc = tokens[index].loc();
                    return Err(format!("{} expected `=` in variable declaration", log_color(loc)).into());
                }
                Self::bound_check(tokens, &mut index, "SemiColon")?;

                let value_loc = tokens[index].loc();
                let mut value: Vec<Token> = Vec::new();
                while tokens[index].is_symbol("SemiColon").is_err() {
                    value.push(tokens[index].clone());
                    Self::bound_check(tokens, &mut index, "SemiColon")?;
                }

                ast.push(Ast::Variable {
                    loc,
                    name,
                    var_t: Self::str_to_type(var_t),
                    value: Self::expr(&value, value_loc)?,
                });
            } else if tokens[index].is_keyword("if").is_ok() {
                // IF STATEMENT
                let loc = tokens[index].loc();

                Self::bound_check(tokens, &mut index, "OpenBrace")?;

                let mut comparison: Vec<Token> = Vec::new();
                let comparison_loc = tokens[index].loc();
                while tokens[index].is_symbol("OpenBrace").is_err() {
                    comparison.push(tokens[index].clone());
                    Self::bound_check(tokens, &mut index, "OpenBrace")?;
                }

                Self::bound_check(tokens, &mut index, "CloseBrace")?;

                // { }
                let body = Self::scope(tokens, &mut index)?;

                // no bound check because it will error if the if statement is at the end
                index += 1;
                let mut else_body: Vec<Token> = Vec::new();
                // NOTE: COMPRESS THIS SOMEHOW
                if index >= tokens.len() {
                    ast.push(Ast::If {
                        loc,
                        comparison: Self::parse_comparison(&comparison, comparison_loc)?,
                        body: Self::parse(&body)?,
                        else_body: Vec::new(),
                    });
                } else if tokens[index].is_keyword("else").is_err() {
                    ast.push(Ast::If {
                        loc,
                        comparison: Self::parse_comparison(&comparison, comparison_loc)?,
                        body: Self::parse(&body)?,
                        else_body: Vec::new(),
                    });
                    index -= 1;
                } else if tokens[index].is_keyword("else").is_ok() {
                    Self::bound_check(tokens, &mut index, "OpenBrace")?;
                    if tokens[index].is_keyword("if").is_ok() {
                        else_body.push(tokens[index].clone());
                        Self::bound_check(tokens, &mut index, "Comparison")?;

                        // modified version of scope function
                        let mut indentation = 0;

                        while index < tokens.len() {
                            if tokens[index].is_symbol("OpenBrace").is_ok() {
                                indentation += 1;
                            } else if tokens[index].is_symbol("CloseBrace").is_ok() {
                                indentation -= 1;
                            }
                            if tokens[index].is_symbol("CloseBrace").is_ok() && indentation == 0 {
                                if index + 1 < tokens.len() {
                                    if tokens[index + 1].is_keyword("else").is_err() {
                                        else_body.push(tokens[index].clone());
                                        break;
                                    }
                                } else {
                                    else_body.push(tokens[index].clone());
                                    break;
                                }
                            }
                            else_body.push(tokens[index].clone());
                            // no bound checking on purpose
                            // because the while loop checks it
                            index += 1;
                        }
                    } else {
                        Self::bound_check(tokens, &mut index, "CloseBrace")?;

                        else_body = Self::scope(tokens, &mut index)?;
                    }
                    ast.push(Ast::If {
                        loc,
                        comparison: Self::parse_comparison(&comparison, comparison_loc)?,
                        body: Self::parse(&body)?,
                        else_body: Self::parse(&else_body)?,
                    });
                }
            } else if tokens[index].is_keyword("while").is_ok() {
                // WHILE STATEMENT
                let loc = tokens[index].loc();

                Self::bound_check(tokens, &mut index, "OpenBrace")?;

                // while [STATEMENT] {
                let mut comparison: Vec<Token> = Vec::new();
                let comparison_loc = tokens[index].loc();
                while tokens[index].is_symbol("OpenBrace").is_err() {
                    comparison.push(tokens[index].clone());
                    Self::bound_check(tokens, &mut index, "OpenBrace")?;
                }

                Self::bound_check(tokens, &mut index, "CloseBrace")?;

                // { }
                let body = Self::scope(tokens, &mut index)?;

                ast.push(Ast::While {
                    loc,
                    comparison: Self::parse_comparison(&comparison, comparison_loc)?,
                    body: Self::parse(&body)?,
                });
            } else if let Ok(ptr_type) = Self::is_type(tokens[index].clone()) {
                // POINTER MUTATION
                let loc = tokens[index].loc();

                // [
                Self::bound_check(tokens, &mut index, "OpenBracket")?;
                if tokens[index].is_symbol("OpenBracket").is_err() {
                    return Err(format!("{} expected `OpenBracket`", log_color(loc)).into());
                }

                Self::bound_check(tokens, &mut index, "Value")?;

                // [ptr]
                let mut ptr: Vec<Token> = Vec::new();
                while tokens[index].is_symbol("CloseBracket").is_err() {
                    ptr.push(tokens[index].clone());
                    Self::bound_check(tokens, &mut index, "CloseBracket")?;
                }

                // =
                Self::bound_check(tokens, &mut index, "Equal")?;
                if tokens[index].is_symbol("Equal").is_err() {
                    return Err(format!("{} expected `Equal`", log_color(loc)).into());
                }

                Self::bound_check(tokens, &mut index, "Value")?;

                // value;
                let mut value: Vec<Token> = Vec::new();
                while tokens[index].is_symbol("SemiColon").is_err() {
                    value.push(tokens[index].clone());
                    Self::bound_check(tokens, &mut index, "SemiColon")?;
                }

                ast.push(Ast::MutatePtr {
                    loc,
                    ptr_type: Self::str_to_type(ptr_type),
                    ptr: Self::expr(&ptr, loc)?,
                    value: Self::expr(&value, loc)?,
                });
            } else if tokens[index].is_keyword("include").is_ok() {
                // INCLUDE
                let loc = tokens[index].loc();

                Self::bound_check(tokens, &mut index, "string")?;

                let include_path = if let Ok(string) = tokens[index].is_section("string") {
                    string
                } else {
                    return Err(format!("{} expected `string`", log_color(loc)).into());
                };

                ast.extend(generate_ast(&include_path));
            } else if tokens[index].is_keyword("asm").is_ok() {
                // INLINE ASM
                let loc = tokens[index].loc();

                Self::bound_check(tokens, &mut index, "OpenParen")?;
                if tokens[index].is_symbol("OpenParen").is_err() {
                    return Err(format!("{} expected `(` in function call", log_color(loc)).into());
                }

                Self::bound_check(tokens, &mut index, "CloseParen")?;

                let asm = match tokens[index].is_section("string") {
                    Ok(string) => string,
                    Err(_) => {
                        return Err(format!("{} expected `string` in inline assembly", log_color(loc)).into());
                    },
                };

                Self::bound_check(tokens, &mut index, "CloseParen")?;
                Self::bound_check(tokens, &mut index, "SemiColon")?;

                Self::double_symbol((tokens[index - 1].clone(), "CloseParen"), (tokens[index].clone(), "SemiColon"))?;

                ast.push(Ast::InlineAsm {
                    loc,
                    asm,
                });
            } else if tokens[index].is_section("comment").is_ok() {
                // skip the comment
            }
            index += 1;
        }
        return Ok(ast);
    }
}



