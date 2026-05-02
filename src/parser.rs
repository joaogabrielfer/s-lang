use std::fs::{self, read_to_string};
use std::iter::Peekable;
use std::process::exit;
use std::{collections::HashMap, error::Error};

use crate::error::LangError;
use crate::lexer::{Token, tokenize};
use crate::value::RuntimeValue;

pub const STD_LIB_PATH: &str = "/home/joaogabriel/personal/programming/misc/slur/std";

pub enum Flow{
    Next,
    Return
}

pub fn parse(
    tokens: Vec<Token>,
    stack: &mut Vec<RuntimeValue>,
    variables: &mut HashMap<String, i32>,
    functions: &mut HashMap<String, Vec<Token>>
) -> Result<Flow, Box<dyn Error>>{
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
                            Some(&Token::NumberLit(_)) | Some(&Token::QuotedLit(_)) | Some(&Token::UnquotedLit(_)) => { }
                            _ => break
                        }
                    }

                    match iter.next() {
                        Some(Token::NumberLit(n)) => stack.push(RuntimeValue::Int(*n)),
                        Some(Token::QuotedLit(s)) => {
                            let trimmed = s.trim_matches('\"').to_string();
                            stack.push(RuntimeValue::String(trimmed));
                        }
                        Some(Token::UnquotedLit(s)) => {
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
                if let Some(&Token::UnquotedLit(s)) = iter.peek(){
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
            Token::QuotedLit(s) => return Err(LangError::InvalidToken(Token::QuotedLit(s.to_string())).into()),
            Token::UnquotedLit(s) => return Err(LangError::InvalidToken(Token::UnquotedLit(s.to_string())).into()),
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
                                Some(Token::UnquotedLit(s)) => s.to_string(),
                                _ => unreachable!("str_next")
                            }
                        },
                        Some(Token::UnquotedLit(s)) if variables.contains_key(s) => s.to_string(),
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
            Token::Swap => {
                if stack.len() < 2{
                    return Err(LangError::UnsufficientValues{
                        op: "Swap".to_string(),
                        exp: 2,
                        got: stack.len()
                    }.into());
                }
                let n1 = stack.pop().unwrap_or_else(|| unreachable!("Swap"));
                let n2 = stack.pop().unwrap_or_else(|| unreachable!("Swap"));

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
                    Some(&Token::UnquotedLit(s)) => {
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
                    Some(&Token::UnquotedLit(s)) => {
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
                    Some(&Token::UnquotedLit(s)) => {
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

                expect_open_curly(iter.next())?;

                let if_branch_vec = collect_tokens_into_block(&mut iter);
                if let Some(&Token::Else) = iter.peek() {
                    iter.next();

                    expect_open_curly(iter.next())?;

                    let else_branch_vec = collect_tokens_into_block(&mut iter);

                    if condition {
                        if let Flow::Return = parse(if_branch_vec, stack, variables, functions)?{
                            return Ok(Flow::Return);
                        }
                    } else {
                        if let Flow::Return = parse(else_branch_vec, stack, variables, functions)?{
                            return Ok(Flow::Return);
                        }
                    }
                } else {
                    if condition && let Flow::Return = parse(if_branch_vec, stack, variables, functions)? {
                            return Ok(Flow::Return);
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
                    // Some(&Token::UnquotedLit(s)) => {
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
                    // Some(&Token::UnquotedLit(s)) => {
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
            Token::Not => {
                if stack.is_empty(){
                    return Err(LangError::StackEmpty.into())
                }

                match stack.pop().unwrap_or_else(|| unreachable!("not")){
                    RuntimeValue::Bool(b) => stack.push(RuntimeValue::Bool(!b)),
                    other => return Err(LangError::UnexpectedType { exp: RuntimeValue::Bool(false), got: other }.into())
                }
            }
            Token::FunDeclaration(fun_name) => {
                expect_open_curly(iter.next())?;
                let fun_block = collect_tokens_into_block(&mut iter);
                if functions.contains_key(fun_name){
                    return Err(LangError::RedeclarationVar(fun_name.to_string()).into()) //TODO: modify this to generic error
                }
                functions.insert(fun_name.to_string(), fun_block);
            }
            Token::FunCall(fun_name) => {
                if !functions.contains_key(fun_name) {
                    return Err(LangError::UndeclaredVar(fun_name.to_string()).into()) // here too
                }
                parse(functions[fun_name].clone(), stack, variables, functions)?;
            }
            Token::Len => stack.push(RuntimeValue::Int(stack.len().try_into().unwrap())),
            Token::Split => {
                if stack.len() < 2 {
                    return Err(LangError::UnsufficientValues { op: "Split".to_string(), exp: 2, got: stack.len() }.into())
                }

                let pattern = stack.pop().unwrap_or_else(|| unreachable!("split"));
                let source = stack.pop().unwrap_or_else(|| unreachable!("split"));
                if let Some((left, right)) = source.to_string().split_once(&pattern.to_string()) {
                    stack.push(RuntimeValue::String(right.to_string()));
                    stack.push(RuntimeValue::String(left.to_string()));
                }
            }
            Token::SplitB => {
                if stack.len() < 2 {
                    return Err(LangError::UnsufficientValues { op: "Split".to_string(), exp: 2, got: stack.len() }.into())
                }
                let pattern = stack.pop().unwrap_or_else(|| unreachable!("split"));
                let source = stack.pop().unwrap_or_else(|| unreachable!("split"));
                let source_str = source.to_string();
                let result = source_str.split_once(&pattern.to_string());
                match result{
                    Some((left, right)) =>{
                        stack.push(RuntimeValue::String(right.to_string()));
                        stack.push(RuntimeValue::String(left.to_string()));
                        stack.push(RuntimeValue::Bool(true))
                    }
                    None => {
                        stack.push(source);
                        stack.push(RuntimeValue::Bool(false));
                    }
                }
            }
            Token::Ret => return Ok(Flow::Return),
            Token::Include => {
                match iter.next(){
                    Some(Token::QuotedLit(_s)) => {

                    },
                    Some(Token::UnquotedLit(s)) => {
                        if s.contains('/') || s.contains('\\') || s.contains("..") {
                            return Err(LangError::InvalidImport(s.clone()).into());
                        }
                        let new_s = format!("{s}.slur");

                        let target_path = std::path::Path::new(STD_LIB_PATH).join(new_s);

                        match read_to_string(&target_path) {
                            Ok(content) => {
                                let tokens = tokenize(content);
                                parse(tokens, stack, variables, functions)?;
                            }
                            Err(_) => {
                                return Err(LangError::FileNotFound {
                                    file: s.clone(),
                                    reason: "No module with this name.".to_string(),
                                }.into());
                            }
                        }
                    }
                    other => return Err(LangError::UnexpectedToken {
                        exp: "QuotedLit or UnquotedLit".to_string(),
                        got: format!("{:?}", other)
                    }.into())
                }
            }
        }
    }
    Ok(Flow::Next)
}

fn parse_var(v: Option<&Token>, variables: &mut HashMap<String, i32>) -> Result<(), Box<dyn Error>>{
    match v{
        Some(tk) => match tk{
            Token::UnquotedLit(s) => {
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


fn expect_open_curly(nt: Option<&Token>) -> Result<(), Box<dyn Error>>{
    match nt {
        Some(next_token) => {
            if *next_token != Token::OpenCurly {
                Err(LangError::UnexpectedToken { exp: "OpenCurly".to_string(), got: format!("{:?}", next_token) }.into())
            } else {
                Ok(())
            }
        }
        None => Err(LangError::UnexpectedToken { exp: "OpenCurly".to_string(), got: "None".to_string() }.into())
    }
}

fn collect_tokens_into_block(iter: &mut Peekable<std::slice::Iter<Token>>) -> Vec<Token>{
    let mut block_vector: Vec<Token> = vec![];
    let mut brace_depth = 1;

    for inner_tk in iter {
        match inner_tk {
            Token::OpenCurly => {
                brace_depth += 1;
                block_vector.push(inner_tk.clone());
            }
            Token::CloseCurly => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    break;
                } else {
                    block_vector.push(inner_tk.clone());
                }
            }
            _ => {
                block_vector.push(inner_tk.clone());
            }
        }
    };
    block_vector
}
