use crate::lexer::Token;
use crate::value::RuntimeValue;

macro_rules! ret_error {
    ($variant:ident, $arg:expr) => {
        return Err(LangError::$variant($arg.into()).into())
    };

    (UnexpectedToken, [$($exp:ident),*], $got:expr) => {
        return Err(LangError::UnexpectedToken {
            exp: vec![ $(
                ret_error!(@token $exp)
            ),* ],
            got: $got.map(|t| t.clone())
        }.into())
    };

    // --- INTERNAL HELPER RULES FOR TOKENS ---
    // Specifically handle Tuple Variants (Tokens that hold data)
    (@token NumberLit) => { Token::NumberLit(Default::default()) };
    (@token QuotedLit) => { Token::QuotedLit(Default::default()) };
    (@token UnquotedLit) => { Token::UnquotedLit(Default::default()) };
    (@token BoolLit) => { Token::BoolLit(Default::default()) };
    (@token FunDeclaration) => { Token::FunDeclaration(Default::default()) };
    (@token FunCall) => { Token::FunCall(Default::default()) };

    // Fallback for Unit Variants (e.g., OpenCurly, Var, Swap, Eq, etc.)
    (@token $variant:ident) => {
        Token::$variant
    };
    // -----------------------------------------

    // 2. Generic Vector Helper: throw!(UnexpectedTypes, [v1, v2], got_vec)
    ($variant:ident,[$($exp_items:expr),*], $got:expr) => {
        return Err(LangError::$variant {
            exp: vec![ $($exp_items.into()),* ],
            got: $got.into()
        }.into())
    };

    // 3. Keep the Struct syntax for others
    ($variant:ident { $($field:ident : $val:expr),* $(,)? }) => {
        return Err(LangError::$variant {
            $($field: $val.into()),*
        }.into())
    };

    // 4. Basic variants
    ($variant:ident) => { return Err(LangError::$variant.into()) };
}

pub(crate) use ret_error;


#[derive(Debug, Clone)]
pub enum LangError{
    StackUnderflow,
    InvalidToken(Token),
    InvalidPath(String),
    FileNotFound{
        file: String,
        reason: String,
    },
    UnsufficientValues{
        op: String,
        exp: usize,
        got: usize
    },
    UnexpectedToken{
        exp: Vec<Token>,
        got: Option<Token>,
    },
    UnexpectedTypes{
        exp: Vec<RuntimeValue>,
        got: Vec<RuntimeValue>,
    },
    IndexOutOfRange(i64),
    StackIndexOutOfRange {
        op: String,
        index: usize,
        stack_len: usize,
    },
    UndeclaredObject{
        t: String,
        name: String,
    },
    UnknownType(String),
}



impl std::fmt::Display for LangError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidToken(tk) => write!(f, "Invalid token '{:?}'", tk),
            Self::InvalidPath(p) => write!(f, "Invalid path '{p}'"),
            Self::FileNotFound{file, reason} => write!(f, "Could not open file {file}: {reason}"),
            Self::StackUnderflow => write!(f, "Stack Underflow"),
            Self::UnsufficientValues{op, exp, got} => write!(f, "Cannot {op}: Expected {exp} value in the stack, got {got}"),
            Self::UnexpectedToken{exp, got} => {
                let type_names: Vec<&str> = exp
                    .iter()
                    .map(|x| x.type_name())
                    .collect();

                write!(f, "Expected token '{:?}' got '{:?}'", type_names, got)
            },
            Self::UndeclaredObject{t, name} => write!(f, "Undeclared {t} {name}"),
            Self::UnexpectedTypes { exp, got } => {
                let exp_type_names: Vec<&str> = exp
                    .iter()
                    .map(|x| x.type_name())
                    .collect();
                let got_type_names: Vec<&str> = got
                    .iter()
                    .map(|x| x.type_name())
                    .collect();
                write!(f, "Expected {:?}, got {:?}", exp_type_names, got_type_names)
            }
            Self::IndexOutOfRange(i) => write!(f, "Index out of range: {i}"),
            Self::StackIndexOutOfRange { op, index, stack_len } => write!(f, "Cannot {op}: index {index} out of range for stack of length {stack_len}"),
            Self::UnknownType(t) => write!(f, "Unknown type '{}'", t),
        }
    }
}

impl std::error::Error for LangError {}
