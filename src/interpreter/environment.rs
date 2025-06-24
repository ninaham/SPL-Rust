use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::interpreter::value::Value;

use super::value::ValueRef;

#[derive(Clone, Debug)]
pub struct Environment<'a> {
    pub parent: Option<Rc<Environment<'a>>>,
    pub vars: RefCell<HashMap<String, ValueRef<'a>>>,
}

impl<'a> Environment<'a> {
    pub fn get(&self, key: &str) -> Option<ValueRef<'a>> {
        self.vars.borrow().get(key).map_or_else(
            || self.parent.as_ref().and_then(|p| p.get(key)),
            |v| Some(v.clone()),
        )
    }

    pub fn insert_ref(&self, key: &str, value: ValueRef<'a>) {
        self.vars.borrow_mut().insert(key.to_string(), value);
    }

    pub fn insert_val(&self, key: &str, value: Value<'a>) {
        self.insert_ref(key, ValueRef::new(RefCell::new(value)));
    }
}
