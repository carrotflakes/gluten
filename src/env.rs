use std::rc::Rc;
use std::cell::RefCell;
use crate::data::*;
use crate::reader::Reader;

use std::collections::HashMap;

struct EnvInner {
    hash_map: HashMap<Symbol, Val>,
    parent: Result<Env, Rc<RefCell<Reader>>>
}

pub struct Env(Rc<RefCell<EnvInner>>);

impl Env {
    pub fn new(reader: Rc<RefCell<Reader>>) -> Env {
        Env(Rc::new(RefCell::new(EnvInner {
            hash_map: HashMap::new(),
            parent: Err(reader)
        })))
    }

    pub fn insert(&mut self, s: Symbol, val: Val) {
        self.0.borrow_mut().hash_map.insert(s, val);
    }

    pub fn get(&self, s: &Symbol) -> Option<Val> {
        if let Some(val) = self.0.borrow().hash_map.get(s) {
            Some(val.clone())
        } else {
            self.0.borrow().parent.as_ref().ok().and_then(|env| env.get(s))
        }
    }

    pub fn reader(&self) -> Rc<RefCell<Reader>> {
        match self.0.borrow().parent {
            Ok(ref parent) => parent.reader(),
            Err(ref reader) => reader.clone()
        }
    }

    pub fn child(&self) -> Env {
        Env(Rc::new(RefCell::new(EnvInner {
            hash_map: HashMap::new(),
            parent: Ok(self.clone())
        })))
    }

    pub fn clone(&self) -> Env {
        Env(self.0.clone())
    }
}
