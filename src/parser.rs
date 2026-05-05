use std::fs::{File, read_to_string};
use std::io::{BufRead, BufReader};
use std::iter::Peekable;
use std::process::exit;
use std::error::Error;
use std::rc::Rc;

use crate::error::{LangError, ret_error};
use crate::lexer::{Token, tokenize};
use crate::value::{RuntimeValue, PVM};

pub const STD_LIB_PATH: &str = "/home/joaogabriel/personal/programming/misc/slur/std";

pub enum Flow{
    Next,
    Return
}

impl PVM {
    pub fn parse(
        &mut self,
        tokens: Vec<Token>,
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
                            Some(Token::NumberLit(n)) => self.stack.push(RuntimeValue::Int(*n)),
                            Some(Token::QuotedLit(s)) => {
                                let trimmed = s.trim_matches('\"').to_string();
                                self.stack.push(RuntimeValue::String(Rc::new(trimmed)));
                            }
                            Some(Token::UnquotedLit(s)) => {
                                if self.elements.contains_key(s) {
                                    self.stack.push(self.elements.remove(s).unwrap_or_else(|| unreachable!("push <var>")));
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
                    if let (Some(a), Some(b)) = (self.stack.pop(),self.stack.pop()){
                        match (a.clone(), b.clone()){
                            (RuntimeValue::Int(_), RuntimeValue::Int(_)) => self.stack.push(a + b),
                            (type1, type2) => ret_error!(UnexpectedTypes,[RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![Some(type1), Some(type2)])
                        }
                    } else {
                        ret_error!(UnsufficientValues { op: "add", exp: 2_usize, got: self.stack.len() })
                    }
                }
                Token::Mul =>{
                    if let (Some(a), Some(b)) = (self.stack.pop(),self.stack.pop()){
                        match (a.clone(), b.clone()){
                            (RuntimeValue::Int(_), RuntimeValue::Int(_)) => self.stack.push(a * b),
                            (type1, type2) => ret_error!(UnexpectedTypes,[RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![Some(type1), Some(type2)])
                        }
                    } else {
                        ret_error!(UnsufficientValues { op: "mul", exp: 2_usize, got: self.stack.len() })
                    }
                }
                Token::Sub =>{
                    if let (Some(a), Some(b)) = (self.stack.pop(),self.stack.pop()){
                        match (a.clone(), b.clone()){
                            (RuntimeValue::Int(_), RuntimeValue::Int(_)) => self.stack.push(a - b),
                            (type1, type2) => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![Some(type1), Some(type2)])
                        }
                    } else {
                        ret_error!(UnsufficientValues { op: "sub", exp: 2_usize, got: self.stack.len() })
                    }
                }
                Token::Div =>{
                    if let (Some(a), Some(b)) = (self.stack.pop(),self.stack.pop()){
                        match (a.clone(), b.clone()){
                            (RuntimeValue::Int(_), RuntimeValue::Int(_)) => self.stack.push(a / b),
                            (type1, type2) => ret_error!(UnexpectedTypes,[RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![Some(type1), Some(type2)])
                        }
                    } else {
                        ret_error!(UnsufficientValues { op: "div", exp: 2_usize, got: self.stack.len() })
                    }
                }
                Token::Pop => {
                    let p = self.stack.pop();
                    match p{
                        Some(p) => {
                            match p{
                                RuntimeValue::String(s) if *s == "\\n" => println!(),
                                _ => print!("{p}"),
                            }
                        }
                        None => ret_error!(StackEmpty)
                    }
                }
                Token::Drop => {
                    if self.stack.is_empty(){
                        ret_error!(StackEmpty)
                    }
                    self.stack.pop().unwrap_or_else(|| unreachable!("drop"));
                }
                Token::Neg => {
                    if let Some(n) = self.stack.pop(){
                        match n {
                            RuntimeValue::Int(_) => self.stack.push(-n),
                            other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![Some(other)])
                        }
                    } else {
                        ret_error!(UnsufficientValues { op: "neg", exp: 1_usize, got: self.stack.len() })
                    }
                }
                Token::Dup => {
                    if let Some(&Token::UnquotedLit(s)) = iter.peek(){
                        iter.next();
                        if self.elements.contains_key(s){
                            self.stack.push(self.elements[s].clone());
                            continue;
                        } else {
                            ret_error!(UndeclaredObject { t: "variable", name: s })
                        }
                    }
                    if self.stack.is_empty() {
                        ret_error!(UnsufficientValues { op: "dup", exp: 1_usize, got: self.stack.len() })
                    }

                    self.stack.push(self.stack[self.stack.len() - 1].clone())
                }
                Token::NumberLit(num) => ret_error!(InvalidToken, Token::NumberLit(*num)),
                Token::QuotedLit(s) => ret_error!(InvalidToken, Token::QuotedLit(s.to_string())),
                Token::UnquotedLit(s) => ret_error!(InvalidToken, Token::UnquotedLit(s.to_string())),
                Token::Var => {
                    let next_token = iter.next();
                    self.parse_var(next_token)?;
                }
                Token::Into => {
                    let next_keyword = iter.next();
                    let var_name: String = {
                        match next_keyword {
                            Some(Token::Var) => {
                                let str_next = iter.next();
                                self.parse_var(str_next)?;
                                match str_next{
                                    Some(Token::UnquotedLit(s)) => s.to_string(),
                                    _ => unreachable!("str_next")
                                }
                            },
                            Some(Token::UnquotedLit(s)) if self.elements.contains_key(s) => s.to_string(),
                            Some(other) => ret_error!(UnexpectedToken, [Var, UnquotedLit], Some(other.clone())),
                            None => todo!(),
                        }
                    };
                    match self.stack.pop(){
                        Some(n) => self.elements.insert(var_name, n),
                        None => ret_error!(StackEmpty)
                    };
                }
                Token::Swap => {
                    if self.stack.len() < 2{
                        ret_error!(UnsufficientValues { op: "Swap", exp: 2_usize, got: self.stack.len() })
                    }
                    let n1 = self.stack.pop().unwrap_or_else(|| unreachable!("Swap"));
                    let n2 = self.stack.pop().unwrap_or_else(|| unreachable!("Swap"));

                    self.stack.push(n1);
                    self.stack.push(n2);
                }
                Token::Rot => self.stack.reverse(),
                Token::Over => {
                    if self.stack.len() < 2{
                        ret_error!(UnsufficientValues { op: "over", exp: 2_usize, got: self.stack.len() })
                    }

                    self.stack.push(self.stack[self.stack.len() - 2].clone());
                }
                Token::BoolLit(b) => self.stack.push(RuntimeValue::Bool(*b)),
                Token::Eq => {
                    if self.stack.len() < 2 {
                        ret_error!(UnsufficientValues { op: "eq", exp: 2_usize, got: self.stack.len() })
                    }

                    let b = self.stack.pop().unwrap_or_else(|| unreachable!("eq"));
                    let a = self.stack.pop().unwrap_or_else(|| unreachable!("eq"));

                    self.stack.push(RuntimeValue::Bool(a == b));
                }
                Token::Gt => {
                    if self.stack.len() < 2 {
                        ret_error!(UnsufficientValues { op: "gt", exp: 2_usize, got: self.stack.len() })
                    }

                    let b = self.stack.pop().unwrap_or_else(|| unreachable!("gt"));
                    let a = self.stack.pop().unwrap_or_else(|| unreachable!("gt"));

                    match (a, b) {
                        (RuntimeValue::Int(n1), RuntimeValue::Int(n2)) => {
                            self.stack.push(RuntimeValue::Bool(n1 > n2));
                        }
                        (type1, type2) => {
                            ret_error!(UnexpectedTypes,[RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![Some(type1), Some(type2)])
                        }
                    }
                }
                Token::Lt => {
                    if self.stack.len() < 2 {
                        ret_error!(UnsufficientValues { op: "lt", exp: 2_usize, got: self.stack.len() })
                    }

                    let b = self.stack.pop().unwrap_or_else(|| unreachable!("lt"));
                    let a = self.stack.pop().unwrap_or_else(|| unreachable!("lt"));

                    match (a, b) {
                        (RuntimeValue::Int(n1), RuntimeValue::Int(n2)) => {
                            self.stack.push(RuntimeValue::Bool(n1 < n2));
                        }
                        (type1, type2) => {
                            ret_error!(UnexpectedTypes, [RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![Some(type1), Some(type2)])
                        }
                    }
                }
                Token::If => {
                    if self.stack.is_empty() {
                        ret_error!(UnsufficientValues { op: "if", exp: 1_usize, got: self.stack.len() })
                    }
                    let a = self.stack.pop().unwrap_or_else(|| unreachable!("if"));

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
                            if let Flow::Return = self.parse(if_branch_vec)?{
                                return Ok(Flow::Return);
                            }
                        } else {
                            if let Flow::Return = self.parse(else_branch_vec)?{
                                return Ok(Flow::Return);
                            }
                        }
                    } else {
                        if condition && let Flow::Return = self.parse(if_branch_vec)? {
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

                            if self.stack.is_empty() {
                                ret_error!(UnsufficientValues { op: "and", exp: 1_usize, got: self.stack.len() })
                            }

                            let a = self.stack.pop().unwrap_or_else(|| unreachable!("and"));

                            match a {
                                RuntimeValue::Bool(b_self) =>self.stack.push(RuntimeValue::Bool(b_self && *b_arg)),
                                other => ret_error!(UnexpectedTypes,[RuntimeValue::Bool(true)], vec![Some(other)])
                            }
                        }
                        // TODO: add support to bool vars
                        // Some(&Token::UnquotedLit(s)) => {
                        //     iter.next();
                        //
                        //     if self.stack.is_empty() {
                        //         return Err(LangError::UnsufficientValues {
                        //             op: "and".to_string(),
                        //             exp: 1_usize,
                        //             got: self.stack.len()
                        //         }.into());
                        //     }
                        //
                        //     let a = self.stack.pop().unwrap_or_else(|| unreachable!("and"));
                        //
                        //     if self.elements.contains_key(s) {
                        //         self.stack.push(RuntimeValue::Bool(a < RuntimeValue::Int(self.elements[s])));
                        //     } else {
                        //         return Err(LangError::UndeclaredVar(s.to_string()).into());
                        //     }
                        // }
                        _ => {
                            if self.stack.len() < 2 {
                                ret_error!(UnsufficientValues { op: "and", exp: 2_usize, got: self.stack.len() })
                            }

                            let b = self.stack.pop().unwrap_or_else(|| unreachable!("and"));
                            let a = self.stack.pop().unwrap_or_else(|| unreachable!("and"));

                            match (a, b) {
                                (RuntimeValue::Bool(b1), RuntimeValue::Bool(b2)) => {
                                    self.stack.push(RuntimeValue::Bool(b1 && b2));
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

                            if self.stack.is_empty() {
                                ret_error!(UnsufficientValues { op: "or", exp: 1_usize, got: self.stack.len() })
                            }

                            let a = self.stack.pop().unwrap_or_else(|| unreachable!("or"));

                            match a {
                                RuntimeValue::Bool(b_self) =>self.stack.push(RuntimeValue::Bool(b_self || *b_arg)),
                                other => ret_error!(UnexpectedTypes, [RuntimeValue::Bool(true)], vec![Some(other)])
                            }
                        }
                        // TODO: add support to bool vars
                        // Some(&Token::UnquotedLit(s)) => {
                        //     iter.next();
                        //
                        //     if self.stack.is_empty() {
                        //         return Err(LangError::UnsufficientValues {
                        //             op: "or".to_string(),
                        //             exp: 1_usize,
                        //             got: self.stack.len()
                        //         }.into());
                        //     }
                        //
                        //     let a = self.stack.pop().unwrap_or_else(|| unreachable!("or"));
                        //
                        //     if self.elements.contains_key(s) {
                        //         self.stack.push(RuntimeValue::Bool(a < RuntimeValue::Int(self.elements[s])));
                        //     } else {
                        //         return Err(LangError::UndeclaredVar(s.to_string()).into());
                        //     }
                        // }
                        _ => {
                            if self.stack.len() < 2 {
                                ret_error!(UnsufficientValues { op: "or", exp: 2_usize, got: self.stack.len() })
                            }

                            let b = self.stack.pop().unwrap_or_else(|| unreachable!("or"));
                            let a = self.stack.pop().unwrap_or_else(|| unreachable!("or"));

                            match (a, b) {
                                (RuntimeValue::Bool(b1), RuntimeValue::Bool(b2)) => {
                                    self.stack.push(RuntimeValue::Bool(b1 || b2));
                                }
                                (type1, type2) => {
                                    ret_error!(UnexpectedTypes,[RuntimeValue::Bool(false), RuntimeValue::Bool(false)], vec![Some(type1), Some(type2)])
                                }
                            }
                        }
                    }
                }
                Token::Not => {
                    if self.stack.is_empty(){
                        ret_error!(StackEmpty)
                    }

                    match self.stack.pop().unwrap_or_else(|| unreachable!("not")){
                        RuntimeValue::Bool(b) => self.stack.push(RuntimeValue::Bool(!b)),
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::Bool(false)], vec![Some(other)])
                    }
                }
                Token::FunDeclaration(fun_name) => {
                    expect_open_curly(iter.next())?;
                    let fun_block = collect_tokens_into_block(&mut iter);
                    if self.elements.contains_key(fun_name){
                        ret_error!(RedeclarationObject { t: "function", name: fun_name })
                    }
                    self.elements.insert(fun_name.to_string(), RuntimeValue::Block(fun_block));
                }
                Token::FunCall(fun_name) => {
                    if !self.elements.contains_key(fun_name) {
                        ret_error!(UndeclaredObject { t: "function", name: fun_name })
                    }
                    match &self.elements[fun_name]{
                        RuntimeValue::Block(tokens) => {
                            let tokens_clone = tokens.clone();
                            self.parse(tokens_clone)?;
                        }
                    _ => {
                            let e = self.elements[fun_name].clone();
                            self.stack.push(e);
                        }
                    };
                }
                Token::Len => self.stack.push(RuntimeValue::Int(self.stack.len().try_into().unwrap())),
                Token::Split => {
                    if self.stack.len() < 2 {
                        ret_error!(UnsufficientValues { op: "Split", exp: 2_usize, got: self.stack.len() })
                    }

                    let pattern = self.stack.pop().unwrap_or_else(|| unreachable!("split"));
                    let source = self.stack.pop().unwrap_or_else(|| unreachable!("split"));
                    if let Some((left, right)) = source.to_string().split_once(&pattern.to_string()) {
                        self.stack.push(RuntimeValue::String(Rc::new(right.to_string())));
                        self.stack.push(RuntimeValue::String(Rc::new(left.to_string())));
                    }
                }
                Token::SplitB => {
                    if self.stack.len() < 2 {
                        ret_error!(UnsufficientValues { op: "Split", exp: 2_usize, got: self.stack.len() })
                    }
                    let pattern = self.stack.pop().unwrap_or_else(|| unreachable!("split"));
                    let source = self.stack.pop().unwrap_or_else(|| unreachable!("split"));
                    let source_str = source.to_string();
                    let result = source_str.split_once(&pattern.to_string());
                    match result{
                        Some((left, right)) =>{
                            self.stack.push(RuntimeValue::String(Rc::new(right.to_string())));
                            self.stack.push(RuntimeValue::String(Rc::new(left.to_string())));
                            self.stack.push(RuntimeValue::Bool(true))
                        }
                        None => {
                            self.stack.push(source);
                            self.stack.push(RuntimeValue::Bool(false));
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
                                    self.parse(tokens)?;
                                }
                                Err(_) => ret_error!(FileNotFound { file: s.clone(), reason: "No module with this name." })
                            }
                        }
                        other => ret_error!(UnexpectedToken,[QuotedLit, UnquotedLit], other)
                    }
                }
                Token::ReadLine => {
                    if self.stack.len() < 2{
                        ret_error!(UnsufficientValues { op: "readline", exp: 2_usize, got: self.stack.len() })
                    }

                    let line_num: usize = match self.stack.pop().unwrap_or_else(|| unreachable!("readline")){
                        RuntimeValue::Int(x) if x > 0 => x as usize,
                        RuntimeValue::Int(_) => todo!("return new error type to line idx out of range"),
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![Some(other)])
                    };
                    let path = match self.stack.pop().unwrap_or_else(|| unreachable!("readline")){
                        RuntimeValue::String(s) => s,
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::String(Rc::new("".to_string()))], vec![Some(other)])
                    };

                    let file = File::open((*path).clone())?;
                    let reader = BufReader::new(file);

                    if let Some(line) = reader.lines().nth(line_num - 1){
                        match line {
                            Ok(l) => self.stack.push(RuntimeValue::String(Rc::new(l))),
                            _ => todo!("return new error type to line idx out of range")
                        }
                    }
                }
                Token::ReadLineB => {
                    if self.stack.len() < 2{
                        self.stack.push(RuntimeValue::Bool(false))
                    }

                    let line_num: usize = match self.stack.pop().unwrap_or_else(|| unreachable!("readline")){
                        RuntimeValue::Int(x) if x > 0 => x as usize,
                        RuntimeValue::Int(_) => todo!("return new error type to line idx out of range"),
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![Some(other)])
                    };
                    let path = match self.stack.pop().unwrap_or_else(|| unreachable!("readline")){
                        RuntimeValue::String(s) => s,
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::String(Rc::new("".to_string()))], vec![Some(other)])
                    };

                    let file = File::open((*path).clone())?;
                    let reader = BufReader::new(file);

                    match reader.lines().nth(line_num - 1){
                        Some(Ok(l)) => {
                            self.stack.push(RuntimeValue::String(Rc::new(l)));
                            self.stack.push(RuntimeValue::Bool(true));
                        }
                        _ => self.stack.push(RuntimeValue::Bool(false))
                    }
                }
                Token::Int => {
                    if self.stack.is_empty(){
                        ret_error!(StackEmpty)
                    }

                    match self.stack.pop().unwrap_or_else(|| unreachable!("intb")){
                        RuntimeValue::String(s) => match (*s).parse::<i64>(){
                            Ok(n) => self.stack.push(RuntimeValue::Int(n)),
                            Err(_) => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![Some(RuntimeValue::String(Rc::new("".to_string())))]) // TODO: fix this error handling
                        }
                        RuntimeValue::Bool(b) => if b {
                            self.stack.push(RuntimeValue::Int(1));
                        } else {
                            self.stack.push(RuntimeValue::Int(0));
                        }
                        RuntimeValue::Int(i) => self.stack.push(RuntimeValue::Int(i)),
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::String(Rc::new("".to_string()))], [Some(other)])
                    };
                }
                Token::IntB => {
                    if self.stack.is_empty(){
                        ret_error!(StackEmpty)
                    }

                    match self.stack.pop().unwrap_or_else(|| unreachable!("intb")){
                        RuntimeValue::String(s) => match (*s).parse::<i64>(){
                            Ok(n) => {
                                self.stack.push(RuntimeValue::Int(n));
                                self.stack.push(RuntimeValue::Bool(true));
                            }
                            Err(_) => self.stack.push(RuntimeValue::Bool(false)),
                        }
                        RuntimeValue::Bool(b) => if b {
                            self.stack.push(RuntimeValue::Int(1));
                            self.stack.push(RuntimeValue::Bool(true));
                        } else {
                            self.stack.push(RuntimeValue::Int(0));
                            self.stack.push(RuntimeValue::Bool(false));
                        }
                        RuntimeValue::Int(i) =>{
                            self.stack.push(RuntimeValue::Int(i));
                            self.stack.push(RuntimeValue::Bool(true));
                        }
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::String(Rc::new("".to_string()))], [Some(other)])
                    };
                }
                Token::Clear => {
                    if self.stack.is_empty(){
                        ret_error!(StackEmpty)
                    }
                    self.stack.clear();
                }
                Token::Roll => {
                    if self.stack.len() < 2{
                        ret_error!(UnsufficientValues { op: "Roll", exp: 2_usize, got: self.stack.len() });
                    }

                    let index = match self.stack.pop().unwrap_or_else(||unreachable!("Roll")){
                        RuntimeValue::Int(i) => i as usize,
                        other => ret_error!(UnexpectedTypes { exp: [RuntimeValue::Int(0)], got: vec![Some(other)] })
                    };

                    if index > self.stack.len(){
                        ret_error!(UnsufficientValues { op: "Roll", exp: 0_usize, got: 0_usize }) //TODO: get a good error value
                    }

                    let result = self.stack.remove(self.stack.len() - index);
                    self.stack.push(result);
                }
                Token::Pick => {
                    if self.stack.len() < 2{
                        ret_error!(UnsufficientValues { op: "Pick", exp: 2_usize, got: self.stack.len() });
                    }

                    let index = match self.stack.pop().unwrap_or_else(||unreachable!("Pick")){
                        RuntimeValue::Int(i) => i as usize,
                        other => ret_error!(UnexpectedTypes { exp: [RuntimeValue::Int(0)], got: vec![Some(other)] })
                    };

                    if index > self.stack.len(){
                        ret_error!(UnsufficientValues { op: "Pick", exp: 0_usize, got: 0_usize }) //TODO: get a good error value
                    }

                    self.stack.push(self.stack[self.stack.len() - index].clone());
                }
            }
        }
        Ok(Flow::Next)
    }

    fn parse_var(&mut self, v: Option<&Token>) -> Result<(), Box<dyn Error>>{
        match v{
            Some(tk) => match tk{
                Token::UnquotedLit(s) => {
                    if self.elements.contains_key(s){
                        ret_error!(RedeclarationObject { t: "variable", name: s })
                    }
                    self.elements.insert(s.clone(), RuntimeValue::Int(0))
                }
                _ => ret_error!(UnexpectedToken, [UnquotedLit], Some(tk))
            }
            None => ret_error!(UnexpectedToken, [UnquotedLit], None::<Token>)
        };
        Ok(())
    }

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
