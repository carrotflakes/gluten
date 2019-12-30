use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

pub type R<T> = Rc<RefCell<T>>;
pub type Val = R<dyn Any>;
pub type MyFn = Box<dyn Fn(Vec<Val>) -> Val>;

pub fn r<T: 'static>(t: T) -> Val {
    Rc::new(RefCell::new(t)) as Val
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Symbol(pub String);
