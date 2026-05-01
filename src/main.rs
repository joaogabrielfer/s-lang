use std::{collections::HashMap, error::Error, fmt::Debug, fs, io::{self, Write}, process::exit, vec};

fn main() -> Result<(), Box<dyn Error>>{
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        let mut input = String::new();
        let mut stack: Vec<RuntimeValue> = vec![];
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
        let mut stack : Vec<RuntimeValue> = vec![];
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

#[derive(Debug, Clone, PartialEq)]
enum Token{
    Push,
    Pop,
    Drop,
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Dup,
    Var,
    Into,
    Swp,
    Eq,
    Rot,
    Gt,
    Lt,
    Over,
    OpenCurly,
    CloseCurly,
    If,
    Else,
    And,
    Or,
    BoolLit(bool),
    StrLit(String),
    NumberLit(i32),
    Quit,
}

#[derive(Clone, PartialEq)]
enum RuntimeValue {
    Int(i32),
    Bool(bool),
    // String(String),
}

impl RuntimeValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            RuntimeValue::Int(_) => "int",
            RuntimeValue::Bool(_) => "bool",
            // RuntimeValue::String(_) => "str",
        }
    }
}

impl PartialOrd for RuntimeValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => a.partial_cmp(b),
            (Self::Bool(a), Self::Bool(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl std::fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            RuntimeValue::Int(n) => write!(f, "{n}"),
            // RuntimeValue::String(s) => write!(f, "\"{s}\""),
            RuntimeValue::Bool(b) => write!(f, "{b}"),
        }
    }
}

impl std::fmt::Debug for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::ops::Neg for RuntimeValue {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self{
            Self::Int(n) => Self::Int(-n),
            _ => panic!("Mismatch types while negating RuntimeValue")
        }
    }
}

impl std::ops::Div for RuntimeValue {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs){
            (Self::Int(n1), Self::Int(n2)) => Self::Int(n1 / n2),
            _ => panic!("Mismatch types while dividing RuntimeValue")
        }
    }
}

impl std::ops::Sub for RuntimeValue {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs){
            (Self::Int(n1), Self::Int(n2)) => Self::Int(n1 - n2),
            _ => panic!("Mismatch types while subtractin RuntimeValue")
        }
    }
}

impl std::ops::Mul for RuntimeValue {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs){
            (Self::Int(n1), Self::Int(n2)) => Self::Int(n1 * n2),
            _ => panic!("Mismatch types while multiplying RuntimeValue")
        }
    }
}

impl std::ops::Add for RuntimeValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs){
            (Self::Int(n1), Self::Int(n2)) => Self::Int(n1 + n2),
            _ => panic!("Mismatch types while adding RuntimeValue")
        }
    }
}

#[derive(Debug, Clone)]
enum LangError{
    StackEmpty,
    InvalidToken(Token),
    UnsufficientValues{
        op: String,
        exp: usize,
        got: usize
    },
    UnexpectedToken{
        exp: String,
        got: String,
    },
    UnexpectedTypes{
        exp: (RuntimeValue, RuntimeValue),
        got: (RuntimeValue, RuntimeValue)
    },
    UnexpectedType{
        exp: RuntimeValue,
        got: RuntimeValue,
    },
    RedeclarationVar(String),
    UndeclaredVar(String),
}

impl std::fmt::Display for LangError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidToken(tk) => write!(f, "Invalid token '{:?}'", tk),
            Self::StackEmpty => write!(f, "Could not pop variable: stack empty"),
            Self::UnsufficientValues{op, exp, got} => write!(f, "Cannot {op}: Expected {exp} value in the stack, got {got}"),
            Self::UnexpectedToken{exp, got} => write!(f, "Expected token '{exp}' got '{got}'"),
            Self::RedeclarationVar(var) => write!(f, "Trying to redeclare variable {var}"),
            Self::UndeclaredVar(var) => write!(f, "Undeclared variable {var}"),
            Self::UnexpectedTypes { exp, got } => write!(f, "Expected ({:?} and {:?}), got ({:?} and {:?})", exp.0.type_name(), exp.1.type_name(), got.0.type_name(), got.1.type_name()),
            Self::UnexpectedType { exp, got } => write!(f, "Expected {:?}, got {:?}", exp.type_name(), got.type_name()),
        }
    }
}

impl std::error::Error for LangError {}

fn tokenize(content: String) -> Vec<Token> {
    let mut tokens = Vec::new();
    let iter = content.split_whitespace();

    for word in iter {
        let token   =  match word {
            "push"  => Token::Push,
            "pop"   => Token::Pop,
            "drop"  => Token::Drop,
            "add"   => Token::Add,
            "sub"   => Token::Sub,
            "mul"   => Token::Mul,
            "div"   => Token::Div,
            "neg"   => Token::Neg,
            "dup"   => Token::Dup,
            "var"   => Token::Var,
            "into"  => Token::Into,
            "swp"   => Token::Swp,
            "rot"   => Token::Rot,
            "over"  => Token::Over,
            "eq"    => Token::Eq,
            "lt"    => Token::Lt,
            "gt"    => Token::Gt,
            "{"     => Token::OpenCurly,
            "}"     => Token::CloseCurly,
            "if"    => Token::If,
            "else"  => Token::Else,
            "and"   => Token::And,
            "or"    => Token::Or,
            "true"  => Token::BoolLit(true),
            "false" => Token::BoolLit(false),
            "quit"  => Token::Quit,
            _ => {
                let parse_result = word.parse::<i32>();
                match parse_result{
                    Ok(num) => Token::NumberLit(num),
                    Err(_) => Token::StrLit(word.to_string()),
                }
            }
        };
        tokens.push(token);
    }
    tokens
}

fn parse(tokens: Vec<Token>, stack: &mut Vec<RuntimeValue>, variables: &mut HashMap<String, i32>) -> Result<(), Box<dyn Error>>{
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
                            Some(&Token::NumberLit(_)) | Some(&Token::StrLit(_)) => { }
                            _ => break
                        }
                    }

                    let next_token = iter.next();
                    match next_token {
                        Some(Token::NumberLit(n)) => stack.push(RuntimeValue::Int(*n)),
                        Some(Token::StrLit(s)) => {
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
                    Some(p) => println!("pop: {p}"),
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
                if let Some(&Token::StrLit(s)) = iter.peek(){
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
                                Some(Token::StrLit(s)) => s.to_string(),
                                _ => unreachable!("str_next")
                            }
                        },
                        Some(Token::StrLit(s)) if variables.contains_key(s) => s.to_string(),
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
                    Some(&Token::StrLit(s)) => {
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
                    Some(&Token::StrLit(s)) => {
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
                    Some(&Token::StrLit(s)) => {
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
            Token::And => todo!(),
            Token::Or => todo!(),
        }
    }
    Ok(())
}

fn parse_var(v: Option<&Token>, variables: &mut HashMap<String, i32>) -> Result<(), Box<dyn Error>>{
    match v{
        Some(tk) => match tk{
            Token::StrLit(s) => {
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

#[cfg(feature = "token-logging")]
fn log_tokens(tokens: Vec<Token>){
    tokens
        .iter()
        .for_each(|tk| println!("{:?}", tk));
}
