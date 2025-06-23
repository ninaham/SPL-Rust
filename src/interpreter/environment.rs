use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use crate::interpreter::value::Value;

#[derive(Clone)]
pub struct Environment<'a> {
    pub parent: Option<Rc<Environment<'a>>>,
    pub vars: RefCell<HashMap<String, Value<'a>>>,
}

impl<'a> Environment<'a> {
    pub fn get(&self, key: &str) -> Option<Value<'a>> {
        self.vars.borrow().get(key).map_or_else(
            || self.parent.as_ref().and_then(|p| p.get(key)),
            |v| Some(v.clone()),
        )
    }

    pub fn get_mut<'c, 'b: 'a>(&'b self, key: &str) -> RefMut<'b, Value<'a>> {
        RefMut::map(self.vars.borrow_mut(), |var| {
            var.get_mut(key).expect(&format!("not found: {}", key))
        })
    }

    pub fn insert(&self, key: &str, value: Value<'a>) {
        self.vars.borrow_mut().insert(key.to_string(), value);
    }
}
