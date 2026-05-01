use std::process::exit;
use std::{collections::HashMap, error::Error};

use crate::error::LangError;

use crate::lexer::Token;

use crate::value::RuntimeValue;
pub fn parse(tokens: Vec<Token>, stack: &mut Vec<RuntimeValue>, variables: &mut HashMap<String, i32>) -> Result<(), Box<dyn Error>>{
    let mut iter = tokens.iter().peekable();
    while let Some(tk) = iter.next() {
        match tk {
            Token::Quit => {
                println!("Exiting program...");
                exit(0)
            }
            Token::Push => {
                let mut is_first = true;

                loop {
                    if !is_first {
                        match iter.peek() {
                            Some(&Token::NumberLit(_)) | Some(&Token::StrLit(_)) | Some(&Token::VarLit(_)) => { }
                            _ => break
                        }
                    }

                    match iter.next() {
                        Some(Token::NumberLit(n)) => stack.push(RuntimeValue::Int(*n)),
                        Some(Token::StrLit(s)) => {
                            let trimmed = s.trim_matches('\"').to_string();
                            stack.push(RuntimeValue::String(trimmed));
                        }
                        Some(Token::VarLit(s)) => {
                            if variables.contains_key(s) {
                                stack.push(RuntimeValue::Int(variables.remove(s).unwrap_or_else(|| unreachable!("push <var>"))));
                            } else {
                                   return Err(LangError::UndeclaredVar(s.to_string()).into());
                            }
                        }
                        other => {
                            return Err(LangError::UnexpectedToken {
                                exp: "Number or String".to_string(),
                                got: format!("{:?}", other),
                            }.into());
                        }
                    }
                    is_first = false;
                }
            }
            Token::Add => {
                if let (Some(a), Some(b)) = (stack.pop(), stack.pop()){
                    match (a.clone(), b.clone()){
                        (RuntimeValue::Int(_), RuntimeValue::Int(_)) => stack.push(a + b),
                        (type1, type2) => return Err(LangError::UnexpectedTypes { exp: (RuntimeValue::Int(0), RuntimeValue::Int(0)), got: (type1, type2) }.into())
                    }
                } else {
                    return Err(LangError::UnsufficientValues{
                        op: "add".to_string(),
                        exp: 2,
                        got: stack.len()
                    }.into());
                }
            }
            Token::Mul =>{
                if let (Some(a), Some(b)) = (stack.pop(), stack.pop()){
                    match (a.clone(), b.clone()){
                        (RuntimeValue::Int(_), RuntimeValue::Int(_)) => stack.push(a * b),
                        (type1, type2) => return Err(LangError::UnexpectedTypes { exp: (RuntimeValue::Int(0), RuntimeValue::Int(0)), got: (type1, type2) }.into())
                    }
                } else {
                    return Err(LangError::UnsufficientValues{
                        op: "mul".to_string(),
                        exp: 2,
                        got: stack.len()
                    }.into());
                }
            }
            Token::Sub =>{
                if let (Some(a), Some(b)) = (stack.pop(), stack.pop()){
                    match (a.clone(), b.clone()){
                        (RuntimeValue::Int(_), RuntimeValue::Int(_)) => stack.push(a - b),
                        (type1, type2) => return Err(LangError::UnexpectedTypes { exp: (RuntimeValue::Int(0), RuntimeValue::Int(0)), got: (type1, type2) }.into())
                    }
                } else {
                    return Err(LangError::UnsufficientValues{
                        op: "sub".to_string(),
                        exp: 2,
                        got: stack.len()
                    }.into());
                }
            }
            Token::Div =>{
                if let (Some(a), Some(b)) = (stack.pop(), stack.pop()){
                    match (a.clone(), b.clone()){
                        (RuntimeValue::Int(_), RuntimeValue::Int(_)) => stack.push(a / b),
                        (type1, type2) => return Err(LangError::UnexpectedTypes { exp: (RuntimeValue::Int(0), RuntimeValue::Int(0)), got: (type1, type2) }.into())
                    }
                } else {
                    return Err(LangError::UnsufficientValues{
                        op: "div".to_string(),
                        exp: 2,
                        got: stack.len()
                    }.into());
                }
            }
            Token::Pop => {
                let p = stack.pop();
                match p{
                    Some(p) => {
                        match p{
                            RuntimeValue::String(s) if s == "\\n" => println!(),
                            _ => print!("{p}"),
                        }
                    }
                    None => return Err(LangError::StackEmpty.into())
                }
            }
            Token::Drop => {
                if stack.is_empty(){
                    return Err(LangError::StackEmpty.into())
                }
                stack.pop().unwrap_or_else(|| unreachable!("drop"));
            }
            Token::Neg => {
                if let Some(n) = stack.pop(){
                    match n {
                        RuntimeValue::Int(_) => stack.push(-n),
                        other => return Err(LangError::UnexpectedType { exp: RuntimeValue::Int(0), got: other }.into())
                    }
                } else {
                    return Err(LangError::UnsufficientValues{
                        op: "neg".to_string(),
                        exp: 1,
                        got: stack.len()
                    }.into());
                }
            }
            Token::Dup => {
                if let Some(&Token::VarLit(s)) = iter.peek(){
                    iter.next();
                    if variables.contains_key(s){
                        stack.push(RuntimeValue::Int(variables[s]));
                        continue;
                    } else {
                        return Err(LangError::UndeclaredVar(s.to_string()).into())
                    }
                }
                if stack.is_empty() {
                    return Err(LangError::UnsufficientValues{
                        op: "dup".to_string(),
                        exp: 1,
                        got: stack.len()
                    }.into());
                }

                stack.push(stack[stack.len() - 1].clone())
            }
            Token::NumberLit(num) => return Err(LangError::InvalidToken(Token::NumberLit(*num)).into()),
            Token::StrLit(s) => return Err(LangError::InvalidToken(Token::StrLit(s.to_string())).into()),
            Token::Var => {
                let next_token = iter.next();
                parse_var(next_token, variables)?;
            }
            Token::Into => {
                let next_keyword = iter.next();
                let var_name: String = {
                    match next_keyword {
                        Some(Token::Var) => {
                            let str_next = iter.next();
                            parse_var(str_next, variables)?;
                            match str_next{
                                Some(Token::VarLit(s)) => s.to_string(),
                                _ => unreachable!("str_next")
                            }
                        },
                        Some(Token::VarLit(s)) if variables.contains_key(s) => s.to_string(),
                        Some(other) => return Err(LangError::UnexpectedToken {
                            exp: "Var or Str".to_string(),
                            got: format!("{:?}", other)
                        }.into()),
                        None => todo!(),
                    }
                };
                match stack.pop(){
                    Some(n) =>{
                        match n {
                            RuntimeValue::Int(n) => variables.insert(var_name, n),
                            other => return Err(LangError::UnexpectedType { exp: RuntimeValue::Int(0), got: other }.into())
                        }
                    }
                    None => return Err(LangError::StackEmpty.into())
                };
            }
            Token::Swp => {
                if stack.len() < 2{
                    return Err(LangError::UnsufficientValues{
                        op: "swp".to_string(),
                        exp: 2,
                        got: stack.len()
                    }.into());
                }
                let n1 = stack.pop().unwrap_or_else(|| unreachable!("swp"));
                let n2 = stack.pop().unwrap_or_else(|| unreachable!("swp"));

                stack.push(n1);
                stack.push(n2);
            }
            Token::Rot => stack.reverse(),
            Token::Over => {
                if stack.len() < 2{
                    return Err(LangError::UnsufficientValues{
                        op: "over".to_string(),
                        exp: 2,
                        got: stack.len()
                    }.into());
                }

                stack.push(stack[stack.len() - 2].clone());
            }
            Token::BoolLit(b) => stack.push(RuntimeValue::Bool(*b)),
            Token::Eq => {
                match iter.peek() {
                    Some(&Token::NumberLit(n_arg)) => {
                        iter.next();

                        if stack.is_empty() {
                            return Err(LangError::UnsufficientValues {
                                op: "eq".to_string(),
                                exp: 1,
                                got: stack.len()
                            }.into());
                        }

                        let a = stack.pop().unwrap_or_else(|| unreachable!("eq"));

                        stack.push(RuntimeValue::Bool(a == RuntimeValue::Int(*n_arg)));
                    }
                    Some(&Token::VarLit(s)) => {
                        iter.next();

                        if stack.is_empty() {
                            return Err(LangError::UnsufficientValues {
                                op: "eq".to_string(),
                                exp: 1,
                                got: stack.len()
                            }.into());
                        }

                        let a = stack.pop().unwrap_or_else(|| unreachable!("eq"));

                        if variables.contains_key(s) {
                            stack.push(RuntimeValue::Bool(a == RuntimeValue::Int(variables[s])));
                        } else {
                            return Err(LangError::UndeclaredVar(s.to_string()).into());
                        }
                    }
                    _ => {
                        if stack.len() < 2 {
                            return Err(LangError::UnsufficientValues {
                                op: "eq".to_string(),
                                exp: 2,
                                got: stack.len()
                            }.into());
                        }

                        let b = stack.pop().unwrap_or_else(|| unreachable!("eq"));
                        let a = stack.pop().unwrap_or_else(|| unreachable!("eq"));

                        stack.push(RuntimeValue::Bool(a == b));
                    }
                }
            }
            Token::Gt => {
                match iter.peek() {
                    Some(&Token::NumberLit(n_arg)) => {
                        iter.next();

                        if stack.is_empty() {
                            return Err(LangError::UnsufficientValues {
                                op: "gt".to_string(),
                                exp: 1,
                                got: stack.len()
                            }.into());
                        }

                        let a = stack.pop().unwrap_or_else(|| unreachable!("gt"));

                        match a {
                            RuntimeValue::Int(n1) => stack.push(RuntimeValue::Bool(n1 > *n_arg)),
                            other => return Err(LangError::UnexpectedType {
                                exp: RuntimeValue::Int(0),
                                got: other
                            }.into())
                        }
                    }
                    Some(&Token::VarLit(s)) => {
                        iter.next();

                        if stack.is_empty() {
                            return Err(LangError::UnsufficientValues {
                                op: "gt".to_string(),
                                exp: 1,
                                got: stack.len()
                            }.into());
                        }

                        let a = stack.pop().unwrap_or_else(|| unreachable!("gt"));

                        if variables.contains_key(s) {
                            stack.push(RuntimeValue::Bool(a > RuntimeValue::Int(variables[s])));
                        } else {
                            return Err(LangError::UndeclaredVar(s.to_string()).into());
                        }
                    }
                    _ => {
                        if stack.len() < 2 {
                            return Err(LangError::UnsufficientValues {
                                op: "gt".to_string(),
                                exp: 2,
                                got: stack.len()
                            }.into());
                        }

                        let b = stack.pop().unwrap_or_else(|| unreachable!("gt"));
                        let a = stack.pop().unwrap_or_else(|| unreachable!("gt"));

                        match (a, b) {
                            (RuntimeValue::Int(n1), RuntimeValue::Int(n2)) => {
                                stack.push(RuntimeValue::Bool(n1 > n2));
                            }
                            (type1, type2) => {
                                return Err(LangError::UnexpectedTypes {
                                    exp: (RuntimeValue::Int(0), RuntimeValue::Int(0)),
                                    got: (type1, type2)
                                }.into());
                            }
                        }
                    }
                }
            }
            Token::Lt => {
                match iter.peek() {
                    Some(&Token::NumberLit(n_arg)) => {
                        iter.next();

                        if stack.is_empty() {
                            return Err(LangError::UnsufficientValues {
                                op: "lt".to_string(),
                                exp: 1,
                                got: stack.len()
                            }.into());
                        }

                        let a = stack.pop().unwrap_or_else(|| unreachable!("lt"));

                        match a {
                            RuntimeValue::Int(n1) => stack.push(RuntimeValue::Bool(n1 < *n_arg)),
                            other => return Err(LangError::UnexpectedType {
                                exp: RuntimeValue::Int(0),
                                got: other
                            }.into())
                        }
                    }
                    Some(&Token::VarLit(s)) => {
                        iter.next();

                        if stack.is_empty() {
                            return Err(LangError::UnsufficientValues {
                                op: "lt".to_string(),
                                exp: 1,
                                got: stack.len()
                            }.into());
                        }

                        let a = stack.pop().unwrap_or_else(|| unreachable!("lt"));

                        if variables.contains_key(s) {
                            stack.push(RuntimeValue::Bool(a < RuntimeValue::Int(variables[s])));
                        } else {
                            return Err(LangError::UndeclaredVar(s.to_string()).into());
                        }
                    }
                    _ => {
                        if stack.len() < 2 {
                            return Err(LangError::UnsufficientValues {
                                op: "lt".to_string(),
                                exp: 2,
                                got: stack.len()
                            }.into());
                        }

                        let b = stack.pop().unwrap_or_else(|| unreachable!("lt"));
                        let a = stack.pop().unwrap_or_else(|| unreachable!("lt"));

                        match (a, b) {
                            (RuntimeValue::Int(n1), RuntimeValue::Int(n2)) => {
                                stack.push(RuntimeValue::Bool(n1 < n2));
                            }
                            (type1, type2) => {
                                return Err(LangError::UnexpectedTypes {
                                    exp: (RuntimeValue::Int(0), RuntimeValue::Int(0)),
                                    got: (type1, type2)
                                }.into());
                            }
                        }
                    }
                }
            }
            Token::If => {
                if stack.is_empty() {
                    return Err(LangError::UnsufficientValues {
                        op: "if".to_string(),
                        exp: 1,
                        got: stack.len()
                    }.into());
                }
                let a = stack.pop().unwrap_or_else(|| unreachable!("if"));

                let condition = match a {
                    RuntimeValue::Bool(c) => c,
                    other => return Err(LangError::UnexpectedType {
                        exp: RuntimeValue::Bool(true),
                        got: other
                    }.into())
                };

                let next_token_option = iter.next();
                match next_token_option {
                    Some(next_token) => {
                        if *next_token != Token::OpenCurly {
                            return Err(LangError::UnexpectedToken { exp: "OpenCurly".to_string(), got: format!("{:?}", next_token) }.into());
                        }
                    }
                    None => return Err(LangError::UnexpectedToken { exp: "OpenCurly".to_string(), got: "None".to_string() }.into())
                }

                let mut if_branch_vec: Vec<Token> = vec![];
                let mut brace_depth = 1;

                for inner_tk in &mut iter {
                    match inner_tk {
                        Token::OpenCurly => {
                            brace_depth += 1;
                            if_branch_vec.push(inner_tk.clone());
                        }
                        Token::CloseCurly => {
                            brace_depth -= 1;
                            if brace_depth == 0 {
                                break;
                            } else {
                                if_branch_vec.push(inner_tk.clone());
                            }
                        }
                        _ => {
                            if_branch_vec.push(inner_tk.clone());
                        }
                    }
                }

                if let Some(&Token::Else) = iter.peek() {
                    iter.next();

                    match iter.next() {
                        Some(tk) => {
                            if *tk != Token::OpenCurly {
                                return Err(LangError::UnexpectedToken { exp: "OpenCurly".to_string(), got: format!("{:?}", tk) }.into());
                            }
                        }
                        None => return Err(LangError::UnexpectedToken { exp: "OpenCurly".to_string(), got: "None".to_string() }.into()),
                    }

                    let mut else_branch_vec: Vec<Token> = vec![];
                    let mut else_brace_depth = 1;

                    for inner_tk in &mut iter {
                        match inner_tk {
                            Token::OpenCurly => {
                                else_brace_depth += 1;
                                else_branch_vec.push(inner_tk.clone());
                            }
                            Token::CloseCurly => {
                                else_brace_depth -= 1;
                                if else_brace_depth == 0 {
                                    break;
                                } else {
                                    else_branch_vec.push(inner_tk.clone());
                                }
                            }
                            _ => {
                                else_branch_vec.push(inner_tk.clone());
                            }
                        }
                    }

                    if condition {
                        parse(if_branch_vec, stack, variables)?;
                    } else {
                        parse(else_branch_vec, stack, variables)?;
                    }
                } else {
                    if condition {
                        parse(if_branch_vec, stack, variables)?;
                    }
                }
            }
            Token::Else => return Err(LangError::InvalidToken(Token::Else).into()),
            Token::OpenCurly => { },
            Token::CloseCurly => { },
            Token::And => {
                match iter.peek() {
                    Some(&Token::BoolLit(b_arg)) => {
                        iter.next();

                        if stack.is_empty() {
                            return Err(LangError::UnsufficientValues {
                                op: "and".to_string(),
                                exp: 1,
                                got: stack.len()
                            }.into());
                        }

                        let a = stack.pop().unwrap_or_else(|| unreachable!("and"));

                        match a {
                            RuntimeValue::Bool(b_stack) => stack.push(RuntimeValue::Bool(b_stack && *b_arg)),
                            other => return Err(LangError::UnexpectedType {
                                exp: RuntimeValue::Int(0),
                                got: other
                            }.into())
                        }
                    }
                    // TODO: add support to bool vars
                    // Some(&Token::VarLit(s)) => {
                    //     iter.next();
                    //
                    //     if stack.is_empty() {
                    //         return Err(LangError::UnsufficientValues {
                    //             op: "and".to_string(),
                    //             exp: 1,
                    //             got: stack.len()
                    //         }.into());
                    //     }
                    //
                    //     let a = stack.pop().unwrap_or_else(|| unreachable!("and"));
                    //
                    //     if variables.contains_key(s) {
                    //         stack.push(RuntimeValue::Bool(a < RuntimeValue::Int(variables[s])));
                    //     } else {
                    //         return Err(LangError::UndeclaredVar(s.to_string()).into());
                    //     }
                    // }
                    _ => {
                        if stack.len() < 2 {
                            return Err(LangError::UnsufficientValues {
                                op: "and".to_string(),
                                exp: 2,
                                got: stack.len()
                            }.into());
                        }

                        let b = stack.pop().unwrap_or_else(|| unreachable!("and"));
                        let a = stack.pop().unwrap_or_else(|| unreachable!("and"));

                        match (a, b) {
                            (RuntimeValue::Bool(b1), RuntimeValue::Bool(b2)) => {
                                stack.push(RuntimeValue::Bool(b1 && b2));
                            }
                            (type1, type2) => {
                                return Err(LangError::UnexpectedTypes {
                                    exp: (RuntimeValue::Bool(false), RuntimeValue::Bool(false)),
                                    got: (type1, type2)
                                }.into());
                            }
                        }
                    }
                }
            }
            Token::Or => {
                match iter.peek() {
                    Some(&Token::BoolLit(b_arg)) => {
                        iter.next();

                        if stack.is_empty() {
                            return Err(LangError::UnsufficientValues {
                                op: "or".to_string(),
                                exp: 1,
                                got: stack.len()
                            }.into());
                        }

                        let a = stack.pop().unwrap_or_else(|| unreachable!("or"));

                        match a {
                            RuntimeValue::Bool(b_stack) => stack.push(RuntimeValue::Bool(b_stack || *b_arg)),
                            other => return Err(LangError::UnexpectedType {
                                exp: RuntimeValue::Int(0),
                                got: other
                            }.into())
                        }
                    }
                    // TODO: add support to bool vars
                    // Some(&Token::VarLit(s)) => {
                    //     iter.next();
                    //
                    //     if stack.is_empty() {
                    //         return Err(LangError::UnsufficientValues {
                    //             op: "or".to_string(),
                    //             exp: 1,
                    //             got: stack.len()
                    //         }.into());
                    //     }
                    //
                    //     let a = stack.pop().unwrap_or_else(|| unreachable!("or"));
                    //
                    //     if variables.contains_key(s) {
                    //         stack.push(RuntimeValue::Bool(a < RuntimeValue::Int(variables[s])));
                    //     } else {
                    //         return Err(LangError::UndeclaredVar(s.to_string()).into());
                    //     }
                    // }
                    _ => {
                        if stack.len() < 2 {
                            return Err(LangError::UnsufficientValues {
                                op: "or".to_string(),
                                exp: 2,
                                got: stack.len()
                            }.into());
                        }

                        let b = stack.pop().unwrap_or_else(|| unreachable!("or"));
                        let a = stack.pop().unwrap_or_else(|| unreachable!("or"));

                        match (a, b) {
                            (RuntimeValue::Bool(b1), RuntimeValue::Bool(b2)) => {
                                stack.push(RuntimeValue::Bool(b1 || b2));
                            }
                            (type1, type2) => {
                                return Err(LangError::UnexpectedTypes {
                                    exp: (RuntimeValue::Bool(false), RuntimeValue::Bool(false)),
                                    got: (type1, type2)
                                }.into());
                            }
                        }
                    }
                }
            }
            Token::VarLit(_) => todo!("report this"),
        }
    }
    Ok(())
}

fn parse_var(v: Option<&Token>, variables: &mut HashMap<String, i32>) -> Result<(), Box<dyn Error>>{
    match v{
        Some(tk) => match tk{
            Token::VarLit(s) => {
                if variables.contains_key(s){
                    return Err(LangError::RedeclarationVar(s.clone()).into());
                }
                variables.insert(s.clone(), 0)
            }
            _ => return Err(LangError::UnexpectedToken {
                exp: "Str".to_string(),
                got: format!("{:?}", v)
            }.into())
        }
        None => return Err(LangError::UnexpectedToken {
            exp: "Str".to_string(),
            got: "None".to_string(),
        }.into())
    };
    Ok(())
}
