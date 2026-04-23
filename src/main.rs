use std::{error::Error, fs, io::{self, Write}, vec};

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
            }

            input.clear();
        }
    } else {
        let content = fs::read_to_string(args[1].clone())?;
        let tokens = tokenize(content);
        let mut stack : Vec<i32> = vec![];
        parse(tokens, &mut stack)?;

        if !stack.is_empty(){
            println!("warning: trailing number still in the stack: {:?}", stack);
        }
    }
    

    Ok(())
}

#[derive(Debug, Clone)]
enum Token{
    Number(i32),
    Operator(Ops),
    Sig,
    Pop,
    PS,
    Unknown
}

#[derive(Debug, Clone)]
enum Ops{
    Add,
    Subtract,
    Multiply,
    Divide,
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
            "+" => Token::Operator(Ops::Add),
            "-" => Token::Operator(Ops::Subtract),
            "*" => Token::Operator(Ops::Multiply),
            "/" => Token::Operator(Ops::Divide),
            "sig" => Token::Sig,
            "pop" => Token::Pop,
            "ps" => Token::PS,
            _ => {
                let num = x.parse::<i32>();
                match num {
                    Ok(num) => Token::Number(num),
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
            Token::Number(num) => stack.push(*num),
            Token::Operator(op) => {
                match op {
                    Ops::Add => {
                        if let (Some(a), Some(b)) = (stack.pop(), stack.pop()){
                            stack.push(a + b);
                        } else {
                            return Err(LangError::UnsufficientValues("add").into());
                        }
                    }
                    Ops::Multiply =>{
                        if let (Some(a), Some(b)) = (stack.pop(), stack.pop()){
                            stack.push(a * b);
                        } else {
                            return Err(LangError::UnsufficientValues("multiply").into());
                        }
                    }
                    Ops::Subtract =>{
                        if stack.len() != 2 {return Err(LangError::UnsufficientValues("subtract").into());}
                        let result = stack[0] - stack[1];
                        stack.clear();
                        stack.push(result);
                    }
                    Ops::Divide =>{
                        if stack.len() != 2 {return Err(LangError::UnsufficientValues("divide").into());}
                        let result = stack[0] / stack[1];
                        stack.clear();
                        stack.push(result);
                    }
                }
            }
            Token::Pop => {
                let p = stack.pop();
                match p{
                    Some(p) => println!("pop: {p}"),
                    None => return Err(LangError::StackEmpty.into())
                }
            }
            Token::PS => {
                println!("stack: {:?}", stack)
            }
            Token::Sig => {
                if let Some(n) = stack.pop(){
                    stack.push(-n);
                } else {
                    return Err(LangError::UnsufficientValues("sig").into());
                }
            }
        }
    }
    Ok(())
}
