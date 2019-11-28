use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

pub type R<T> = Rc<RefCell<T>>;
pub type V = dyn Any;
pub type MyFn = Box<dyn Fn(Vec<R<V>>) -> R<V>>;

pub fn r<T>(t: T) -> R<T> {
    Rc::new(RefCell::new(t))
}
