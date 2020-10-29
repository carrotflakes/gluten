use crate::env::Env;
use crate::error::GlutenError;
pub use crate::package::Package;
use std::rc::Rc;
use std::{any::Any, cell::RefCell};

pub type R<T> = Rc<RefCell<T>>;
pub type NativeFn = Box<dyn Fn(Vec<R<Val>>) -> Result<R<Val>, crate::error::GlutenError>>;
pub type SpecialOperator = Box<dyn Fn(&mut Env, &Vec<R<Val>>) -> Result<R<Val>, GlutenError>>;
pub struct Macro(pub Box<dyn Fn(&mut Env, Vec<R<Val>>) -> Result<R<Val>, GlutenError>>);

pub enum Val {
    Symbol(Symbol),
    Vec(Vec<R<Val>>),
    Fn(NativeFn),
    SpecialOp(SpecialOperator),
    Macro(Macro),
    Int(i64),
    Float(f64),
    True,
    False,
    Any(Box<dyn Any>),
}

impl std::fmt::Debug for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Val::Symbol(str) => write!(f, "{:?}", str),
            Val::Vec(_) => write!(f, "vec#"),
            Val::Fn(_) => write!(f, "fn#"),
            Val::SpecialOp(_) => write!(f, "special-operator#"),
            Val::Macro(_) => write!(f, "macro#"),
            Val::Int(v) => write!(f, "{:?}", v),
            Val::Float(v) => write!(f, "{:?}", v),
            Val::True => write!(f, "true"),
            Val::False => write!(f, "false"),
            Val::Any(_) => write!(f, "any#"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Symbol(pub String);

pub fn r(val: Val) -> R<Val> {
    Rc::new(RefCell::new(val))
}
