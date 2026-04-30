use std::{collections::HashMap, error::Error, fmt::Debug, fs, io::{self, Write}, process::exit, vec};

fn main() -> Result<(), Box<dyn Error>>{
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        let mut input = String::new();
        let mut stack: Vec<i32> = vec![];
        let mut variables: HashMap<String, i32> = HashMap::new();
        loop{
            print!("> ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut input)?;

            let tokens= tokenize(input.clone());
            if cfg!(feature = "token-logging"){
                    #[cfg(feature = "token-logging")]
                    log_tokens(tokens.clone());
                    exit(0);
            }

            if let Err(e) = parse(tokens, &mut stack, &mut variables){
                eprintln!("ERROR: {e}")
            } else {
                println!("stack: {:?}", stack);
            }

            input.clear();
        }
    } else {
        let content = fs::read_to_string(args[1].clone())?;
        let mut stack : Vec<i32> = vec![];
        let mut variables: HashMap<String, i32> = HashMap::new();
        let tokens = tokenize(content.clone());
        if cfg!(feature = "token-logging"){
            #[cfg(feature = "token-logging")]
            log_tokens(tokens.clone());
            exit(0);
        }

        if let Err(e) = parse(tokens, &mut stack, &mut variables){
            eprintln!("ERROR: {e}");
            exit(0);
        }


        if !stack.is_empty(){
            println!("warning: trailing number still in the stack: {:?}", stack);
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum Token{
    Number(i32),
    Push,
    Add,
    Sub,
    Mul,
    Div,
    Sig,
    Pop,
    Dup,
    Var,
    Str(String),
}

#[derive(Debug, Clone)]
enum LangError{
    StackEmpty,
    InvalidToken(Token),
    UnsufficientValues(&'static str),
    UnexpectedToken{
        exp: String,
        unexp: String,
    },
    RedeclarationVar(String),
    UndeclaredVar(String),
}

impl std::fmt::Display for LangError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidToken(tk) => write!(f, "Invalid token '{:?}'", tk),
            Self::StackEmpty => write!(f, "Could not pop variable: stack empty"),
            Self::UnsufficientValues(operation) => write!(f, "Cannot {operation}: too much or too many variables in the stack"),
            Self::UnexpectedToken{exp, unexp} => write!(f, "Expected token '{exp}' got '{unexp}'"),
            Self::RedeclarationVar(var) => write!(f, "Trying to redeclare variable {var}"),
            Self::UndeclaredVar(var) => write!(f, "Undeclared variable {var}"),
        }
    }
}

impl std::error::Error for LangError {}

fn tokenize(content: String) -> Vec<Token> {
    let mut tokens = Vec::new();
    let iter = content.split_whitespace();

    for word in iter {
        let token = match word {
            "push" => Token::Push,
            "add"  => Token::Add,
            "sub"  => Token::Sub,
            "mul"  => Token::Mul,
            "div"  => Token::Div,
            "sig"  => Token::Sig,
            "pop"  => Token::Pop,
            "dup"  => Token::Dup,
            "var"  => Token::Var,
            "quit" => {
                println!("Exiting program...");
                exit(0);
            },
            _ => {
                let parse_result = word.parse::<i32>();
                match parse_result{
                    Ok(num) => Token::Number(num),
                    Err(_) => Token::Str(word.to_string()),
                }
            }
        };
        tokens.push(token);
    }
    tokens
}

fn parse(tokens: Vec<Token>, stack: &mut Vec<i32>, variables: &mut HashMap<String, i32>) -> Result<(), Box<dyn Error>>{
    let mut iter = tokens.iter().peekable();
    while let Some(tk) = iter.next() {
        match tk {
            Token::Push => {
                let next_token = iter.next();
                match next_token{
                    Some(tk) => match tk{
                        Token::Number(n) => {
                            stack.push(*n);
                        }
                        _ => return Err(LangError::UnexpectedToken {
                        exp: "Number".to_string(),
                        unexp: "None".to_string(),
                    }.into())                    }
                    None => return Err(LangError::UnexpectedToken {
                        exp: "Number".to_string(),
                        unexp: format!("{:?}", tk)
                    }.into())
                };
            }
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
            Token::Sig => {
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
            Token::Number(num) => return Err(LangError::InvalidToken(Token::Number(*num)).into()),
            Token::Var => {
                let next_token = iter.next();
                match next_token{
                    Some(tk) => match tk{
                        Token::Str(s) => {
                            if variables.contains_key(s){
                                return Err(LangError::RedeclarationVar(s.clone()).into());
                            }
                            variables.insert(s.clone(), 0)
                        }
                        _ => return Err(LangError::UnexpectedToken {
                        exp: "Str".to_string(),
                        unexp: "None".to_string(),
                    }.into())                    }
                    None => return Err(LangError::UnexpectedToken {
                        exp: "Str".to_string(),
                        unexp: format!("{:?}", tk)
                    }.into())
                };
            }
            Token::Str(s) => {
                if variables.contains_key(s){
                    stack.push(variables[s]);
                } else {
                    return Err(LangError::UndeclaredVar(s.to_string()).into())
                }
            }
        }
    }
    Ok(())
}

#[cfg(feature = "token-logging")]
fn log_tokens(tokens: Vec<Token>){
    tokens
        .iter()
        .for_each(|tk| println!("{:?}", tk));
}
