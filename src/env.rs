use std::rc::Rc;
use std::cell::RefCell;
use crate::data::*;
use crate::reader::Reader;
use crate::error::GlutenError;

use std::collections::HashMap;

struct EnvInner {
    hash_map: HashMap<Symbol, Val>,
    parent: Result<Env, Rc<RefCell<Reader>>>
}

pub struct Env(Rc<RefCell<EnvInner>>);

impl Env {
    pub fn new(reader: Rc<RefCell<Reader>>) -> Env {
        Env(Rc::new(RefCell::new(EnvInner {
            hash_map: HashMap::new(),
            parent: Err(reader)
        })))
    }

    pub fn macro_expand(&mut self, val: Val) -> Result<Val, GlutenError> {
        macro_expand(self, val)
    }

    pub fn eval(&mut self, val: Val) -> Result<Val, GlutenError> {
        eval(self.clone(), val)
    }

    pub fn insert(&mut self, s: Symbol, val: Val) {
        self.0.borrow_mut().hash_map.insert(s, val);
    }

    pub fn get(&self, s: &Symbol) -> Option<Val> {
        if let Some(val) = self.0.borrow().hash_map.get(s) {
            Some(val.clone())
        } else {
            self.0.borrow().parent.as_ref().ok().and_then(|env| env.get(s))
        }
    }

    pub fn reader(&self) -> Rc<RefCell<Reader>> {
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

pub fn eval(env: Env, val: Val) -> Result<Val, GlutenError> {
    if let Some(s) = val.downcast_ref::<Symbol>() {
        return env.get(s).ok_or_else(|| GlutenError::Unbound(s.clone()));
    } else if let Some(ref vec) = val.downcast_ref::<Vec<Val>>() {
        let first = eval(env.clone(), vec[0].clone())?;
        let handle_err = |err| {
            if let GlutenError::Frozen(val, continuation) = err {
                GlutenError::Frozen(val, continuation)
            } else {
                let name = vec[0].downcast_ref::<Symbol>().map(|s| format!("{}", s.0)).unwrap_or_else(|| "#UNKNOWN".to_owned());
                GlutenError::Stacked(name, Box::new(err))
            }
        };
        let r = if let Some(ref f) = first.downcast_ref::<MyFn>() {
            let args = vec.iter().skip(1).map(|val| eval(env.clone(), val.clone())).collect::<Result<Vec<Val>, GlutenError>>()?;
            f(args)
        } else if let Some(ref f) = first.downcast_ref::<SpecialOperator>() {
            return f(&mut env.clone(), vec).map_err(handle_err);
        } else if let Some(ref f) = first.downcast_ref::<NativeFn>() {
            let mut args: Vec<Val> = Vec::new();
            for val in vec.iter().skip(1) {
                match eval(env.clone(), val.clone()) {
                    Ok(val) => args.push(val),
                    Err(GlutenError::Frozen(val, continuation)) => {
                        let mut new_continuation = Vec::new();
                        new_continuation.push(quote_val(first));
                        new_continuation.extend(args.into_iter().map(quote_val));
                        new_continuation.push(quote_val(continuation));
                        new_continuation.extend(vec.iter().skip(new_continuation.len()).cloned());
                        return Err(GlutenError::Frozen(val, r(new_continuation)));
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            return f(args).map_err(handle_err);
        } else {
            return Err(GlutenError::NotFunction(vec[0].clone()));
        };
        return Ok(r);
    } else {
        return Ok(val.clone());
    }
}

pub fn macro_expand(env: &mut Env, val: Val) -> Result<Val, GlutenError> {
    if let Some(ref vec) = val.downcast_ref::<Vec<Val>>() {
        let expaned_first = macro_expand(env, vec[0].clone())?;
        if let Some(ref s) = expaned_first.downcast_ref::<Symbol>() {
            if let Some(val) = env.get(s) {
                if let Some(ref mac) = val.downcast_ref::<Macro>() {
                    let args = vec.iter().skip(1).cloned().collect();
                    let expanded = (mac.0)(env, args)?;
                    return macro_expand(env, expanded);
                }
            }
        }
        let args = vec.iter().skip(1).map(|v| macro_expand(env, v.clone())).collect::<Result<Vec<Val>, GlutenError>>()?;
        return Ok(r(vec![expaned_first].into_iter().chain(args).collect::<Vec<Val>>()));
    }
    Ok(val)
}

pub fn quote_val(val: Val) -> Val {
    use crate::special_operators::quote;
    r(vec![r(Box::new(quote) as SpecialOperator), val])
}
