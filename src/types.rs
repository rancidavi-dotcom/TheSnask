#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int, // Genérico (i64 por padrão)
    Float,
    String,
    Bool,
    List,
    Dict,
    Void,
    Any,
    // Tipos Precisos de Sistema
    U8,
    I32,
    I64,
    Ptr,
    Function(Vec<Type>, Box<Type>), // param_types, return_type
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::U8 | Type::I32 | Type::I64 | Type::Ptr)
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
            _ => Err(TypeParseError),
        }
    }
}