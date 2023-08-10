use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::ast::ast::{Identifier, Statement};
use crate::evaluator::environment::Environment;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Int(i64),
    Bool(bool),
    Function(Vec<Identifier>, Vec<Statement>, Rc<RefCell<Environment>>),
    Null,
    ReturnValue(Box<Object>),
    Error(String),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Object::Int(ref value) => write!(f, "{value}"),
            Object::Bool(ref value) => write!(f, "{value}"),
            Object::Function(ref params, _, _) => {
                let mut result = String::new();

                for (i, Identifier(ref s)) in params.iter().enumerate() {
                    if i < 1 {
                        result.push_str(&format!("{s}"));
                    } else {
                        result.push_str(&format!(", {s}"));
                    }
                }

                write!(f, "fn({result}) {{ ... }}")
            }
            Object::Null => write!(f, "null"),
            Object::ReturnValue(ref value) => write!(f, "{value}"),
            Object::Error(ref value) => write!(f, "{value}"),
        }
    }
}
