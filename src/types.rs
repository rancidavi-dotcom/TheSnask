use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Bool,
    Void,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Usize,
    Isize,
    Ptr,
    F32,
    F64,
    User(String),
    Function(Vec<Type>, Box<Type>),
    Struct(String),
    Array(Box<Type>, u64),
    Volatile(Box<Type>),
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::I8 | Type::I16 | Type::I32 | Type::I64 |
            Type::U8 | Type::U16 | Type::U32 | Type::U64 |
            Type::Usize | Type::Isize | Type::Ptr |
            Type::F32 | Type::F64
        )
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::I8 | Type::I16 | Type::I32 | Type::I64 |
            Type::U8 | Type::U16 | Type::U32 | Type::U64 |
            Type::Usize | Type::Isize
        )
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }

    pub fn is_unsigned_integer(&self) -> bool {
        matches!(self, Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Usize)
    }

    pub fn bit_width(&self) -> Option<u32> {
        match self {
            Type::U8 | Type::I8 => Some(8),
            Type::U16 | Type::I16 => Some(16),
            Type::U32 | Type::I32 => Some(32),
            Type::U64 | Type::I64 | Type::Usize | Type::Isize => Some(64),
            Type::Array(inner, _) => inner.bit_width(),
            _ => None,
        }
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Type::Struct(_))
    }

    pub fn size_in_bytes(&self, struct_sizes: &HashMap<String, u64>) -> u64 {
        match self {
            Type::U8 | Type::I8 => 1,
            Type::U16 | Type::I16 => 2,
            Type::U32 | Type::I32 => 4,
            Type::U64 | Type::I64 | Type::Usize | Type::Isize => 8,
            Type::F32 => 4,
            Type::F64 => 8,
            Type::Bool => 1,
            Type::Ptr => 8,
            Type::Struct(name) => *struct_sizes.get(name).unwrap_or(&0),
            Type::Array(inner, count) => inner.size_in_bytes(struct_sizes) * count,
            Type::Volatile(inner) => inner.size_in_bytes(struct_sizes),
            Type::Function(_, _) | Type::User(_) | Type::Void => 8,
        }
    }

    pub fn align_of(&self) -> u64 {
        match self {
            Type::U8 | Type::I8 | Type::Bool => 1,
            Type::U16 | Type::I16 => 2,
            Type::U32 | Type::I32 | Type::F32 => 4,
            Type::U64 | Type::I64 | Type::Usize | Type::Isize | Type::F64 => 8,
            Type::Ptr => 8,
            Type::Struct(_) => 8,
            Type::Array(inner, _) => inner.align_of(),
            Type::Volatile(inner) => inner.align_of(),
            Type::Function(_, _) | Type::User(_) | Type::Void => 8,
        }
    }
}

#[derive(Debug)]
pub struct TypeParseError;

impl std::str::FromStr for Type {
    type Err = TypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "f32" => Ok(Type::F32),
            "f64" => Ok(Type::F64),
            "bool" => Ok(Type::Bool),
            "void" => Ok(Type::Void),
            "i8" => Ok(Type::I8),
            "i16" => Ok(Type::I16),
            "u8" => Ok(Type::U8),
            "u16" => Ok(Type::U16),
            "u32" => Ok(Type::U32),
            "u64" => Ok(Type::U64),
            "i32" => Ok(Type::I32),
            "i64" => Ok(Type::I64),
            "usize" => Ok(Type::Usize),
            "isize" => Ok(Type::Isize),
            "ptr" => Ok(Type::Ptr),
            _ => Err(TypeParseError),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Bool => write!(f, "bool"),
            Type::Void => write!(f, "void"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::Usize => write!(f, "usize"),
            Type::Isize => write!(f, "isize"),
            Type::Ptr => write!(f, "ptr"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::User(n) => write!(f, "{n}"),
            Type::Function(params, ret) => {
                let p: Vec<String> = params.iter().map(|t| t.to_string()).collect();
                write!(f, "fun({}) -> {ret}", p.join(", "))
            }
            Type::Struct(n) => write!(f, "struct {n}"),
            Type::Array(t, n) => write!(f, "[{t}; {n}]"),
            Type::Volatile(t) => write!(f, "volatile {t}"),
        }
    }
}
