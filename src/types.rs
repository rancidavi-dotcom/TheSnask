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
            _ => None,
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
