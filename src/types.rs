#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int, // Genérico (i64 por padrão)
    Float,
    String,
    Bool,
    List,
    ListOf(Box<Type>),
    Dict,
    DictOf(Box<Type>, Box<Type>),
    Void,
    Any,
    // Tipos Precisos de Sistema
    U8,
    I32,
    I64,
    Ptr,
    User(String),
    Function(Vec<Type>, Box<Type>), // param_types, return_type
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::U8 | Type::I32 | Type::I64 | Type::Ptr)
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
            "str" => Ok(Type::String),
            "bool" => Ok(Type::Bool),
            "list" => Ok(Type::List),
            "dict" => Ok(Type::Dict),
            "void" => Ok(Type::Void),
            "any" => Ok(Type::Any),
            // Tipos Precisos
            "u8" => Ok(Type::U8),
            "i32" => Ok(Type::I32),
            "i64" => Ok(Type::I64),
            "ptr" => Ok(Type::Ptr),
            _ => Err(TypeParseError),
        }
    }
}
