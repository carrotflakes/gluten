use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use crate::string_pool::InternedString;

pub type R<T> = Rc<RefCell<T>>;
pub type Val = R<dyn Any>;
pub type MyFn = Box<dyn Fn(Vec<Val>) -> Val>;

pub fn r<T: 'static>(t: T) -> Val {
    Rc::new(RefCell::new(t)) as Val
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol(pub InternedString);
