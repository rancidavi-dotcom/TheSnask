use crate::ast::FuncDecl;
use crate::interpreter::Interpreter;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    List(Vec<Value>),
    Dict(HashMap<Value, Value>),
    Nil,
    Function(FuncDecl),
    NativeFunction(fn(Vec<Value>, &mut Interpreter) -> Result<Value, String>),
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    (*n as i64).hash(state);
                } else {
                    n.to_bits().hash(state);
                }
            },
            Value::String(s) => s.hash(state),
            Value::Boolean(b) => b.hash(state),
            Value::List(_) => { "List".hash(state); },
            Value::Dict(_) => { "Dict".hash(state); },
            Value::Nil => "Nil".hash(state),
            Value::Function(f) => f.name.hash(state),
            Value::NativeFunction(f) => (*f as usize).hash(state),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::List(list) => {
                write!(f, "[")?;
                for (i, item) in list.iter().enumerate() {
                    write!(f, "{}", item)?;
                    if i < list.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            },
            Value::Dict(dict) => {
                write!(f, "{{")?;
                let mut first = true;
                for (key, val) in dict {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, val)?;
                    first = false;
                }
                write!(f, "}}")
            },
            Value::Nil => write!(f, "nil"),
            Value::Function(func) => write!(f, "<fun {}>", func.name),
            Value::NativeFunction(_) => write!(f, "<native fun>"),
        }
    }
}

impl Eq for Value {}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::List(list) => !list.is_empty(),
            Value::Dict(dict) => !dict.is_empty(),
            Value::Nil => false,
            // Functions are generally considered truthy if they exist
            Value::Function(_) => true,
            Value::NativeFunction(_) => true,
        }
    }
}
