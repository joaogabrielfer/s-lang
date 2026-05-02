use crate::lexer::Token;
use crate::value::RuntimeValue;

#[derive(Debug, Clone)]
pub enum LangError{
    StackEmpty,
    InvalidToken(Token),
    InvalidImport(String),
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
            Self::InvalidImport(i) => write!(f, "Invalid import '{i}'"),
            Self::FileNotFound{file, reason} => write!(f, "Could not open file {file}: {reason}"),
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
