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
        use GlutenError::*;
        match self {
            Unbound(sym) => write!(f, "Unbound Symbol: {}", sym.0),
            NotFunction(val) => write!(f, "Not a function: {:?}", val),
            ReadFailed(str) => write!(f, "Read failed: {}", str),
            Str(str) => f.write_str(&str),
        }
    }
}

impl Error for GlutenError {}
