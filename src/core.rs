use crate::data::*;

use std::collections::HashMap;

struct EnvInner {
    hash_map: HashMap<String, R<V>>,
    parent: Option<Env>
}

pub struct Env(R<EnvInner>);

impl Env {
    pub fn new() -> Env {
        Env(r(EnvInner {
            hash_map: HashMap::new(),
            parent: None
        }))
    }

    pub fn insert(&mut self, s: String, rv: R<V>) {
        self.0.borrow_mut().hash_map.insert(s, rv);
    }

    pub fn get(&self, s: &String) -> Option<R<V>> {
        if let Some(rv) = self.0.borrow().hash_map.get(s) {
            Some(rv.clone())
        } else {
            self.0.borrow().parent.as_ref().and_then(|env| env.get(s))
        }
    }

    pub fn child(self) -> Env {
        Env(r(EnvInner {
            hash_map: HashMap::new(),
            parent: Some(self)
        }))
    }

    pub fn clone(&self) -> Env {
        Env(self.0.clone())
    }
}

fn eval_iter<'a>(env: Env, iter: &mut impl Iterator<Item=&'a R<V>>) -> R<V> {
    let mut ret = r("nil".to_string()) as R<V>;
    for rv in iter {
        ret = eval(env.clone(), rv.clone());
    }
    return ret;
}

pub fn eval(env: Env, rv: R<V>) -> R<V> {
    if let Some(ref s) = rv.borrow().downcast_ref::<String>() {
        if let Some(rv) = env.get(s.clone()) {
            return rv.clone();
        } else {
            panic!("unbound: {:?}", s);
        }
    } else if let Some(ref vec) = rv.borrow().downcast_ref::<Vec<R<V>>>() {
        if let Some(ref s) = vec[0].borrow().downcast_ref::<String>() {
            match s.as_str() {
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
                        if let Some(v) = vec[1].borrow().downcast_ref::<Vec<R<V>>>() {
                            let mut env = env.child();
                            for rv in v.iter() {
                                if let Some(v) = rv.borrow().downcast_ref::<Vec<R<V>>>() {
                                    if let Some(s) = v[0].borrow().downcast_ref::<String>() {
                                        let rv = eval(env.clone(), v[1].clone());
                                        env.insert(s.clone(), rv);
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
                    let params = if let Some(params) = vec[1].borrow().downcast_ref::<Vec<R<V>>>() {
                        params.clone()
                    } else {
                        panic!("illegal lambda params");
                    };
                    let body: Vec<R<V>> = vec.iter().skip(2).map(|rv| rv.clone()).collect();
                    return r(Box::new(move |args: Vec<R<V>>| {
                        let mut env = env.clone();
                        for (rs, rv) in params.iter().zip(args.iter()) {
                            if let Some(s) = (*rs).borrow().downcast_ref::<String>() {
                                env.insert(s.clone(), rv.clone());
                                continue;
                            }
                            panic!("illegal lambda");
                        }
                        eval_iter(env, &mut body.iter())
                    }) as MyFn);
                }
                _ => {}
            }
        }
        let first = eval(env.clone(), vec[0].clone());
        let r = if let Some(ref f) = first.borrow().downcast_ref::<MyFn>() {
            let args = vec.iter().skip(1).map(|rv| eval(env.clone(), rv.clone())).collect();
            f(args)
        } else {
            panic!("non-function was applied");
        };
        return r;
    } else {
        return rv.clone();
    }
}
