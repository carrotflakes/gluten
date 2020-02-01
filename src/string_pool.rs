use std::collections::HashMap;
use std::rc::Rc;

pub struct StringPool(HashMap<Rc<String>, ()>);

pub type InternedString = Rc<String>;

impl StringPool {
    pub fn new() -> StringPool {
        StringPool(HashMap::new())
    }

    pub fn intern(&mut self, str: &str) -> InternedString {
        let v = Rc::new(str.to_string());
        if let Some((v, _)) = self.0.get_key_value(&v) {
            v.clone()
        } else {
            self.0.insert(v.clone(), ());
            v
        }
    }

    pub fn try_intern(&self, str: &str) -> Option<InternedString> {
        let v = Rc::new(str.to_string());
        self.0.get_key_value(&v).map(|(v, _)| v.clone())
    }
}
