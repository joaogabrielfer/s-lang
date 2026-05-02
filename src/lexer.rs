use std::process::exit;

#[derive(Debug, Clone, PartialEq)]
pub enum Token{
    Push, Pop, Drop,
    Add, Sub, Mul, Div, Neg, Dup,
    Len, Split, SplitB,
    Var, Into,
    Swap, Rot, Over,
    Eq, Gt, Lt,
    OpenCurly, CloseCurly,
    If, Else,
    And, Or, Not,
    FunDeclaration(String), FunCall(String),
    BoolLit(bool), StrLit(String), VarLit(String), NumberLit(i32),
    Quit, Ret,
}

pub fn tokenize(content: String) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = content.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            _ if c.is_whitespace() => {
                chars.next();
            }
            '{' => {
                tokens.push(Token::OpenCurly);
                chars.next();
            }
            '}' => {
                tokens.push(Token::CloseCurly);
                chars.next();
            }
            ';' => {
                chars.next();
                if chars.peek() == Some(&';') {
                    chars.next();
                    let mut last_was_semi = false;
                    for cc in &mut chars {
                        if cc == ';' {
                            if last_was_semi {
                                break;
                            }
                            last_was_semi = true;
                        } else {
                            last_was_semi = false;
                        }
                    }
                }
            }
            '"' => {
                chars.next();
                let mut string_val = String::new();
                for cc in &mut chars {
                    if cc == '"' {
                        break;
                    }
                    string_val.push(cc);
                }
                tokens.push(Token::StrLit(string_val));
            }
            _ => {
                let mut word = String::new();

                while let Some(&nc) = chars.peek() {
                    if nc.is_whitespace() || nc == '{' || nc == '}' || nc == '"' || nc == ';' {
                        break;
                    }
                    word.push(nc);
                    chars.next();
                }

                let token = match word.as_str() {
                    "push"   => Token::Push,
                    "pop"    => Token::Pop,
                    "drop"   => Token::Drop,
                    "add"    => Token::Add,
                    "sub"    => Token::Sub,
                    "mul"    => Token::Mul,
                    "div"    => Token::Div,
                    "neg"    => Token::Neg,
                    "dup"    => Token::Dup,
                    "var"    => Token::Var,
                    "into"   => Token::Into,
                    "swap"   => Token::Swap,
                    "len"    => Token::Len,
                    "rot"    => Token::Rot,
                    "over"   => Token::Over,
                    "split"  => Token::Split,
                    "splitb" => Token::SplitB,
                    "eq"     => Token::Eq,
                    "lt"     => Token::Lt,
                    "gt"     => Token::Gt,
                    "if"     => Token::If,
                    "else"   => Token::Else,
                    "and"    => Token::And,
                    "or"     => Token::Or,
                    "not"    => Token::Not,
                    "true"   => Token::BoolLit(true),
                    "false"  => Token::BoolLit(false),
                    "quit"   => Token::Quit,
                    "ret"    => Token::Ret,
                    "fun"    => {
                        while let Some(&wc) = chars.peek() {
                            if wc.is_whitespace() { chars.next(); } else { break; }
                        }

                        let mut fun_name = String::new();
                        while let Some(&nc) = chars.peek() {
                            if nc.is_whitespace() || nc == '{' || nc == '}' { break; }
                            fun_name.push(nc);
                            chars.next();
                        }

                        if fun_name.is_empty() {
                            eprintln!("ERROR: Missing function name in declaration");
                            exit(1);
                        }
                        Token::FunDeclaration(fun_name)
                    }

                    "call"  => {
                        while let Some(&wc) = chars.peek() {
                            if wc.is_whitespace() { chars.next(); } else { break; }
                        }

                        let mut fun_name = String::new();
                        while let Some(&nc) = chars.peek() {
                            if nc.is_whitespace() || nc == '{' || nc == '}' { break; }
                            fun_name.push(nc);
                            chars.next();
                        }

                        if fun_name.is_empty() {
                            eprintln!("ERROR: Missing function name in call");
                            exit(1);
                        }
                        Token::FunCall(fun_name)
                    }

                    _ => {
                        if let Ok(num) = word.parse::<i32>() {
                            Token::NumberLit(num)
                        } else {
                            Token::VarLit(word)
                        }
                    }
                };
                tokens.push(token);
            }
        }
    }

    tokens
}
