use std::fs::{File, read_to_string};
use std::io::{BufRead, BufReader};
use std::iter::Peekable;
use std::process::exit;
use std::{collections::HashMap, error::Error};

use crate::error::{LangError, ret_error};
use crate::lexer::{Token, tokenize};
use crate::value::RuntimeValue;

pub const STD_LIB_PATH: &str = "/home/joaogabriel/Personal/Programming/projects/slur/std";

pub enum Flow{
    Next,
    Return
}

pub fn parse(
    tokens: Vec<Token>,
    stack: &mut Vec<RuntimeValue>,
    variables: &mut HashMap<String, RuntimeValue>,
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
                                stack.push(variables.remove(s).unwrap_or_else(|| unreachable!("push <var>")));
                            } else {
                                ret_error!(UndeclaredObject { t: "variable", name: s })
                            }
                        }
                        other => {
                            ret_error!(UnexpectedToken,[QuotedLit, UnquotedLit, NumberLit], other)
                        }
                    }
                    is_first = false;
                }
            }
            Token::Add => {
                if let (Some(a), Some(b)) = (stack.pop(), stack.pop()){
                    match (a.clone(), b.clone()){
                        (RuntimeValue::Int(_), RuntimeValue::Int(_)) => stack.push(a + b),
                        (type1, type2) => ret_error!(UnexpectedTypes,[RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![Some(type1), Some(type2)])
                    }
                } else {
                    ret_error!(UnsufficientValues { op: "add", exp: 2_usize, got: stack.len() })
                }
            }
            Token::Mul =>{
                if let (Some(a), Some(b)) = (stack.pop(), stack.pop()){
                    match (a.clone(), b.clone()){
                        (RuntimeValue::Int(_), RuntimeValue::Int(_)) => stack.push(a * b),
                        (type1, type2) => ret_error!(UnexpectedTypes,[RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![Some(type1), Some(type2)])
                    }
                } else {
                    ret_error!(UnsufficientValues { op: "mul", exp: 2_usize, got: stack.len() })
                }
            }
            Token::Sub =>{
                if let (Some(a), Some(b)) = (stack.pop(), stack.pop()){
                    match (a.clone(), b.clone()){
                        (RuntimeValue::Int(_), RuntimeValue::Int(_)) => stack.push(a - b),
                        (type1, type2) => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![Some(type1), Some(type2)])
                    }
                } else {
                    ret_error!(UnsufficientValues { op: "sub", exp: 2_usize, got: stack.len() })
                }
            }
            Token::Div =>{
                if let (Some(a), Some(b)) = (stack.pop(), stack.pop()){
                    match (a.clone(), b.clone()){
                        (RuntimeValue::Int(_), RuntimeValue::Int(_)) => stack.push(a / b),
                        (type1, type2) => ret_error!(UnexpectedTypes,[RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![Some(type1), Some(type2)])
                    }
                } else {
                    ret_error!(UnsufficientValues { op: "div", exp: 2_usize, got: stack.len() })
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
                    None => ret_error!(StackEmpty)
                }
            }
            Token::Drop => {
                if stack.is_empty(){
                    ret_error!(StackEmpty)
                }
                stack.pop().unwrap_or_else(|| unreachable!("drop"));
            }
            Token::Neg => {
                if let Some(n) = stack.pop(){
                    match n {
                        RuntimeValue::Int(_) => stack.push(-n),
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![Some(other)])
                    }
                } else {
                    ret_error!(UnsufficientValues { op: "neg", exp: 1_usize, got: stack.len() })
                }
            }
            Token::Dup => {
                if let Some(&Token::UnquotedLit(s)) = iter.peek(){
                    iter.next();
                    if variables.contains_key(s){
                        stack.push(variables[s].clone());
                        continue;
                    } else {
                        ret_error!(UndeclaredObject { t: "variable", name: s })
                    }
                }
                if stack.is_empty() {
                    ret_error!(UnsufficientValues { op: "dup", exp: 1_usize, got: stack.len() })
                }

                stack.push(stack[stack.len() - 1].clone())
            }
            Token::NumberLit(num) => ret_error!(InvalidToken, Token::NumberLit(*num)),
            Token::QuotedLit(s) => ret_error!(InvalidToken, Token::QuotedLit(s.to_string())),
            Token::UnquotedLit(s) => ret_error!(InvalidToken, Token::UnquotedLit(s.to_string())),
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
                        Some(other) => ret_error!(UnexpectedToken, [Var, UnquotedLit], Some(other.clone())),
                        None => todo!(),
                    }
                };
                match stack.pop(){
                    Some(n) => variables.insert(var_name, n),
                    None => ret_error!(StackEmpty)
                };
            }
            Token::Swap => {
                if stack.len() < 2{
                    ret_error!(UnsufficientValues { op: "Swap", exp: 2_usize, got: stack.len() })
                }
                let n1 = stack.pop().unwrap_or_else(|| unreachable!("Swap"));
                let n2 = stack.pop().unwrap_or_else(|| unreachable!("Swap"));

                stack.push(n1);
                stack.push(n2);
            }
            Token::Rot => stack.reverse(),
            Token::Over => {
                if stack.len() < 2{
                    ret_error!(UnsufficientValues { op: "over", exp: 2_usize, got: stack.len() })
                }

                stack.push(stack[stack.len() - 2].clone());
            }
            Token::BoolLit(b) => stack.push(RuntimeValue::Bool(*b)),
            Token::Eq => {
                match iter.peek() {
                    Some(&Token::NumberLit(n_arg)) => {
                        iter.next();

                        if stack.is_empty() {
                            ret_error!(UnsufficientValues { op: "eq", exp: 1_usize, got: stack.len() })
                        }

                        let a = stack.pop().unwrap_or_else(|| unreachable!("eq"));

                        stack.push(RuntimeValue::Bool(a == RuntimeValue::Int(*n_arg)));
                    }
                    Some(&Token::UnquotedLit(s)) => {
                        iter.next();

                        if stack.is_empty() {
                            ret_error!(UnsufficientValues { op: "eq", exp: 1_usize, got: stack.len() })
                        }

                        let a = stack.pop().unwrap_or_else(|| unreachable!("eq"));

                        if variables.contains_key(s) {
                            match variables[s].clone() {
                                RuntimeValue::Int(n) => stack.push(RuntimeValue::Bool(a == RuntimeValue::Int(n))),
                                other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![Some(other)]),
                            }
                        } else {
                            ret_error!(UndeclaredObject { t: "variable", name: s })
                        }
                    }
                    _ => {
                        if stack.len() < 2 {
                            ret_error!(UnsufficientValues { op: "eq", exp: 2_usize, got: stack.len() })
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
                            ret_error!(UnsufficientValues { op: "gt", exp: 1_usize, got: stack.len() })
                        }

                        let a = stack.pop().unwrap_or_else(|| unreachable!("gt"));

                        match a {
                            RuntimeValue::Int(n1) => stack.push(RuntimeValue::Bool(n1 > *n_arg)),
                            other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![Some(other)])
                        }
                    }
                    Some(&Token::UnquotedLit(s)) => {
                        iter.next();

                        if stack.is_empty() {
                            ret_error!(UnsufficientValues { op: "gt", exp: 1_usize, got: stack.len() })
                        }

                        let a = stack.pop().unwrap_or_else(|| unreachable!("gt"));

                        if variables.contains_key(s) {
                            match variables[s].clone() {
                                RuntimeValue::Int(n) => stack.push(RuntimeValue::Bool(a > RuntimeValue::Int(n))),
                                other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![Some(other)]),
                            }
                        } else {
                            ret_error!(UndeclaredObject { t: "variable", name: s })
                        }
                    }
                    _ => {
                        if stack.len() < 2 {
                            ret_error!(UnsufficientValues { op: "gt", exp: 2_usize, got: stack.len() })
                        }

                        let b = stack.pop().unwrap_or_else(|| unreachable!("gt"));
                        let a = stack.pop().unwrap_or_else(|| unreachable!("gt"));

                        match (a, b) {
                            (RuntimeValue::Int(n1), RuntimeValue::Int(n2)) => {
                                stack.push(RuntimeValue::Bool(n1 > n2));
                            }
                            (type1, type2) => {
                                ret_error!(UnexpectedTypes,[RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![Some(type1), Some(type2)])
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
                            ret_error!(UnsufficientValues { op: "lt", exp: 1_usize, got: stack.len() })
                        }

                        let a = stack.pop().unwrap_or_else(|| unreachable!("lt"));

                        match a {
                            RuntimeValue::Int(n1) => stack.push(RuntimeValue::Bool(n1 < *n_arg)),
                            other => ret_error!(UnexpectedTypes,[RuntimeValue::Int(0)], vec![Some(other)])
                        }
                    }
                    Some(&Token::UnquotedLit(s)) => {
                        iter.next();

                        if stack.is_empty() {
                            ret_error!(UnsufficientValues { op: "lt", exp: 1_usize, got: stack.len() })
                        }

                        let a = stack.pop().unwrap_or_else(|| unreachable!("lt"));

                        if variables.contains_key(s) {
                            match variables[s].clone() {
                                RuntimeValue::Int(n) => stack.push(RuntimeValue::Bool(a < RuntimeValue::Int(n))),
                                other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![Some(other)]),
                            }
                        } else {
                            ret_error!(UndeclaredObject { t: "variable", name: s })
                        }
                    }
                    _ => {
                        if stack.len() < 2 {
                            ret_error!(UnsufficientValues { op: "lt", exp: 2_usize, got: stack.len() })
                        }

                        let b = stack.pop().unwrap_or_else(|| unreachable!("lt"));
                        let a = stack.pop().unwrap_or_else(|| unreachable!("lt"));

                        match (a, b) {
                            (RuntimeValue::Int(n1), RuntimeValue::Int(n2)) => {
                                stack.push(RuntimeValue::Bool(n1 < n2));
                            }
                            (type1, type2) => {
                                ret_error!(UnexpectedTypes, [RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![Some(type1), Some(type2)])
                            }
                        }
                    }
                }
            }
            Token::If => {
                if stack.is_empty() {
                    ret_error!(UnsufficientValues { op: "if", exp: 1_usize, got: stack.len() })
                }
                let a = stack.pop().unwrap_or_else(|| unreachable!("if"));

                let condition = match a {
                    RuntimeValue::Bool(c) => c,
                    other => ret_error!(UnexpectedTypes,[RuntimeValue::Bool(true)], vec![Some(other)])
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
            Token::Else => ret_error!(InvalidToken, Token::Else),
            Token::OpenCurly => { },
            Token::CloseCurly => { },
            Token::And => {
                match iter.peek() {
                    Some(&Token::BoolLit(b_arg)) => {
                        iter.next();

                        if stack.is_empty() {
                            ret_error!(UnsufficientValues { op: "and", exp: 1_usize, got: stack.len() })
                        }

                        let a = stack.pop().unwrap_or_else(|| unreachable!("and"));

                        match a {
                            RuntimeValue::Bool(b_stack) => stack.push(RuntimeValue::Bool(b_stack && *b_arg)),
                            other => ret_error!(UnexpectedTypes,[RuntimeValue::Bool(true)], vec![Some(other)])
                        }
                    }
                    // TODO: add support to bool vars
                    // Some(&Token::UnquotedLit(s)) => {
                    //     iter.next();
                    //
                    //     if stack.is_empty() {
                    //         return Err(LangError::UnsufficientValues {
                    //             op: "and".to_string(),
                    //             exp: 1_usize,
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
                            ret_error!(UnsufficientValues { op: "and", exp: 2_usize, got: stack.len() })
                        }

                        let b = stack.pop().unwrap_or_else(|| unreachable!("and"));
                        let a = stack.pop().unwrap_or_else(|| unreachable!("and"));

                        match (a, b) {
                            (RuntimeValue::Bool(b1), RuntimeValue::Bool(b2)) => {
                                stack.push(RuntimeValue::Bool(b1 && b2));
                            }
                            (type1, type2) => ret_error!(UnexpectedTypes, [RuntimeValue::Bool(false), RuntimeValue::Bool(false)], vec![Some(type1), Some(type2)])
                        }
                    }
                }
            }
            Token::Or => {
                match iter.peek() {
                    Some(&Token::BoolLit(b_arg)) => {
                        iter.next();

                        if stack.is_empty() {
                            ret_error!(UnsufficientValues { op: "or", exp: 1_usize, got: stack.len() })
                        }

                        let a = stack.pop().unwrap_or_else(|| unreachable!("or"));

                        match a {
                            RuntimeValue::Bool(b_stack) => stack.push(RuntimeValue::Bool(b_stack || *b_arg)),
                            other => ret_error!(UnexpectedTypes, [RuntimeValue::Bool(true)], vec![Some(other)])
                        }
                    }
                    // TODO: add support to bool vars
                    // Some(&Token::UnquotedLit(s)) => {
                    //     iter.next();
                    //
                    //     if stack.is_empty() {
                    //         return Err(LangError::UnsufficientValues {
                    //             op: "or".to_string(),
                    //             exp: 1_usize,
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
                            ret_error!(UnsufficientValues { op: "or", exp: 2_usize, got: stack.len() })
                        }

                        let b = stack.pop().unwrap_or_else(|| unreachable!("or"));
                        let a = stack.pop().unwrap_or_else(|| unreachable!("or"));

                        match (a, b) {
                            (RuntimeValue::Bool(b1), RuntimeValue::Bool(b2)) => {
                                stack.push(RuntimeValue::Bool(b1 || b2));
                            }
                            (type1, type2) => {
                                ret_error!(UnexpectedTypes,[RuntimeValue::Bool(false), RuntimeValue::Bool(false)], vec![Some(type1), Some(type2)])
                            }
                        }
                    }
                }
            }
            Token::Not => {
                if stack.is_empty(){
                    ret_error!(StackEmpty)
                }

                match stack.pop().unwrap_or_else(|| unreachable!("not")){
                    RuntimeValue::Bool(b) => stack.push(RuntimeValue::Bool(!b)),
                    other => ret_error!(UnexpectedTypes, [RuntimeValue::Bool(false)], vec![Some(other)])
                }
            }
            Token::FunDeclaration(fun_name) => {
                expect_open_curly(iter.next())?;
                let fun_block = collect_tokens_into_block(&mut iter);
                if functions.contains_key(fun_name){
                    ret_error!(RedeclarationObject { t: "function", name: fun_name })
                }
                functions.insert(fun_name.to_string(), fun_block);
            }
            Token::FunCall(fun_name) => {
                if !functions.contains_key(fun_name) {
                    ret_error!(UndeclaredObject { t: "function", name: fun_name })
                }
                parse(functions[fun_name].clone(), stack, variables, functions)?;
            }
            Token::Len => stack.push(RuntimeValue::Int(stack.len().try_into().unwrap())),
            Token::Split => {
                if stack.len() < 2 {
                    ret_error!(UnsufficientValues { op: "Split", exp: 2_usize, got: stack.len() })
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
                    ret_error!(UnsufficientValues { op: "Split", exp: 2_usize, got: stack.len() })
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
                            return Err(LangError::InvalidPath(s.clone()).into());
                        }
                        let new_s = format!("{s}.slur");

                        let target_path = std::path::Path::new(STD_LIB_PATH).join(new_s);

                        match read_to_string(&target_path) {
                            Ok(content) => {
                                let tokens = tokenize(content);
                                parse(tokens, stack, variables, functions)?;
                            }
                            Err(_) => ret_error!(FileNotFound { file: s.clone(), reason: "No module with this name." })
                        }
                    }
                    other => ret_error!(UnexpectedToken,[QuotedLit, UnquotedLit], other)
                }
            }
            Token::ReadLine => {
                if stack.len() < 2{
                    ret_error!(UnsufficientValues { op: "readline", exp: 2_usize, got: stack.len() })
                }

                let line_num: usize = match stack.pop().unwrap_or_else(|| unreachable!("readline")){
                    RuntimeValue::Int(x) if x > 0 => x as usize,
                    RuntimeValue::Int(_) => todo!("return new error type to line idx out of range"),
                    other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![Some(other)])
                };
                let path = match stack.pop().unwrap_or_else(|| unreachable!("readline")){
                    RuntimeValue::String(s) => s,
                    other => ret_error!(UnexpectedTypes, [RuntimeValue::String("".to_string())], vec![Some(other)])
                };

                let file = File::open(path)?;
                let reader = BufReader::new(file);

                if let Some(line) = reader.lines().nth(line_num - 1){
                    match line {
                        Ok(l) => stack.push(RuntimeValue::String(l)),
                        _ => todo!("return new error type to line idx out of range")
                    }
                }
            }
            Token::ReadLineB => {
                if stack.len() < 2{
                    stack.push(RuntimeValue::Bool(false))
                }

                let line_num: usize = match stack.pop().unwrap_or_else(|| unreachable!("readline")){
                    RuntimeValue::Int(x) if x > 0 => x as usize,
                    RuntimeValue::Int(_) => todo!("return new error type to line idx out of range"),
                    other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![Some(other)])
                };
                let path = match stack.pop().unwrap_or_else(|| unreachable!("readline")){
                    RuntimeValue::String(s) => s,
                    other => ret_error!(UnexpectedTypes, [RuntimeValue::String("".to_string())], vec![Some(other)])
                };

                let file = File::open(path)?;
                let reader = BufReader::new(file);

                match reader.lines().nth(line_num - 1){
                    Some(Ok(l)) => {
                        stack.push(RuntimeValue::String(l));
                        stack.push(RuntimeValue::Bool(true));
                    }
                    _ => stack.push(RuntimeValue::Bool(false))
                }
            }
            Token::Int => {
                if stack.is_empty(){
                    ret_error!(StackEmpty)
                }

                match stack.pop().unwrap_or_else(|| unreachable!("intb")){
                    RuntimeValue::String(s) => match s.parse::<i32>(){
                        Ok(n) => stack.push(RuntimeValue::Int(n)),
                        Err(_) => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![Some(RuntimeValue::String(s))]) // TODO: fix this error handling
                    }
                    RuntimeValue::Bool(b) => if b {
                        stack.push(RuntimeValue::Int(1));
                    } else {
                        stack.push(RuntimeValue::Int(0));
                    }
                    RuntimeValue::Int(i) => stack.push(RuntimeValue::Int(i)),
                };
            }
            Token::IntB => {
                if stack.is_empty(){
                    ret_error!(StackEmpty)
                }

                match stack.pop().unwrap_or_else(|| unreachable!("intb")){
                    RuntimeValue::String(s) => match s.parse::<i32>(){
                        Ok(n) => {
                            stack.push(RuntimeValue::Int(n));
                            stack.push(RuntimeValue::Bool(true));
                        }
                        Err(_) => stack.push(RuntimeValue::Bool(false)),
                    }
                    RuntimeValue::Bool(b) => if b {
                        stack.push(RuntimeValue::Int(1));
                        stack.push(RuntimeValue::Bool(true));
                    } else {
                        stack.push(RuntimeValue::Int(0));
                        stack.push(RuntimeValue::Bool(false));
                    }
                    RuntimeValue::Int(i) =>{
                        stack.push(RuntimeValue::Int(i));
                        stack.push(RuntimeValue::Bool(true));
                    }
                };
            }
        }
    }
    Ok(Flow::Next)
}

fn parse_var(v: Option<&Token>, variables: &mut HashMap<String, RuntimeValue>) -> Result<(), Box<dyn Error>>{
    match v{
        Some(tk) => match tk{
            Token::UnquotedLit(s) => {
                if variables.contains_key(s){
                    ret_error!(RedeclarationObject { t: "variable", name: s })
                }
                variables.insert(s.clone(), RuntimeValue::Int(0))
            }
            _ => ret_error!(UnexpectedToken, [UnquotedLit], Some(tk))
        }
        None => ret_error!(UnexpectedToken, [UnquotedLit], None::<Token>)
    };
    Ok(())
}


fn expect_open_curly(nt: Option<&Token>) -> Result<(), Box<dyn Error>>{
    match nt {
        Some(next_token) => {
            if *next_token != Token::OpenCurly {
                ret_error!(UnexpectedToken, [OpenCurly], Some(next_token))
            } else {
                Ok(())
            }
        }
        None => {
            ret_error!(UnexpectedToken, [OpenCurly], None::<Token>)
        }
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
