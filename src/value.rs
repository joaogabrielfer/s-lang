#[derive(Clone, PartialEq)]
pub enum RuntimeValue {
    Int(i32),
    Bool(bool),
    String(String),
}

impl RuntimeValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            RuntimeValue::Int(_) => "int",
            RuntimeValue::Bool(_) => "bool",
            RuntimeValue::String(_) => "str",
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
