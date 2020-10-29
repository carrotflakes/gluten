use crate::data::{Symbol, Val, R};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct Package(HashMap<String, R<Val>>);

impl Package {
    pub fn new() -> Package {
        Package(HashMap::new())
    }

    pub fn intern(&mut self, str: &String) -> R<Val> {
        if let Some((_, symbol)) = self.0.get_key_value(str) {
            symbol.clone()
        } else {
            let symbol = Rc::new(RefCell::new(Val::Symbol(Symbol(str.to_string()))));
            self.0.insert(str.to_string(), symbol.clone());
            symbol
        }
    }

    pub fn try_intern(&self, str: &String) -> Option<R<Val>> {
        self.0.get_key_value(str).map(|(_, symbol)| symbol.clone())
    }
}
