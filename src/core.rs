use std::rc::Rc;
use std::cell::RefCell;
use crate::data::*;

use std::collections::HashMap;

struct EnvInner {
    hash_map: HashMap<String, Val>,
    parent: Option<Env>
}

pub struct Env(R<EnvInner>);

impl Env {
    pub fn new() -> Env {
        Env(Rc::new(RefCell::new(EnvInner {
            hash_map: HashMap::new(),
            parent: None
        })))
    }

    pub fn insert(&mut self, s: String, val: Val) {
        self.0.borrow_mut().hash_map.insert(s, val);
    }

    pub fn get(&self, s: &String) -> Option<Val> {
        if let Some(val) = self.0.borrow().hash_map.get(s) {
            Some(val.clone())
        } else {
            self.0.borrow().parent.as_ref().and_then(|env| env.get(s))
        }
    }

    pub fn child(self) -> Env {
        Env(Rc::new(RefCell::new(EnvInner {
            hash_map: HashMap::new(),
            parent: Some(self)
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
        if let Some(val) = env.get(&s.0) {
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
                                        env.insert(s.0.clone(), val);
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
                        let mut env = env.clone();
                        for (rs, val) in params.iter().zip(args.iter()) {
                            if let Some(s) = (*rs).borrow().downcast_ref::<Symbol>() {
                                env.insert(s.0.clone(), val.clone());
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
                            env.0.borrow_mut().hash_map.insert(name.0.clone(), val.clone());
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
