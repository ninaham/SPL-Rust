use std::{collections::HashMap, rc::Rc};

use super::value::ValueRef;

#[derive(Clone, Debug)]
pub struct Environment<'a> {
    pub parent: Option<Rc<Environment<'a>>>,
    pub vars: HashMap<String, ValueRef<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new(
        parent: Option<Rc<Self>>,
        vars_iter: impl Iterator<Item = (String, ValueRef<'a>)>,
    ) -> Self {
        Self {
            parent,
            vars: vars_iter.collect(),
        }
    }

    pub fn get(&self, key: &str) -> Option<ValueRef<'a>> {
        self.vars.get(key).map_or_else(
            || self.parent.as_ref().and_then(|p| p.get(key)),
            |v| Some(v.clone()),
        )
    }
}
