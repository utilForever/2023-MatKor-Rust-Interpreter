use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Int(i64),
    Bool(bool),
    Null,
    ReturnValue(Box<Object>),
    Error(String),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Object::Int(ref value) => write!(f, "{value}"),
            Object::Bool(ref value) => write!(f, "{value}"),
            Object::Null => write!(f, "null"),
            Object::ReturnValue(ref value) => write!(f, "{value}"),
            Object::Error(ref value) => write!(f, "{value}"),
        }
    }
}
