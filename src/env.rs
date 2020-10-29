use std::rc::Rc;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use crate::data::*;
use crate::reader::Reader;
use crate::error::GlutenError;

use std::collections::HashMap;

struct Key(R<Val>);

impl PartialEq for Key {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Key {}

impl Hash for Key {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(&*self.0, state);
    }
}

struct EnvInner {
    hash_map: HashMap<Key, R<Val>>,
    parent: Result<Env, R<Reader>>
}

#[derive(Clone)]
pub struct Env(Rc<RefCell<EnvInner>>);

impl Env {
    pub fn new(reader: R<Reader>) -> Env {
        Env(Rc::new(RefCell::new(EnvInner {
            hash_map: HashMap::new(),
            parent: Err(reader)
        })))
    }

    pub fn macro_expand(&mut self, val: R<Val>) -> Result<R<Val>, GlutenError> {
        macro_expand(self, val)
    }

    pub fn eval(&mut self, val: R<Val>) -> Result<R<Val>, GlutenError> {
        eval(self.clone(), val)
    }

    pub fn insert(&mut self, key: R<Val>, val: R<Val>) {
        self.0.borrow_mut().hash_map.insert(Key(key), val);
    }

    pub fn get(&self, key: &R<Val>) -> Option<R<Val>> {
        if let Some(val) = self.0.borrow().hash_map.get(&Key(key.clone())) {
            Some(val.clone())
        } else {
            self.0.borrow().parent.as_ref().ok().and_then(|env| env.get(key))
        }
    }

    pub fn reader(&self) -> R<Reader> {
        match self.0.borrow().parent {
            Ok(ref parent) => parent.reader(),
            Err(ref reader) => reader.clone()
        }
    }

    pub fn child(&self) -> Env {
        Env(Rc::new(RefCell::new(EnvInner {
            hash_map: HashMap::new(),
            parent: Ok(self.clone())
        })))
    }

    pub fn clone(&self) -> Env {
        Env(self.0.clone())
    }
}

pub fn eval(env: Env, val: R<Val>) -> Result<R<Val>, GlutenError> {
    match *val.borrow() {
        Val::Symbol(ref s) => {
            env.get(&val).ok_or_else(|| GlutenError::Unbound(s.clone()))
        }
        Val::Vec(ref vec) => {
            let first = eval(env.clone(), vec[0].clone())?;
            let handle_err = |err| {
                if let GlutenError::Frozen(val, continuation) = err {
                    GlutenError::Frozen(val, continuation)
                } else {
                    let name = if let Val::Symbol(ref s) = *vec[0].borrow() {
                        format!("{}", s.0)
                    } else {
                        "#UNKNOWN".to_owned()
                    };
                    GlutenError::Stacked(name, Box::new(err))
                }
            };
            match *first.clone().borrow() {
                Val::Fn(ref f) => {
                    let mut args: Vec<R<Val>> = Vec::new();
                    for val in vec.iter().skip(1) {
                        match eval(env.clone(), val.clone()) {
                            Ok(val) => args.push(val),
                            Err(GlutenError::Frozen(val, continuation)) => {
                                let mut new_continuation = Vec::new();
                                new_continuation.push(quote_val(first));
                                new_continuation.extend(args.into_iter().map(quote_val));
                                new_continuation.push(quote_val(continuation));
                                new_continuation.extend(vec.iter().skip(new_continuation.len()).cloned());
                                return Err(GlutenError::Frozen(val, r(Val::Vec(new_continuation))));
                            },
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    f(args).map_err(handle_err)
                }
                Val::SpecialOp(ref f) => {
                    f(&mut env.clone(), &vec).map_err(handle_err)
                }
                _ => {
                    Err(GlutenError::NotFunction(vec[0].clone()))
                }
            }
        }
        _ => {
            Ok(val.clone())
        }
    }
}

pub fn macro_expand(env: &mut Env, val: R<Val>) -> Result<R<Val>, GlutenError> {
    if let Val::Vec(ref vec) = *val.borrow() {
        let expaned_first = macro_expand(env, vec[0].clone())?;
        if let Val::Symbol(_) = *expaned_first.borrow() {
            if let Some(val) = env.get(&expaned_first) {
                if let Val::Macro(ref mac) = *val.borrow() {
                    let args = vec.iter().skip(1).cloned().collect();
                    let expanded = (mac.0)(env, args)?;
                    return macro_expand(env, expanded);
                }
            }
        }
        let args = vec.iter().skip(1).map(|v| macro_expand(env, v.clone())).collect::<Result<Vec<R<Val>>, GlutenError>>()?;
        return Ok(r(Val::Vec(vec![expaned_first].into_iter().chain(args).collect::<Vec<R<Val>>>())));
    }
    Ok(val)
}

pub fn quote_val(val: R<Val>) -> R<Val> {
    use crate::special_operators::quote;
    r(Val::Vec(vec![r(Val::SpecialOp(Box::new(quote))), val]))
}
