use std::rc::Rc;
use std::cell::RefCell;
use crate::data::*;
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

fn eval_iter<'a>(env: Env, iter: &mut impl Iterator<Item=&'a Val>) -> Val {
    let mut ret = r("nil".to_string()) as Val;
    for val in iter {
        ret = eval(env.clone(), val.clone());
    }
    return ret;
}

pub fn eval(env: Env, val: Val) -> Val {
    if let Some(ref s) = val.borrow().downcast_ref::<Symbol>() {
        if let Some(val) = env.get(s) {
            return val.clone();
        } else {
            panic!("unbound: {:?}", s);
        }
    } else if let Some(ref vec) = val.borrow().downcast_ref::<Vec<Val>>() {
        if let Some(ref s) = vec[0].borrow().downcast_ref::<Symbol>() {
            match s.0.as_str() {
                "quote" =>
                    if vec.len() == 2 {
                        return vec[1].clone();
                    }
                "if" =>
                    if vec.len() == 4 {
                        let cond = eval(env.clone(), vec[1].clone());
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
                                        let val = eval(env.clone(), v[1].clone());
                                        env.insert(s.clone(), val);
                                        continue;
                                    }
                                }
                                panic!("illegal let");
                            }
                            return eval_iter(env, &mut vec.iter().skip(2))
                        }
                    }
                "do" => {
                    return eval_iter(env, &mut vec.iter().skip(1))
                }
                "lambda" => {
                    let params = if let Some(params) = vec[1].borrow().downcast_ref::<Vec<Val>>() {
                        params.clone()
                    } else {
                        panic!("illegal lambda params");
                    };
                    let body: Vec<Val> = vec.iter().skip(2).map(|val| val.clone()).collect();
                    return r(Box::new(move |args: Vec<Val>| {
                        let mut env = env.child();
                        for (rs, val) in params.iter().zip(args.iter()) {
                            if let Some(s) = (*rs).borrow().downcast_ref::<Symbol>() {
                                env.insert(s.clone(), val.clone());
                                continue;
                            }
                            panic!("illegal lambda");
                        }
                        eval_iter(env, &mut body.iter())
                    }) as MyFn);
                }
                "set" => {
                    if vec.len() == 3 {
                        if let Some(name) = vec[1].borrow().downcast_ref::<Symbol>() {
                            let val = eval(env.clone(), vec[2].clone());
                            env.0.borrow_mut().hash_map.insert(name.clone(), val.clone());
                            return val;
                        }
                    }
                    panic!("illegal set");
                }
                _ => {}
            }
        }
        let first = eval(env.clone(), vec[0].clone());
        let r = if let Some(ref f) = first.borrow().downcast_ref::<MyFn>() {
            let args = vec.iter().skip(1).map(|val| eval(env.clone(), val.clone())).collect();
            f(args)
        } else {
            panic!("non-function was applied");
        };
        return r;
    } else {
        return val.clone();
    }
}

pub struct Macro(pub Box<dyn Fn(&mut Env, Vec<Val>) -> Val>);

pub fn macro_expand(env: &mut Env, val: Val) -> Val {
    if let Some(ref vec) = val.borrow().downcast_ref::<Vec<Val>>() {
        let expaned_first = macro_expand(env, vec[0].clone());
        if let Some(ref s) = expaned_first.borrow().downcast_ref::<Symbol>() {
            if let Some(val) = env.get(s) {
                if let Some(ref mac) = val.borrow().downcast_ref::<Macro>() {
                    let args = vec.iter().skip(1).map(|v| v.clone()).collect();
                    let expanded = (mac.0)(env, args);
                    return macro_expand(env, expanded);
                }
            }
        }
        let args = vec.iter().skip(1).map(|v| macro_expand(env, v.clone()));
        return r(vec![expaned_first].into_iter().chain(args).collect::<Vec<Val>>());
    }
    val
}

pub fn defmacro(env: &mut Env, vec: Vec<Val>) -> Val {
    let name = if let Some(name) = vec[0].borrow().downcast_ref::<Symbol>() {
        name.clone()
    } else {
        panic!("macro name must be a symbol");
    };
    let params = if let Some(params) = vec[1].borrow().downcast_ref::<Vec<Val>>() {
        params.clone()
    } else {
        panic!("illegal macro params");
    };
    let body: Vec<Val> = vec.iter().skip(2).map(|val| val.clone()).collect();
    let mac = r(Macro(Box::new(move |env: &mut Env, args: Vec<Val>| {
        let mut env = env.child();
        for (rs, val) in params.iter().zip(args.iter()) {
            if let Some(s) = (*rs).borrow().downcast_ref::<Symbol>() {
                env.insert(s.clone(), val.clone());
                continue;
            }
            panic!("illegal macro");
        }
        eval_iter(env, &mut body.iter())
    })));
    env.insert(name, mac.clone());
    mac
}
