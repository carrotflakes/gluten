use std::rc::Rc;
use std::any::Any;
pub use crate::symbol::Symbol;

pub type R<T> = Rc<T>;
pub type Val = Rc<dyn Any>;
pub type MyFn = Box<dyn Fn(Vec<Val>) -> Val>; // depricated!
pub type NativeFn = Box<dyn Fn(Vec<Val>) -> Result<Val, crate::error::GlutenError>>;
pub type SpecialOperator = Box<dyn Fn(&mut crate::core::Env, &Vec<Val>) -> Result<Val, crate::error::GlutenError>>;

pub fn r<T: 'static>(t: T) -> Val {
    Rc::new(t) as Val
}
