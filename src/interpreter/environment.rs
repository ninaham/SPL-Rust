use std::{collections::HashMap, rc::Rc};

use crate::{interpreter::value::Value, spl_builtins::PROCEDURES};

use super::value::ValueRef;

#[derive(Clone, Debug)]
pub struct Environment<'a> {
    pub parent: Option<Rc<Environment<'a>>>,
    pub vars: HashMap<String, ValueRef<'a>>,
}

impl<'a> Environment<'a> {
    pub fn new(parent: Rc<Self>, vars_iter: impl Iterator<Item = (String, ValueRef<'a>)>) -> Self {
        Self {
            parent: Some(parent),
            vars: vars_iter.collect(),
        }
    }

    pub fn new_global(procedures: impl Iterator<Item = (String, ValueRef<'a>)>) -> Self {
        Self {
            parent: None,
            vars: get_builtins().chain(procedures).collect(),
        }
    }

    pub fn get(&self, key: &str) -> Option<ValueRef<'a>> {
        self.vars.get(key).map_or_else(
            || self.parent.as_ref().and_then(|p| p.get(key)),
            |v| Some(v.clone()),
        )
    }
}

pub fn get_builtins<'a>() -> impl Iterator<Item = (String, ValueRef<'a>)> {
    PROCEDURES.iter().filter_map(|&(name, params, body)| {
        body.map(|body| {
            let params = params.iter().map(|p| (p.name.to_string(), p.is_reference));
            (
                name.to_string(),
                Value::new_refcell(Value::new_builtin_proc(params, body)),
            )
        })
    })
}
