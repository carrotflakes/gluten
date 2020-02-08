use crate::data::*;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum GlutenError {
    Unbound(Symbol),
    NotFunction(Val),
    ReadFailed(String),
    Str(String)
}

impl fmt::Display for GlutenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for GlutenError {
    fn description(&self) -> &str {
        "てへぺろ"
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

pub type NativeFn = Box<dyn Fn(Vec<Val>) -> Result<Val, GlutenError>>;
