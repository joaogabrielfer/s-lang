use std::{collections::HashMap, rc::Rc};
use crate::lexer::Token;

pub struct PVM{
    pub data_stack: Vec<RuntimeValue>,
    pub call_stack: Vec<CallFrame>,
    pub elements: HashMap<String, Element>
}

impl PVM {
    pub fn new() -> Self {
        Self {
            data_stack: vec![],
            call_stack: vec![],
            elements: HashMap::new()
        }
    }
}

#[derive(Clone, Debug)]
pub enum Element{
    Var(RuntimeValue),
    Function {
        arguments_t: Vec<RuntimeValueT>,
        return_t: Vec<RuntimeValueT>,
        block: Vec<Token>,
    }
}

pub struct CallFrame {
    pub instructions: Vec<Token>,
    pub ip: usize,
    pub frame_pointer: usize,
}
impl CallFrame {
    pub fn peek(&self) -> Option<&Token> {
        self.instructions.get(self.ip)
    }

    pub fn next(&mut self) -> Option<&Token> {
        let result = self.instructions.get(self.ip);
        self.ip += 1;
        result
    }
}

#[derive(Clone, PartialEq)]
pub enum RuntimeValue {
    Int(i64),
    Bool(bool),
    String(Rc<String>),
    Block(Vec<Token>),
    Function{
        arguments_t: Vec<RuntimeValueT>,
        return_t: Vec<RuntimeValueT>,
        block: Vec<Token>,
    },
}

pub fn default_runtime_int() -> RuntimeValue{
    RuntimeValue::Int(0)
}

pub fn default_runtime_bool() -> RuntimeValue{
    RuntimeValue::Bool(false)
}

pub fn default_runtime_string() -> RuntimeValue{
    RuntimeValue::String(Rc::new("".to_string()))
}

pub fn default_runtime_block() -> RuntimeValue{
    RuntimeValue::Block(vec![])
}

pub fn default_runtime_function() -> RuntimeValue{
    RuntimeValue::Function{arguments_t: vec![],return_t: vec![], block:vec![]}
}

#[derive(Clone, PartialEq, Debug)]
pub enum RuntimeValueT {
    Int,
    Bool,
    String,
    Block,
    Function,
    Any,
    Variadic(Box<RuntimeValueT>),
}

impl RuntimeValueT {
    pub fn to_runtimevalue(&self) -> RuntimeValue {
        match self {
            RuntimeValueT::Int       => default_runtime_int(),
            RuntimeValueT::Bool      => default_runtime_bool(),
            RuntimeValueT::String    => default_runtime_string(),
            RuntimeValueT::Block     => default_runtime_block(),
            RuntimeValueT::Function  => default_runtime_function(),
            RuntimeValueT::Any       => panic!("Tried to convert `any` typo to its RuntimeValue correspondent"),
            RuntimeValueT::Variadic(_)  => panic!("Tried to convert `variadic` typo to its RuntimeValue correspondent"),
        }
    }
}

impl RuntimeValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            RuntimeValue::Int(_)              => "int",
            RuntimeValue::Bool(_)             => "bool",
            RuntimeValue::String(_)           => "str",
            RuntimeValue::Block(_)            => "block",
            RuntimeValue::Function{..}        => "function"
        }
    }
    pub fn compare_type(&self, t: RuntimeValueT) -> bool {
        if t == RuntimeValueT::Any { return true }
        match self {
            RuntimeValue::Int(_)       => t == RuntimeValueT::Int,
            RuntimeValue::Bool(_)      => t == RuntimeValueT::Bool,
            RuntimeValue::String(_)    => t == RuntimeValueT::String,
            RuntimeValue::Block(_)     => t == RuntimeValueT::Block,
            RuntimeValue::Function{..} => t == RuntimeValueT::Function,
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
            RuntimeValue::String(s) => write!(f, "{s}"),
            RuntimeValue::Bool(b) => write!(f, "{b}"),
            RuntimeValue::Block(b) => write!(f, "{:?}", b),
            RuntimeValue::Function { arguments_t, return_t, block } => write!(f, "({:?}) -> {:?} {{{:?}}}", arguments_t, return_t, block),
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
