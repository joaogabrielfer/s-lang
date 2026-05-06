
#[derive(Debug, Clone, PartialEq)]
pub enum Token{
    Push, Pop, Drop, ReadLine, ReadLineB, Clear,
    Add, Sub, Mul, Div, Neg, Dup,
    Len, SplitB,
    Var, Into,
    AsIntB,
    Swap, Rot, Over, Roll, Pick,
    Eq, Gt, Lt,
    OpenCurly, CloseCurly,
    OpenParen, CloseParen,
    If, Else,
    And, Or, Not,
    ElementCall(String), Eval,
    BoolLit(bool), QuotedLit(String), UnquotedLit(String), NumberLit(i64),
    Quit, Ret,
    Include,
}

impl Token {
    pub fn type_name(&self) -> &'static str {
        match self {
            Token::Push           => "Push",
            Token::Pop            => "Pop",
            Token::Drop           => "Drop",
            Token::ReadLine       => "ReadLine",
            Token::ReadLineB      => "ReadLineB",
            Token::Clear          => "Clear",
            Token::Add            => "Add",
            Token::Sub            => "Sub",
            Token::Mul            => "Mul",
            Token::Div            => "Div",
            Token::Neg            => "Neg",
            Token::Dup            => "Dup",
            Token::Len            => "Len",
            Token::SplitB         => "SplitB",
            Token::Var            => "Var",
            Token::Into           => "Into",
            Token::AsIntB         => "AsIntB",
            Token::Swap           => "Swap",
            Token::Rot            => "Rot",
            Token::Over           => "Over",
            Token::Roll           => "Roll",
            Token::Pick           => "Pick",
            Token::Eq             => "Eq",
            Token::Gt             => "Gt",
            Token::Lt             => "Lt",
            Token::OpenCurly      => "OpenCurly",
            Token::CloseCurly     => "CloseCurly",
            Token::OpenParen      => "OpenParen",
            Token::CloseParen     => "CloseParen",
            Token::If             => "If",
            Token::Else           => "Else",
            Token::And            => "And",
            Token::Or             => "Or",
            Token::Not            => "Not",
            Token::ElementCall(_) => "ElementCall",
            Token::Eval           => "Eval",
            Token::BoolLit(_)     => "BoolLit",
            Token::QuotedLit(_)   => "QuotedLit",
            Token::UnquotedLit(_) => "UnquotedLit",
            Token::NumberLit(_)   => "NumberLit",
            Token::Quit           => "Quit",
            Token::Ret            => "Ret",
            Token::Include        => "Include",
        }
    }
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
            '(' => {
                tokens.push(Token::OpenParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::CloseParen);
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
                tokens.push(Token::QuotedLit(string_val));
            }
            '#' => {
                chars.next();
                let mut elm = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc.is_whitespace() || nc == '{' || nc == '}' || nc == '"' || nc == ';' || nc == '(' || nc == ')' {
                        break;
                    }
                    elm.push(nc);
                    chars.next();
                }
                tokens.push(Token::ElementCall(elm));
            }
            _ => {
                let mut word = String::new();

                while let Some(&nc) = chars.peek() {
                    if nc.is_whitespace() || nc == '{' || nc == '}' || nc == '"' || nc == ';' || nc == '(' || nc == ')' {
                        break;
                    }
                    word.push(nc);
                    chars.next();
                }

                let token = match word.as_str() {
                    "push"      => Token::Push,
                    "pop"       => Token::Pop,
                    "drop"      => Token::Drop,
                    "readline"  => Token::ReadLine,
                    "readlineb" => Token::ReadLineB,
                    "clear"     => Token::Clear,
                    "add"       => Token::Add,
                    "sub"       => Token::Sub,
                    "mul"       => Token::Mul,
                    "div"       => Token::Div,
                    "neg"       => Token::Neg,
                    "dup"       => Token::Dup,
                    "var"       => Token::Var,
                    "into"      => Token::Into,
                    "as_intb"    => Token::AsIntB,
                    "swap"      => Token::Swap,
                    "len"       => Token::Len,
                    "rot"       => Token::Rot,
                    "over"      => Token::Over,
                    "roll"      => Token::Roll,
                    "pick"      => Token::Pick,
                    "splitb"    => Token::SplitB,
                    "eq"        => Token::Eq,
                    "lt"        => Token::Lt,
                    "gt"        => Token::Gt,
                    "if"        => Token::If,
                    "else"      => Token::Else,
                    "and"       => Token::And,
                    "or"        => Token::Or,
                    "not"       => Token::Not,
                    "true"      => Token::BoolLit(true),
                    "false"     => Token::BoolLit(false),
                    "quit"      => Token::Quit,
                    "eval"      => Token::Eval,
                    "ret"       => Token::Ret,
                    "include"   => Token::Include,
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

                        Token::ElementCall(fun_name)
                    }

                    _ => {
                        if let Ok(num) = word.parse::<i64>() {
                            Token::NumberLit(num)
                        } else {
                            Token::UnquotedLit(word)
                        }
                    }
                };
                tokens.push(token);
            }
        }
    }

    tokens
}
