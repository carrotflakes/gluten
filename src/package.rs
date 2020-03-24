use std::collections::HashMap;
use crate::data::{Symbol, Val, r};

pub struct Package(HashMap<String, Val>);

impl Package {
    pub fn new() -> Package {
        Package(HashMap::new())
    }

    pub fn intern(&mut self, str: &String) -> Val {
        if let Some((_, symbol)) = self.0.get_key_value(str) {
            symbol.clone()
        } else {
            let symbol = r(Symbol(str.to_string()));
            self.0.insert(str.to_string(), symbol.clone());
            symbol
        }
    }

    pub fn try_intern(&self, str: &String) -> Option<Val> {
        self.0.get_key_value(str).map(|(_, symbol)| symbol.clone())
    }
}
