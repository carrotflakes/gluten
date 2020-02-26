use std::rc::Rc;
use std::cell::RefCell;
use crate::data::*;
use crate::error::GlutenError;
use crate::reader::Reader;

use std::collections::HashMap;

struct EnvInner {
    hash_map: HashMap<Symbol, Val>,
    parent: Result<Env, R<Reader>>
}

pub struct Env(R<EnvInner>);

impl Env {
    pub fn new(reader: R<Reader>) -> Env {
        Env(Rc::new(RefCell::new(EnvInner {
            hash_map: HashMap::new(),
            parent: Err(reader)
        })))
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

fn eval_iter<'a>(env: Env, iter: &mut impl Iterator<Item=&'a Val>) -> Result<Val, GlutenError> {
    let mut ret = r(false);
    for val in iter {
        ret = eval(env.clone(), val.clone())?;
    }
    Ok(ret)
}

pub fn eval(env: Env, val: Val) -> Result<Val, GlutenError> {
    if let Some(s) = val.borrow().downcast_ref::<Symbol>() {
        return env.get(s).ok_or_else(|| GlutenError::Unbound(s.clone()));
    } else if let Some(ref vec) = val.borrow().downcast_ref::<Vec<Val>>() {
        if let Some(ref s) = vec[0].borrow().downcast_ref::<Symbol>() {
            match s.0.as_str() {
                "quote" =>
                    if vec.len() == 2 {
                        return Ok(vec[1].clone());
                    }
                "if" =>
                    if vec.len() == 4 {
                        let cond = eval(env.clone(), vec[1].clone())?;
                        return if let Some(false) = cond.borrow().downcast_ref::<bool>() {
                            eval(env, vec[3].clone())
                        } else {
                            eval(env, vec[2].clone())
                        };
                    },
                "let" =>
                    if vec.len() >= 2 {
                        if let Some(v) = vec[1].borrow().downcast_ref::<Vec<Val>>() {
                            let mut env = env.child();
                            for val in v.iter() {
                                if let Some(v) = val.borrow().downcast_ref::<Vec<Val>>() {
                                    if let Some(s) = v[0].borrow().downcast_ref::<Symbol>() {
                                        let val = eval(env.clone(), v[1].clone())?;
                                        env.insert(s.clone(), val);
                                        continue;
                                    }
                                }
                                return Err(GlutenError::Str("illegal let".to_string()));
                            }
                            return eval_iter(env, &mut vec.iter().skip(2));
                        }
                    }
                "do" => {
                    return eval_iter(env, &mut vec.iter().skip(1));
                }
                "lambda" => {
                    let params = if let Some(params) = vec[1].borrow().downcast_ref::<Vec<Val>>() {
                        params.clone()
                    } else {
                        return Err(GlutenError::Str("illegal lambda params".to_string()));
                    };
                    let body: Vec<Val> = vec.iter().skip(2).map(|val| val.clone()).collect();
                    return Ok(r(Box::new(move |args: Vec<Val>| {
                        let mut env = env.child();
                        for (rs, val) in params.iter().zip(args.iter()) {
                            if let Some(s) = (*rs).borrow().downcast_ref::<Symbol>() {
                                env.insert(s.clone(), val.clone());
                                continue;
                            }
                            panic!("illegal lambda");
                        }
                        eval_iter(env, &mut body.iter())
                    }) as NativeFn));
                }
                "set" => {
                    if vec.len() == 3 {
                        if let Some(name) = vec[1].borrow().downcast_ref::<Symbol>() {
                            let val = eval(env.clone(), vec[2].clone())?;
                            env.0.borrow_mut().hash_map.insert(name.clone(), val.clone());
                            return Ok(val);
                        }
                    }
                    return Err(GlutenError::Str("illegal set".to_string()));
                }
                _ => {}
            }
        }
        let first = eval(env.clone(), vec[0].clone())?;
        let r = if let Some(ref f) = first.borrow().downcast_ref::<MyFn>() {
            let args = vec.iter().skip(1).map(|val| eval(env.clone(), val.clone())).collect::<Result<Vec<Val>, GlutenError>>()?;
            f(args)
        } else if let Some(ref f) = first.borrow().downcast_ref::<NativeFn>() {
            let args = vec.iter().skip(1).map(|val| eval(env.clone(), val.clone())).collect::<Result<Vec<Val>, GlutenError>>()?;
            return f(args).map_err(|err| GlutenError::Stacked(vec[0].borrow().downcast_ref::<Symbol>().map(|s| format!("{}", s.0)).unwrap_or_else(|| "#UNKNOWN".to_owned()), Box::new(err)));
        } else {
            return Err(GlutenError::NotFunction(vec[0].clone()));
        };
        return Ok(r);
    } else {
        return Ok(val.clone());
    }
}

pub struct Macro(pub Box<dyn Fn(&mut Env, Vec<Val>) -> Result<Val, GlutenError>>);

pub fn macro_expand(env: &mut Env, val: Val) -> Result<Val, GlutenError> {
    if let Some(ref vec) = val.borrow().downcast_ref::<Vec<Val>>() {
        let expaned_first = macro_expand(env, vec[0].clone())?;
        if let Some(ref s) = expaned_first.borrow().downcast_ref::<Symbol>() {
            if let Some(val) = env.get(s) {
                if let Some(ref mac) = val.borrow().downcast_ref::<Macro>() {
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

pub fn defmacro(env: &mut Env, vec: Vec<Val>) -> Result<Val, GlutenError> {
    let name = if let Some(name) = vec[0].borrow().downcast_ref::<Symbol>() {
        name.clone()
    } else {
        return Err(GlutenError::Str("macro name must be a symbol".to_string()));
    };
    let params = if let Some(params) = vec[1].borrow().downcast_ref::<Vec<Val>>() {
        params.clone()
    } else {
        return Err(GlutenError::Str("illegal macro params".to_string()));
    };
    let body: Vec<Val> = vec.iter().skip(2).cloned().collect();
    let mac = r(Macro(Box::new(move |env: &mut Env, args: Vec<Val>| {
        let mut env = env.child();
        for (rs, val) in params.iter().zip(args.iter()) {
            if let Some(s) = (*rs).borrow().downcast_ref::<Symbol>() {
                env.insert(s.clone(), val.clone());
                continue;
            }
            return Err(GlutenError::Str("illegal macro".to_string()));
        }
        eval_iter(env, &mut body.iter())
    })));
    env.insert(name, mac.clone());
    Ok(mac)
}
