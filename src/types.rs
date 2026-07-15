#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int, // Genérico (i64 por padrão)
    Float,
    F32,
    F64,
    String,
    Bool,
    List,
    ListOf(Box<Type>),
    Dict,
    DictOf(Box<Type>, Box<Type>),
    Void,
    Any,
    // Tipos Precisos de Sistema
    I8,
    I16,
    U8,
    U16,
    U32,
    U64,
    I32,
    I64,
    Usize,
    Isize,
    Ptr,
    User(String),
    Function(Vec<Type>, Box<Type>), // param_types, return_type
    Struct(String),  // repr(C) struct type
    Array(Box<Type>, u64), // fixed-size array [T; N]
    Volatile(Box<Type>), // volatile T
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::Int
                | Type::Float
                | Type::F32
                | Type::F64
                | Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Usize
                | Type::Isize
                | Type::Ptr
        )
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::Int
                | Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Usize
                | Type::Isize
        )
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Type::Float | Type::F32 | Type::F64)
    }

    pub fn is_unsigned_integer(&self) -> bool {
        matches!(
            self,
            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::Usize
        )
    }

    pub fn bit_width(&self) -> Option<u32> {
        match self {
            Type::I8 | Type::U8 => Some(8),
            Type::I16 | Type::U16 => Some(16),
            Type::I32 | Type::U32 => Some(32),
            Type::Int | Type::I64 | Type::U64 | Type::Usize | Type::Isize => Some(64),
            Type::Array(inner, _) => inner.bit_width(),
            _ => None,
        }
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Type::Struct(_))
    }

    /// Returns the LLVM integer type name for this structural type (for sizeof calculation).
    pub fn size_in_bytes(&self, struct_sizes: &std::collections::HashMap<String, u64>) -> u64 {
        match self {
            Type::I8 | Type::U8 => 1,
            Type::I16 | Type::U16 => 2,
            Type::I32 | Type::U32 => 4,
            Type::I64 | Type::U64 | Type::Usize | Type::Isize | Type::Int => 8,
            Type::F32 => 4,
            Type::Float | Type::F64 => 8,
            Type::Bool => 1,
            Type::Ptr | Type::String => 8,
            Type::Struct(name) => *struct_sizes.get(name).unwrap_or(&0),
            Type::Array(inner, count) => inner.size_in_bytes(struct_sizes) * count,
            Type::Volatile(inner) => inner.size_in_bytes(struct_sizes),
            _ => 8,
        }
    }

    pub fn align_of(&self) -> u64 {
        match self {
            Type::I8 | Type::U8 | Type::Bool => 1,
            Type::I16 | Type::U16 => 2,
            Type::I32 | Type::U32 | Type::F32 => 4,
            Type::I64 | Type::U64 | Type::Usize | Type::Isize | Type::Int | Type::Float | Type::F64 => 8,
            Type::Ptr | Type::String => 8,
            Type::Struct(_) => 8,
            Type::Array(inner, _) => inner.align_of(),
            Type::Volatile(inner) => inner.align_of(),
            _ => 8,
        }
    }

    pub fn is_list_like(&self) -> bool {
        matches!(self, Type::List | Type::ListOf(_))
    }

    pub fn is_dict_like(&self) -> bool {
        matches!(self, Type::Dict | Type::DictOf(_, _))
    }
}

#[derive(Debug)]
pub struct TypeParseError;

impl std::str::FromStr for Type {
    type Err = TypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "int" => Ok(Type::Int),
            "float" => Ok(Type::Float),
            "f32" => Ok(Type::F32),
            "f64" => Ok(Type::F64),
            "str" => Ok(Type::String),
            "bool" => Ok(Type::Bool),
            "list" => Ok(Type::List),
            "dict" => Ok(Type::Dict),
            "void" => Ok(Type::Void),
            "any" => Ok(Type::Any),
            // Tipos Precisos
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
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::String => write!(f, "str"),
            Type::Bool => write!(f, "bool"),
            Type::List => write!(f, "list"),
            Type::ListOf(t) => write!(f, "list<{t}>"),
            Type::Dict => write!(f, "dict"),
            Type::DictOf(k, v) => write!(f, "dict<{k}, {v}>"),
            Type::Void => write!(f, "void"),
            Type::Any => write!(f, "any"),
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::Usize => write!(f, "usize"),
            Type::Isize => write!(f, "isize"),
            Type::Ptr => write!(f, "ptr"),
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
