use std::rc::Rc;
use std::any::Any;
use std::hash::{Hash, Hasher};
use crate::string_pool::InternedString;

pub type R<T> = Rc<T>;
pub type Val = Rc<dyn Any>;
pub type MyFn = Box<dyn Fn(Vec<Val>) -> Val>;

pub fn r<T: 'static>(t: T) -> Val {
    Rc::new(t) as Val
}

#[derive(Debug, Clone)]
pub struct Symbol(pub InternedString);

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Symbol {}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (&*self.0 as *const String).hash(state);
    }
}

pub type NativeFn = Box<dyn Fn(Vec<Val>) -> Result<Val, crate::error::GlutenError>>;
