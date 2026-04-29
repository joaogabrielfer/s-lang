use std::{error::Error, fs, io::{self, Write}, process::exit, vec};

fn main() -> Result<(), Box<dyn Error>>{
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        let mut input = String::new();
        let mut stack : Vec<i32> = vec![];
        loop{
            print!("> ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input)?;

            let tokens = tokenize(input.clone());
            if let Err(e) = parse(tokens, &mut stack){
                eprintln!("ERROR: {e}")
            } else {
                println!("stack: {:?}", stack);
            }

            input.clear();
        }
    } else {
        let content = fs::read_to_string(args[1].clone())?;
        let tokens = tokenize(content);
        let mut stack : Vec<i32> = vec![];
        if let Err(e) = parse(tokens, &mut stack){
            eprintln!("ERROR: {e}")
        } else {
            println!("stack: {:?}", stack);
        }

        if !stack.is_empty(){
            println!("warning: trailing number still in the stack: {:?}", stack);
        }
    }
    

    Ok(())
}

#[derive(Debug, Clone)]
enum Token{
    Push(i32),
    Add,
    Sub,
    Mul,
    Div,
    Minus,
    Pop,
    Dup,
    Unknown
}

#[derive(Debug, Clone)]
enum LangError{
    ParsingUnknown,
    StackEmpty,
    UnsufficientValues(&'static str), 
}

impl std::fmt::Display for LangError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParsingUnknown => write!(f, "Could not parse expression."),
            Self::StackEmpty => write!(f, "Could not pop variable: stack empty"),
            Self::UnsufficientValues(operation) => write!(f, "Cannot {operation}: too much or too many variables in the stack"),
        }
    }
}

impl std::error::Error for LangError {}

fn tokenize(content : String) -> Vec<Token>{
    content
        .split_whitespace()
        .map(|x| match x{
            "add" => Token::Add,
            "sub" => Token::Sub,
            "mul" => Token::Mul,
            "div" => Token::Div,
            "sig" => Token::Minus,
            "pop" => Token::Pop,
            "dup" => Token::Dup,
            "quit" => {
                println!("Exiting program...");
                exit(0);
            },
            _ => {
                let num = x.parse::<i32>();
                match num {
                    Ok(num) => Token::Push(num),
                    Err(_num) => Token::Unknown,
                }
            }    
        })
    .collect()
}

fn parse(tokens : Vec<Token>, stack : &mut Vec<i32>) -> Result<(), Box<dyn Error>>{
    for tk in &tokens{
        match tk {
            Token::Unknown => return Err(LangError::ParsingUnknown.into()),
            Token::Push(num) => stack.push(*num),
            Token::Add => {
                if let (Some(a), Some(b)) = (stack.pop(), stack.pop()){
                    stack.push(a + b);
                } else {
                    return Err(LangError::UnsufficientValues("add").into());
                }
            }
            Token::Mul =>{
                if let (Some(a), Some(b)) = (stack.pop(), stack.pop()){
                    stack.push(a * b);
                } else {
                    return Err(LangError::UnsufficientValues("multiply").into());
                }
            }
            Token::Sub =>{
                if let (Some(a), Some(b)) = (stack.pop(), stack.pop()){
                    stack.push(a - b);
                } else {
                    return Err(LangError::UnsufficientValues("subtract").into());
                }
            }
            Token::Div =>{
                if let (Some(a), Some(b)) = (stack.pop(), stack.pop()){
                    stack.push(a / b);
                } else {
                    return Err(LangError::UnsufficientValues("divide").into());
                }
            }
            Token::Pop => {
                let p = stack.pop();
                match p{
                    Some(p) => println!("pop: {p}"),
                    None => return Err(LangError::StackEmpty.into())
                }
            }
            Token::Minus => {
                if let Some(n) = stack.pop(){
                    stack.push(-n);
                } else {
                    return Err(LangError::UnsufficientValues("Minus").into());
                }
            }
            Token::Dup => {
                if stack.is_empty() { 
                    return Err(LangError::UnsufficientValues("Dup").into());
                }

                stack.push(stack[stack.len() - 1])
            }
        }
    }
    Ok(())
}
