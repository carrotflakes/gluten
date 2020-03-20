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
pub struct Meta(pub Val, pub Box<dyn Any>);

pub fn r<T: 'static>(t: T) -> Val {
    Rc::new(t) as Val
}

pub trait ValInterface {
    fn ref_as<T: 'static>(&self) -> Option<&T>;
    fn get_meta<T: 'static>(&self) -> Option<&T>;
    fn wrap_meta<T: 'static>(self, metadata: T) -> Val;
    fn unwrap_meta(&self) -> &Val;
}

impl ValInterface for Val {
    fn ref_as<T: 'static>(&self) -> Option<&T> {
        if let Some(m) = self.downcast_ref::<Meta>() {
            m.0.ref_as()
        } else {
            self.downcast_ref()
        }
    }

    fn get_meta<T: 'static>(&self) -> Option<&T> {
        if let Some(m) = self.downcast_ref::<Meta>() {
            if let Some(m) = m.1.downcast_ref::<T>() {
                Some(m)
            } else {
                m.0.get_meta()
            }
        } else {
            None
        }
    }

    fn wrap_meta<T: 'static>(self, metadata: T) -> Val {
        r(Meta(self, Box::new(metadata)))
    }

    fn unwrap_meta(&self) -> &Val {
        if let Some(m) = self.downcast_ref::<Meta>() {
            m.0.unwrap_meta()
        } else {
            self
        }
    }
}
