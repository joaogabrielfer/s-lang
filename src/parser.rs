use std::fs::{File, read_to_string};
use std::io::{BufRead, BufReader};
use std::process::exit;
use std::error::Error;
use std::rc::Rc;

use crate::error::{LangError, ret_error};
use crate::lexer::{Token, tokenize};
use crate::value::{self, CallFrame, Element, PVM, RuntimeValue, RuntimeValueT, default_runtime_function};

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
                                    match self.elements[s].clone(){
                                        value::Element::Var(runtime_value) => self.data_stack.push(runtime_value),
                                        value::Element::Function { arguments_t: args_types, block, return_t: return_types } => {
                                            self.data_stack.push(RuntimeValue::Function {
                                                arguments_t: args_types,
                                                return_t: return_types,
                                                block
                                            })
                                        }
                                    }
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
                    if self.data_stack.len() - frame.frame_pointer < 2{
                        ret_error!(StackUnderflow)
                    }
                    let a = self.data_stack.pop().unwrap();
                    let b = self.data_stack.pop().unwrap();
                    match (a.clone(), b.clone()){
                        (RuntimeValue::Int(_), RuntimeValue::Int(_)) => self.data_stack.push(a + b),
                        (type1, type2) => ret_error!(UnexpectedTypes,[RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![type1, type2])
                    }
                }
                Token::Mul =>{
                    if self.data_stack.len() - frame.frame_pointer < 2{
                        ret_error!(StackUnderflow)
                    }
                    let a = self.data_stack.pop().unwrap();
                    let b = self.data_stack.pop().unwrap();
                    match (a.clone(), b.clone()){
                        (RuntimeValue::Int(_), RuntimeValue::Int(_)) => self.data_stack.push(a * b),
                        (type1, type2) => ret_error!(UnexpectedTypes,[RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![type1, type2])
                    }
                }
                Token::Sub =>{
                    if self.data_stack.len() - frame.frame_pointer < 2{
                        ret_error!(StackUnderflow)
                    }
                    let a = self.data_stack.pop().unwrap();
                    let b = self.data_stack.pop().unwrap();
                    match (a.clone(), b.clone()){
                        (RuntimeValue::Int(_), RuntimeValue::Int(_)) => self.data_stack.push(a - b),
                        (type1, type2) => ret_error!(UnexpectedTypes, [RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![type1, type2])
                    }
                }
                Token::Div =>{
                    if self.data_stack.len() - frame.frame_pointer < 2{
                        ret_error!(StackUnderflow)
                    }
                    let a = self.data_stack.pop().unwrap();
                    let b = self.data_stack.pop().unwrap();
                    match (a.clone(), b.clone()){
                        (RuntimeValue::Int(_), RuntimeValue::Int(_)) => self.data_stack.push(a / b),
                        (type1, type2) => ret_error!(UnexpectedTypes,[RuntimeValue::Int(0), RuntimeValue::Int(0)], vec![type1, type2])
                    }
                }
                Token::Pop => {
                    if self.data_stack.len() <= frame.frame_pointer{
                        ret_error!(StackUnderflow)
                    }
                    let p = self.data_stack.pop().unwrap_or_else(|| unreachable!("pop"));
                    match p {
                        RuntimeValue::String(s) if *s == "\\n" => println!(),
                        _ => print!("{p}"),
                    }
                }
                Token::Drop => {
                    if self.data_stack.len() <= frame.frame_pointer{
                        ret_error!(StackUnderflow)
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
                    if self.data_stack.is_empty() {
                        ret_error!(UnsufficientValues { op: "dup", exp: 1_usize, got: self.data_stack.len() })
                    }

                    self.data_stack.push(self.data_stack[self.data_stack.len() - 1].clone())
                }
                Token::NumberLit(num) => ret_error!(InvalidToken, Token::NumberLit(num)),
                Token::QuotedLit(s) => ret_error!(InvalidToken, Token::QuotedLit(s.to_string())),
                Token::UnquotedLit(s) => ret_error!(InvalidToken, Token::UnquotedLit(s.to_string())),
                Token::Into => {
                    let var_name_opt = frame.next();
                    let var_name: String = match var_name_opt {
                        Some(Token::UnquotedLit(s)) => s.to_string(),
                        other => ret_error!(UnexpectedToken, [UnquotedLit], other),
                    };

                    match self.data_stack.len() - frame.frame_pointer {
                        0..1 => ret_error!(StackUnderflow),
                        1.. => {
                            let var = self.data_stack.pop().unwrap_or_else(|| unreachable!("into"));
                            match var {
                                RuntimeValue::Function { arguments_t, return_t, block  } => self.elements.insert(var_name, Element::Function{arguments_t, return_t, block}),
                                _ => self.elements.insert(var_name, Element::Var(var)),
                            };
                        }
                        // 2.. => {
                        //     println!("as fn");
                        //     let block = self.data_stack.pop().unwrap_or_else(|| unreachable!("into"));
                        //
                        //     let args = if self.data_stack.len() - frame.frame_pointer >= 1 {
                        //         match self.data_stack.pop().unwrap_or_else(|| unreachable!("into")){
                        //             RuntimeValue::Function{ arguments_t, ..} => Some(arguments_t),
                        //             v => {
                        //                 self.data_stack.push(v);
                        //                 None
                        //             }
                        //         }
                        //     } else {
                        //         ret_error!(StackUnderflow)
                        //     };
                        //     match block{
                        //         RuntimeValue::Block(b) if args.is_some()=> {
                        //             self.elements.insert(var_name, value::Element::Function {
                        //                 arguments_t: args.unwrap_or_else(|| unreachable!("into block")),
                        //                 return_t: vec![],
                        //                 block: b
                        //             });
                        //         }
                        //         other => {
                        //             self.elements.insert(var_name, value::Element::Var(other));
                        //         }
                        //     }
                        // }
                    }
                }
                Token::Swap => {
                    if self.data_stack.len() - frame.frame_pointer < 2{
                        ret_error!(StackUnderflow)
                    }
                    let n1 = self.data_stack.pop().unwrap_or_else(|| unreachable!("Swap"));
                    let n2 = self.data_stack.pop().unwrap_or_else(|| unreachable!("Swap"));

                    self.data_stack.push(n1);
                    self.data_stack.push(n2);
                }
                Token::Rot => {
                    if self.data_stack.len() - frame.frame_pointer < 3 {
                        ret_error!(StackUnderflow)
                    }

                    let a = self.data_stack.remove(self.data_stack.len() - 3);

                    self.data_stack.push(a);
                }
                Token::Over => {
                    if self.data_stack.len() - frame.frame_pointer < 2{
                        ret_error!(StackUnderflow)
                    }

                    self.data_stack.push(self.data_stack[self.data_stack.len() - 2].clone());
                }
                Token::BoolLit(b) => self.data_stack.push(RuntimeValue::Bool(b)),
                Token::Eq => {
                    if self.data_stack.len() - frame.frame_pointer < 2{
                        ret_error!(StackUnderflow)
                    }

                    let b = self.data_stack.pop().unwrap_or_else(|| unreachable!("eq"));
                    let a = self.data_stack.pop().unwrap_or_else(|| unreachable!("eq"));

                    self.data_stack.push(RuntimeValue::Bool(a == b));
                }
                Token::Gt => {
                    if self.data_stack.len() - frame.frame_pointer < 2{
                        ret_error!(StackUnderflow)
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
                    if self.data_stack.len() - frame.frame_pointer < 2{
                        ret_error!(StackUnderflow)
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

                    let if_branch_vec = collect_tokens_into_block(&mut frame)?;
                    if let Some(&Token::Else) = frame.peek() {
                        frame.next();

                        expect_open_curly(frame.next())?;

                        let else_branch_vec = collect_tokens_into_block(&mut frame)?;

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
                Token::OpenCurly => {
                    let block = collect_tokens_into_block(&mut frame)?;
                    match self.data_stack.pop() {
                        Some(RuntimeValue::Function { arguments_t, .. }) => {
                            self.data_stack.push(RuntimeValue::Function {
                                arguments_t,
                                return_t: vec![],
                                block
                            });
                        }
                        Some(other) => {
                            self.data_stack.push(other);
                            self.data_stack.push(RuntimeValue::Block(block));
                        }
                        None => {
                            self.data_stack.push(RuntimeValue::Block(block));
                        }
                    }
                }
                Token::CloseCurly => { },
                Token::And => {
                    if self.data_stack.len() - frame.frame_pointer < 2{
                        ret_error!(StackUnderflow)
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
                    if self.data_stack.len() - frame.frame_pointer < 2{
                        ret_error!(StackUnderflow)
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
                    if self.data_stack.len() <= frame.frame_pointer{
                        ret_error!(StackUnderflow)
                    }

                    match self.data_stack.pop().unwrap_or_else(|| unreachable!("not")){
                        RuntimeValue::Bool(b) => self.data_stack.push(RuntimeValue::Bool(!b)),
                        other => ret_error!(UnexpectedTypes, [RuntimeValue::Bool(false)], vec![other])
                    }
                }
                Token::ElementCall(element_name) => {
                    if element_name.is_empty(){
                        ret_error!(UnexpectedToken, [UnquotedLit], None::<Token>)
                    }
                    if !self.elements.contains_key(&element_name) {
                        ret_error!(UndeclaredObject { t: "function", name: element_name })
                    }

                    match self.elements[element_name.as_str()].clone() {
                        Element::Var(v) => self.data_stack.push(v),
                        Element::Function { arguments_t: args_types, block, ..} => {
                            let fp = self.resolve_call_frame(&args_types)?;

                            pending_call = Some(CallFrame {
                                instructions: block,
                                ip: 0,
                                frame_pointer: fp
                            })
                        }
                    }
                }
                Token::Len => self.data_stack.push(RuntimeValue::Int(self.data_stack.len().try_into().unwrap())),
                Token::SplitB => {
                    if self.data_stack.len() - frame.frame_pointer < 2{
                        ret_error!(StackUnderflow)
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
                    if self.data_stack.len() - frame.frame_pointer < 2{
                        ret_error!(StackUnderflow)
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
                    if self.data_stack.len() - frame.frame_pointer < 2{
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
                Token::AsIntB => {
                    if self.data_stack.len() <= frame.frame_pointer{
                        ret_error!(StackUnderflow)
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
                    if self.data_stack.len() <= frame.frame_pointer{
                        ret_error!(StackUnderflow)
                    }
                    self.data_stack.clear();
                }
                Token::Roll => {
                    if self.data_stack.len() - frame.frame_pointer < 2{
                        ret_error!(StackUnderflow)
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
                    if self.data_stack.len() - frame.frame_pointer < 2{
                        ret_error!(StackUnderflow)
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
                Token::OpenParen => {
                    let types = collect_tokens_into_types(&mut frame)?;
                    self.data_stack.push(RuntimeValue::Function{
                        arguments_t: types,
                        return_t: vec![],
                        block: vec![],
                    });
                }
                Token::CloseParen => ret_error!(InvalidToken, Token::CloseParen),
                Token::Eval => {
                    if self.data_stack.len() - frame.frame_pointer < 1{
                        ret_error!(StackUnderflow)
                    }

                    match self.data_stack.pop().unwrap_or_else(||unreachable!("eval")){
                        RuntimeValue::Function { arguments_t, block, .. } => {

                            let fp = self.resolve_call_frame(&arguments_t)?;

                            pending_call = Some(CallFrame {
                                instructions: block,
                                ip: 0,
                                frame_pointer: fp
                            });
                        }
                        other => ret_error!(UnexpectedTypes, [default_runtime_function()], vec![other] ),
                    };
                }
            }
            self.call_stack.push(frame);

            if let Some(new_frame) = pending_call {
                self.call_stack.push(new_frame);
            }

        }
        Ok(Flow::Next)
    }

    fn resolve_call_frame(&self, arguments_t: &[RuntimeValueT]) -> Result<usize, Box<dyn Error>> {
        let variadic_pos = arguments_t.iter().position(|t| matches!(t, RuntimeValueT::Variadic(_)));
        let fp;

        if let Some(v_idx) = variadic_pos {
            let fixed_after = arguments_t.len() - 1 - v_idx;
            let fixed_before = v_idx;

            if self.data_stack.len() < fixed_before + fixed_after {
                ret_error!(UnsufficientValues { op: "Call", exp: fixed_before + fixed_after, got: self.data_stack.len() })
            }

            let end_v = self.data_stack.len() - fixed_after;
            let mut start_v = end_v;

            let inner_type = if let RuntimeValueT::Variadic(t) = &arguments_t[v_idx] {
                *t.clone()
            } else {
                unreachable!()
            };

            while start_v > fixed_before {
                if self.data_stack[start_v - 1].compare_type(inner_type.clone()) {
                    start_v -= 1;
                } else {
                    break;
                }
            }

            fp = start_v - fixed_before;
        } else {
            if self.data_stack.len() < arguments_t.len() {
                ret_error!(UnsufficientValues { op: "Call", exp: arguments_t.len(), got: self.data_stack.len() })
            }
            fp = self.data_stack.len() - arguments_t.len();
        }

        for i in fp..self.data_stack.len() {
            let k = i - fp;
            let expected_type = match variadic_pos {
                Some(v_idx) => {
                    let variadic_count = self.data_stack.len() - fp - (arguments_t.len() - 1);
                    if k < v_idx {
                        arguments_t[k].clone()
                    } else if k >= v_idx && k < v_idx + variadic_count {
                        if let RuntimeValueT::Variadic(inner_box) = &arguments_t[v_idx] {
                            *inner_box.clone()
                        } else {
                            unreachable!()
                        }
                    } else {
                        arguments_t[k - variadic_count + 1].clone()
                    }
                }
                None => arguments_t[k].clone()
            };

            if !self.data_stack[i].compare_type(expected_type.clone()) {
                ret_error!(UnexpectedTypes, [expected_type.to_runtimevalue()], vec![self.data_stack[i].clone()])
            }
        }

        Ok(fp)
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

fn collect_tokens_into_block(frame: &mut CallFrame) -> Result<Vec<Token>, Box<dyn Error>>{
    let mut block_vector: Vec<Token> = vec![];
    let mut brace_depth = 1;

    while frame.ip < frame.instructions.len() {
        let tk = frame.instructions[frame.ip].clone();
        frame.ip += 1;
        match tk {
            Token::OpenCurly => {
                brace_depth += 1;
                block_vector.push(tk);
            }
            Token::CloseCurly => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    break;
                } else {
                    block_vector.push(tk);
                }
            }
            _ => {
                block_vector.push(tk);
            }
        }
    };
    if brace_depth != 0 {
        // Replace this with however your VM handles syntax errors (e.g., ret_error!)
        return Err("Syntax Error: Reached end of file but missing a closing '}'".to_string().into());
    }
    Ok(block_vector)
}

fn collect_tokens_into_types(frame: &mut CallFrame) -> Result<Vec<RuntimeValueT>, Box<dyn Error>>{
    let mut types_vec: Vec<RuntimeValueT> = vec![];

    while frame.ip < frame.instructions.len() {
        let tk = frame.instructions[frame.ip].clone();
        frame.ip += 1;
        match tk {
            Token::CloseParen => {
                break;
            }
            Token::UnquotedLit(t) => {
                let word = t.trim();
                if word.is_empty() { continue; }

                let is_variadic = word.starts_with("..");

                let base_word = if is_variadic {
                    if word == ".." { "any" } else { &word[2..] }
                } else {
                    word
                };

                let base_type = match base_word {
                    "string"   => RuntimeValueT::String,
                    "int"      => RuntimeValueT::Int,
                    "bool"     => RuntimeValueT::Bool,
                    "function" => RuntimeValueT::Function,
                    "any"      => RuntimeValueT::Any,
                    _ => ret_error!(UnknownType, base_word.to_string())
                };

                if is_variadic {
                    types_vec.push(RuntimeValueT::Variadic(Box::new(base_type)));
                } else {
                    types_vec.push(base_type);
                }
            }
            other => ret_error!(UnexpectedToken, [UnquotedLit], Some(other))
        }
    };
    Ok(types_vec)
}
