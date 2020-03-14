use std::hash::{Hash, Hasher};
use crate::string_pool::InternedString;

#[derive(Debug, Clone)]
pub struct Symbol(pub InternedString);

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        //Rc::ptr_eq(&self.0, &other.0)
        self.0 == other.0
    }
}

impl Eq for Symbol {}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        //std::ptr::hash(&**self.0, state);
        (&*self.0 as *const String).hash(state);
    }
}
