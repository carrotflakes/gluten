use crate::data::*;

use std::collections::HashMap;

pub struct Env<'a> {
    hash_map: HashMap<String, R<V>>,
    parent: Option<&'a Env<'a>>
}

impl<'a> Env<'a> {
    pub fn new() -> Env<'a> {
        Env {
            hash_map: HashMap::new(),
            parent: None
        }
    }

    pub fn insert(&mut self, s: String, rv: R<V>) {
        self.hash_map.insert(s, rv);
    }

    pub fn get(&self, s: &String) -> Option<R<V>> {
        if let Some(rv) = self.hash_map.get(s) {
            Some(rv.clone())
        } else {
            self.parent.and_then(|renv| renv.get(s))
        }
    }

    pub fn child(&'a self) -> Env<'a> {
        Env {
            hash_map: HashMap::new(),
            parent: Some(self)
        }
    }
}

pub fn eval(env: &Env, rv: R<V>) -> R<V> {
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
                        let cond = eval(env, vec[1].clone());
                        return if let Some(false) = cond.borrow().downcast_ref::<bool>() {
                            eval(env, vec[3].clone())
                        } else {
                            eval(env, vec[2].clone())
                        };
                    },
                "let" =>
                    if vec.len() == 3 {
                        if let Some(v) = vec[1].borrow().downcast_ref::<Vec<R<V>>>() {
                            let mut env = env.child();
                            for rv in v.iter() {
                                if let Some(v) = rv.borrow().downcast_ref::<Vec<R<V>>>() {
                                    if let Some(s) = v[0].borrow().downcast_ref::<String>() {
                                        env.insert(s.clone(), eval(&env, v[1].clone()));
                                        continue;
                                    }
                                }
                                panic!("illegal let");
                            }
                            return eval(&env, vec[2].clone())
                        }
                    }
                _ => {}
            }
        }
        let first = eval(env, vec[0].clone());
        let r = if let Some(ref f) = first.borrow().downcast_ref::<MyFn>() {
            let args = vec.iter().skip(1).map(|rv| eval(env, rv.clone())).collect();
            f(args)
        } else {
            panic!("non-function was applied");
        };
        return r;
    } else {
        return rv.clone();
    }
}
