use std::rc::Rc;
use std::any::Any;
use crate::error::GlutenError;
use crate::env::Env;
pub use crate::symbol::Symbol;

pub type R<T> = Rc<T>;
pub type Val = Rc<dyn Any>;
pub type MyFn = Box<dyn Fn(Vec<Val>) -> Val>; // depricated!
pub type NativeFn = Box<dyn Fn(Vec<Val>) -> Result<Val, crate::error::GlutenError>>;
pub type SpecialOperator = Box<dyn Fn(&mut Env, &Vec<Val>) -> Result<Val, GlutenError>>;
pub struct Macro(pub Box<dyn Fn(&mut Env, Vec<Val>) -> Result<Val, GlutenError>>);

pub fn r<T: 'static>(t: T) -> Val {
    Rc::new(t) as Val
}
