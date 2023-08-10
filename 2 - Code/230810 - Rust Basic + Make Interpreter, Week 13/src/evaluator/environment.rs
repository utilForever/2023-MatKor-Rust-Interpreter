use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::evaluator::object::Object;

#[derive(Debug, PartialEq)]
pub struct Environment {
    store: HashMap<String, Object>,
    outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn new_with_outer(outer: Rc<RefCell<Environment>>) -> Self {
        Environment {
            store: HashMap::new(),
            outer: Some(outer),
        }
    }

    pub fn get(&mut self, name: String) -> Option<Object> {
        match self.store.get(&name) {
            Some(value) => Some(value.clone()),
            None => match self.outer {
                Some(ref outer) => outer.borrow_mut().get(name),
                None => None,
            },
        }
    }

    pub fn set(&mut self, name: String, value: &Object) {
        self.store.insert(name, value.clone());
    }
}
