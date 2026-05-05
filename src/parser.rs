use std::fs::{File, read_to_string};
use std::io::{BufRead, BufReader};
use std::process::exit;
use std::error::Error;
use std::rc::Rc;

use crate::error::{LangError, ret_error};
use crate::lexer::{Token, tokenize};
use crate::value::{CallFrame, PVM, RuntimeValue};

pub const STD_LIB_PATH: &str = "/home/joaogabriel/personal/programming/misc/slur/std";

pub enum Flow{
    Next,
    Return
}

impl PVM {
    pub fn parse(&mut self) -> Result<Flow, Box<dyn Error>>{
        while let Some(mut frame) = self.call_stack.pop(){
            if frame.ip >= frame.instructions.len(){
                continue;
            }

            let tk = frame.instructions[frame.ip].clone();
            frame.ip += 1;

            let mut pending_call: Option<CallFrame> = None;

            match tk {
                Token::Quit => {
                    println!("Exiting program...");
                    exit(0)
                }
                Token::Push => {
                    let mut is_first = true;

                    loop {
                        if !is_first {
                            match frame.peek() {
                                Some(&Token::NumberLit(_)) | Some(&Token::QuotedLit(_)) | Some(&Token::UnquotedLit(_)) => { }
                                _ => break
                            }
                        }

                        match frame.next() {
                            Some(Token::NumberLit(n)) => self.data_stack.push(RuntimeValue::Int(*n)),
                            Some(Token::QuotedLit(s)) => {
                                let trimmed = s.trim_matches('\"').to_string();
                                self.data_stack.push(RuntimeValue::String(Rc::new(trimmed)));
                            }
                            Some(Token::UnquotedLit(s)) => {
                                if self.elements.contains_key(s) {
                                    self.data_stack.push(self.elements.remove(s).unwrap_or_else(|| unreachable!("push <var>")));
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
                    if self.data_stack.len() < 2 {
                        ret_error!(UnsufficientValues { op: "add", exp: 2_usize, got: self.data_stack.len() })
                    }
                    let a = self.data_stack.pop().unwrap();
                    let b = self.data_stack.pop().unwrap();
                    match (a.clone(), b.clone()){
                        (RuntimeValue::Int(_), RuntimeValue::Int(_)) => self.data_stack.push(a + b),
                        (type1, type2) => ret_error!(UnexpectedTypes,[RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![type1, type2])
                    }
                }
                Token::Mul =>{
                    if self.data_stack.len() < 2 {
                        ret_error!(UnsufficientValues { op: "mul", exp: 2_usize, got: self.data_stack.len() })
                    }
                    let a = self.data_stack.pop().unwrap();
                    let b = self.data_stack.pop().unwrap();
                    match (a.clone(), b.clone()){
                        (RuntimeValue::Int(_), RuntimeValue::Int(_)) => self.data_stack.push(a * b),
                        (type1, type2) => ret_error!(UnexpectedTypes,[RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![type1, type2])
                    }
                }
                Token::Sub =>{
                    if self.data_stack.len() < 2 {
                        ret_error!(UnsufficientValues { op: "sub", exp: 2_usize, got: self.data_stack.len() })
                    }
                    let a = self.data_stack.pop().unwrap();
                    let b = self.data_stack.pop().unwrap();
                    match (a.clone(), b.clone()){
                        (RuntimeValue::Int(_), RuntimeValue::Int(_)) => self.data_stack.push(a - b),
                        (type1, type2) => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![type1, type2])
                    }
                }
                Token::Div =>{
                    if self.data_stack.len() < 2 {
                        ret_error!(UnsufficientValues { op: "div", exp: 2_usize, got: self.data_stack.len() })
                    }
                    let a = self.data_stack.pop().unwrap();
                    let b = self.data_stack.pop().unwrap();
                    match (a.clone(), b.clone()){
                        (RuntimeValue::Int(_), RuntimeValue::Int(_)) => self.data_stack.push(a / b),
                        (type1, type2) => ret_error!(UnexpectedTypes,[RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![type1, type2])
                    }
                }
                Token::Pop => {
                    let p = self.data_stack.pop();
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
                    if self.data_stack.is_empty(){
                        ret_error!(StackEmpty)
                    }
                    self.data_stack.pop().unwrap_or_else(|| unreachable!("drop"));
                }
                Token::Neg => {
                    if let Some(n) = self.data_stack.pop(){
                        match n {
                            RuntimeValue::Int(_) => self.data_stack.push(-n),
                            other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![other])
                        }
                    } else {
                        ret_error!(UnsufficientValues { op: "neg", exp: 1_usize, got: self.data_stack.len() })
                    }
                }
                Token::Dup => {
                    if let Some(Token::UnquotedLit(s)) = frame.peek().cloned(){
                        frame.next();
                        if self.elements.contains_key(&s){
                            self.data_stack.push(self.elements[&s].clone());
                            continue;
                        } else {
                            ret_error!(UndeclaredObject { t: "variable", name: s })
                        }
                    }
                    if self.data_stack.is_empty() {
                        ret_error!(UnsufficientValues { op: "dup", exp: 1_usize, got: self.data_stack.len() })
                    }

                    self.data_stack.push(self.data_stack[self.data_stack.len() - 1].clone())
                }
                Token::NumberLit(num) => ret_error!(InvalidToken, Token::NumberLit(num)),
                Token::QuotedLit(s) => ret_error!(InvalidToken, Token::QuotedLit(s.to_string())),
                Token::UnquotedLit(s) => ret_error!(InvalidToken, Token::UnquotedLit(s.to_string())),
                Token::Var => {
                    let next_token = frame.next();
                    self.parse_var(next_token)?;
                }
                Token::Into => {
                    let next_keyword = frame.next();
                    let var_name: String = {
                        match next_keyword {
                            Some(Token::Var) => {
                                let str_next = frame.next();
                                self.parse_var(str_next)?;
                                match str_next{
                                    Some(Token::UnquotedLit(s)) => s.to_string(),
                                    _ => unreachable!("str_next")
                                }
                            },
                            Some(Token::UnquotedLit(s)) if self.elements.contains_key(s) => s.to_string(),
                            other => ret_error!(UnexpectedToken, [Var, UnquotedLit], other),
                        }
                    };
                    match self.data_stack.pop(){
                        Some(n) => self.elements.insert(var_name, n),
                        None => ret_error!(StackEmpty)
                    };
                }
                Token::Swap => {
                    if self.data_stack.len() < 2{
                        ret_error!(UnsufficientValues { op: "Swap", exp: 2_usize, got: self.data_stack.len() })
                    }
                    let n1 = self.data_stack.pop().unwrap_or_else(|| unreachable!("Swap"));
                    let n2 = self.data_stack.pop().unwrap_or_else(|| unreachable!("Swap"));

                    self.data_stack.push(n1);
                    self.data_stack.push(n2);
                }
                Token::Rot => self.data_stack.reverse(),
                Token::Over => {
                    if self.data_stack.len() < 2{
                        ret_error!(UnsufficientValues { op: "over", exp: 2_usize, got: self.data_stack.len() })
                    }

                    self.data_stack.push(self.data_stack[self.data_stack.len() - 2].clone());
                }
                Token::BoolLit(b) => self.data_stack.push(RuntimeValue::Bool(b)),
                Token::Eq => {
                    if self.data_stack.len() < 2 {
                        ret_error!(UnsufficientValues { op: "eq", exp: 2_usize, got: self.data_stack.len() })
                    }

                    let b = self.data_stack.pop().unwrap_or_else(|| unreachable!("eq"));
                    let a = self.data_stack.pop().unwrap_or_else(|| unreachable!("eq"));

                    self.data_stack.push(RuntimeValue::Bool(a == b));
                }
                Token::Gt => {
                    if self.data_stack.len() < 2 {
                        ret_error!(UnsufficientValues { op: "gt", exp: 2_usize, got: self.data_stack.len() })
                    }

                    let b = self.data_stack.pop().unwrap_or_else(|| unreachable!("gt"));
                    let a = self.data_stack.pop().unwrap_or_else(|| unreachable!("gt"));

                    match (a, b) {
                        (RuntimeValue::Int(n1), RuntimeValue::Int(n2)) => {
                            self.data_stack.push(RuntimeValue::Bool(n1 > n2));
                        }
                        (type1, type2) => {
                            ret_error!(UnexpectedTypes,[RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![type1, type2])
                        }
                    }
                }
                Token::Lt => {
                    if self.data_stack.len() < 2 {
                        ret_error!(UnsufficientValues { op: "lt", exp: 2_usize, got: self.data_stack.len() })
                    }

                    let b = self.data_stack.pop().unwrap_or_else(|| unreachable!("lt"));
                    let a = self.data_stack.pop().unwrap_or_else(|| unreachable!("lt"));

                    match (a, b) {
                        (RuntimeValue::Int(n1), RuntimeValue::Int(n2)) => {
                            self.data_stack.push(RuntimeValue::Bool(n1 < n2));
                        }
                        (type1, type2) => {
                            ret_error!(UnexpectedTypes, [RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![type1, type2])
                        }
                    }
                }
                Token::If => {
                    if self.data_stack.is_empty() {
                        ret_error!(UnsufficientValues { op: "if", exp: 1_usize, got: self.data_stack.len() })
                    }
                    let a = self.data_stack.pop().unwrap_or_else(|| unreachable!("if"));

                    let condition = match a {
                        RuntimeValue::Bool(c) => c,
                        other => ret_error!(UnexpectedTypes,[RuntimeValue::Bool(true)], vec![other])
                    };

                    expect_open_curly(frame.next())?;

                    let if_branch_vec = collect_tokens_into_block(&mut frame);
                    if let Some(&Token::Else) = frame.peek() {
                        frame.next();

                        expect_open_curly(frame.next())?;

                        let else_branch_vec = collect_tokens_into_block(&mut frame);

                        if condition {
                            pending_call = Some(CallFrame {
                                instructions: if_branch_vec,
                                ip: 0,
                                frame_pointer: 0
                            })
                        } else {
                            pending_call = Some(CallFrame {
                                instructions: else_branch_vec,
                                ip: 0,
                                frame_pointer: 0
                            })
                        }
                    } else {
                        pending_call = Some(CallFrame {
                            instructions: if_branch_vec,
                            ip: 0,
                            frame_pointer: 0
                        })
                    }
                }
                Token::Else => ret_error!(InvalidToken, Token::Else),
                Token::OpenCurly => { },
                Token::CloseCurly => { },
                Token::And => {
                    if self.data_stack.len() < 2 {
                        ret_error!(UnsufficientValues { op: "and", exp: 2_usize, got: self.data_stack.len() })
                    }

                    let b = self.data_stack.pop().unwrap_or_else(|| unreachable!("and"));
                    let a = self.data_stack.pop().unwrap_or_else(|| unreachable!("and"));

                    match (a, b) {
                        (RuntimeValue::Bool(b1), RuntimeValue::Bool(b2)) => {
                            self.data_stack.push(RuntimeValue::Bool(b1 && b2));
                        }
                        (type1, type2) => ret_error!(UnexpectedTypes, [RuntimeValue::Bool(false), RuntimeValue::Bool(false)], vec![type1, type2])
                    }
                }
                Token::Or => {
                    if self.data_stack.len() < 2 {
                        ret_error!(UnsufficientValues { op: "or", exp: 2_usize, got: self.data_stack.len() })
                    }

                    let b = self.data_stack.pop().unwrap_or_else(|| unreachable!("or"));
                    let a = self.data_stack.pop().unwrap_or_else(|| unreachable!("or"));

                    match (a, b) {
                        (RuntimeValue::Bool(b1), RuntimeValue::Bool(b2)) => {
                            self.data_stack.push(RuntimeValue::Bool(b1 || b2));
                        }
                        (type1, type2) => ret_error!(UnexpectedTypes,[RuntimeValue::Bool(false), RuntimeValue::Bool(false)], vec![type1, type2])
                    }
                }
                Token::Not => {
                    if self.data_stack.is_empty(){
                        ret_error!(StackEmpty)
                    }

                    match self.data_stack.pop().unwrap_or_else(|| unreachable!("not")){
                        RuntimeValue::Bool(b) => self.data_stack.push(RuntimeValue::Bool(!b)),
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::Bool(false)], vec![other])
                    }
                }
                Token::FunDeclaration(fun_name) => {
                    expect_open_curly(frame.next())?;
                    let fun_block = collect_tokens_into_block(&mut frame);
                    if self.elements.contains_key(&fun_name){
                        ret_error!(RedeclarationObject { t: "function", name: fun_name })
                    }
                    self.elements.insert(fun_name.to_string(), RuntimeValue::Block(fun_block));
                }
                Token::FunCall(fun_name) => {
                    if !self.elements.contains_key(&fun_name) {
                        ret_error!(UndeclaredObject { t: "function", name: fun_name })
                    }
                    match &self.elements[&fun_name]{
                        RuntimeValue::Block(tokens) => {
                            pending_call = Some(CallFrame {
                                instructions: tokens.clone(),
                                ip: 0,
                                frame_pointer: 0
                            })
                        }
                        _ => {
                            let e = self.elements[&fun_name].clone();
                            self.data_stack.push(e);
                        }
                    };
                }
                Token::Len => self.data_stack.push(RuntimeValue::Int(self.data_stack.len().try_into().unwrap())),
                Token::Split => {
                    if self.data_stack.len() < 2 {
                        ret_error!(UnsufficientValues { op: "Split", exp: 2_usize, got: self.data_stack.len() })
                    }

                    let pattern = self.data_stack.pop().unwrap_or_else(|| unreachable!("split"));
                    let source = self.data_stack.pop().unwrap_or_else(|| unreachable!("split"));
                    if let Some((left, right)) = source.to_string().split_once(&pattern.to_string()) {
                        self.data_stack.push(RuntimeValue::String(Rc::new(right.to_string())));
                        self.data_stack.push(RuntimeValue::String(Rc::new(left.to_string())));
                    }
                }
                Token::SplitB => {
                    if self.data_stack.len() < 2 {
                        ret_error!(UnsufficientValues { op: "Split", exp: 2_usize, got: self.data_stack.len() })
                    }
                    let pattern = self.data_stack.pop().unwrap_or_else(|| unreachable!("split"));
                    let source = self.data_stack.pop().unwrap_or_else(|| unreachable!("split"));
                    let source_str = source.to_string();
                    let result = source_str.split_once(&pattern.to_string());
                    match result{
                        Some((left, right)) =>{
                            self.data_stack.push(RuntimeValue::String(Rc::new(right.to_string())));
                            self.data_stack.push(RuntimeValue::String(Rc::new(left.to_string())));
                            self.data_stack.push(RuntimeValue::Bool(true))
                        }
                        None => {
                            self.data_stack.push(source);
                            self.data_stack.push(RuntimeValue::Bool(false));
                        }
                    }
                }
                Token::Ret => return Ok(Flow::Return),
                Token::Include => {
                    match frame.next(){
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
                                    pending_call = Some(CallFrame{
                                        instructions: tokens,
                                        ip: 0,
                                        frame_pointer: self.data_stack.len()
                                    })
                                }
                                Err(_) => ret_error!(FileNotFound { file: s.clone(), reason: "No module with this name." })
                            }
                        }
                        other => ret_error!(UnexpectedToken,[QuotedLit, UnquotedLit], other)
                    }
                }
                Token::ReadLine => {
                    if self.data_stack.len() < 2{
                        ret_error!(UnsufficientValues { op: "readline", exp: 2_usize, got: self.data_stack.len() })
                    }

                    let line_num: usize = match self.data_stack.pop().unwrap_or_else(|| unreachable!("readline")){
                        RuntimeValue::Int(x) if x > 0 => x as usize,
                        RuntimeValue::Int(x) => ret_error!(IndexOutOfRange, x),
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![other])
                    };
                    let path = match self.data_stack.pop().unwrap_or_else(|| unreachable!("readline")){
                        RuntimeValue::String(s) => s,
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::String(Rc::new("".to_string()))], vec![other])
                    };

                    let file = File::open((*path).clone())?;
                    let reader = BufReader::new(file);

                    if let Some(line) = reader.lines().nth(line_num - 1){
                        match line {
                            Ok(l) => self.data_stack.push(RuntimeValue::String(Rc::new(l))),
                            _ => ret_error!(LineOutOfRange)
                        }
                    } else {
                        ret_error!(LineOutOfRange)
                    }
                }
                Token::ReadLineB => {
                    if self.data_stack.len() < 2{
                        self.data_stack.push(RuntimeValue::Bool(false));
                        return Ok(Flow::Next);
                    }

                    let line_num: usize = match self.data_stack.pop().unwrap_or_else(|| unreachable!("readline")){
                        RuntimeValue::Int(x) if x > 0 => x as usize,
                        RuntimeValue::Int(x) => ret_error!(IndexOutOfRange, x),
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![other])
                    };
                    let path = match self.data_stack.pop().unwrap_or_else(|| unreachable!("readline")){
                        RuntimeValue::String(s) => s,
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::String(Rc::new("".to_string()))], vec![other])
                    };

                    let file = File::open((*path).clone())?;
                    let reader = BufReader::new(file);

                    match reader.lines().nth(line_num - 1){
                        Some(Ok(l)) => {
                            self.data_stack.push(RuntimeValue::String(Rc::new(l)));
                            self.data_stack.push(RuntimeValue::Bool(true));
                        }
                        _ => self.data_stack.push(RuntimeValue::Bool(false))
                    }
                }
                Token::Int => {
                    if self.data_stack.is_empty(){
                        ret_error!(StackEmpty)
                    }

                    match self.data_stack.pop().unwrap_or_else(|| unreachable!("intb")){
                        RuntimeValue::String(s) => match (*s).parse::<i64>(){
                            Ok(n) => self.data_stack.push(RuntimeValue::Int(n)),
                            Err(_) => ret_error!(ParseError, (*s).clone())
                        }
                        RuntimeValue::Bool(b) => if b {
                            self.data_stack.push(RuntimeValue::Int(1));
                        } else {
                            self.data_stack.push(RuntimeValue::Int(0));
                        }
                        RuntimeValue::Int(i) => self.data_stack.push(RuntimeValue::Int(i)),
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0), RuntimeValue::Bool(false), RuntimeValue::String(Rc::new("".to_string()))], vec![other])
                    };
                }
                Token::IntB => {
                    if self.data_stack.is_empty(){
                        ret_error!(StackEmpty)
                    }

                    match self.data_stack.pop().unwrap_or_else(|| unreachable!("intb")){
                        RuntimeValue::String(s) => match (*s).parse::<i64>(){
                            Ok(n) => {
                                self.data_stack.push(RuntimeValue::Int(n));
                                self.data_stack.push(RuntimeValue::Bool(true));
                            }
                            Err(_) => self.data_stack.push(RuntimeValue::Bool(false)),
                        }
                        RuntimeValue::Bool(b) => if b {
                            self.data_stack.push(RuntimeValue::Int(1));
                            self.data_stack.push(RuntimeValue::Bool(true));
                        } else {
                            self.data_stack.push(RuntimeValue::Int(0));
                            self.data_stack.push(RuntimeValue::Bool(false));
                        }
                        RuntimeValue::Int(i) =>{
                            self.data_stack.push(RuntimeValue::Int(i));
                            self.data_stack.push(RuntimeValue::Bool(true));
                        }
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0), RuntimeValue::Bool(false), RuntimeValue::String(Rc::new("".to_string()))], vec![other])
                    };
                }
                Token::Clear => {
                    if self.data_stack.is_empty(){
                        ret_error!(StackEmpty)
                    }
                    self.data_stack.clear();
                }
                Token::Roll => {
                    if self.data_stack.len() < 2{
                        ret_error!(UnsufficientValues { op: "Roll", exp: 2_usize, got: self.data_stack.len() });
                    }

                    let index = match self.data_stack.pop().unwrap_or_else(||unreachable!("Roll")){
                        RuntimeValue::Int(i) => i as usize,
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![other])
                    };

                    if index > self.data_stack.len(){
                        ret_error!(StackIndexOutOfRange { op: "Roll".to_string(), index: index, stack_len: self.data_stack.len() })
                    }

                    let result = self.data_stack.remove(self.data_stack.len() - index);
                    self.data_stack.push(result);
                }
                Token::Pick => {
                    if self.data_stack.len() < 2{
                        ret_error!(UnsufficientValues { op: "Pick", exp: 2_usize, got: self.data_stack.len() });
                    }

                    let index = match self.data_stack.pop().unwrap_or_else(||unreachable!("Pick")){
                        RuntimeValue::Int(i) => i as usize,
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0)], vec![other])
                    };

                    if index > self.data_stack.len(){
                        ret_error!(StackIndexOutOfRange { op: "Pick".to_string(), index: index, stack_len: self.data_stack.len() })
                    }

                    self.data_stack.push(self.data_stack[self.data_stack.len() - index].clone());
                }
            }
            self.call_stack.push(frame);

            if let Some(new_frame) = pending_call {
                self.call_stack.push(new_frame);
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

fn collect_tokens_into_block(frame: &mut CallFrame) -> Vec<Token>{
    let mut block_vector: Vec<Token> = vec![];
    let mut brace_depth = 1;

    while frame.ip < frame.instructions.len() {
        let tk = frame.instructions[frame.ip].clone();
        frame.ip += 1;
        match tk {
            Token::OpenCurly => {
                brace_depth += 1;
                block_vector.push(tk.clone());
            }
            Token::CloseCurly => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    break;
                } else {
                    block_vector.push(tk.clone());
                }
            }
            _ => {
                block_vector.push(tk.clone());
            }
        }
    };
    block_vector
}
