#[derive(Debug, Clone, PartialEq)]
pub enum Token{
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
    VarLit(String),
    NumberLit(i32),
    Quit,
}

pub fn tokenize(content: String) -> Vec<Token> {
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
                    Err(_) => {
                        if word.starts_with("\"") && word.ends_with("\""){
                            Token::StrLit(word.to_string())
                        } else {
                            Token::VarLit(word.to_string())
                        }
                    }
                }
            }
        };
        tokens.push(token);
    }
    tokens
}
