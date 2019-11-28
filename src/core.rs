use crate::data::*;

use std::collections::HashMap;

pub type Env = HashMap<String, R<V>>;

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
                    if vec.len() == 5 {
                        unimplemented!()
                            //return eval(env, vec[unimplemented!()].clone())
                    },
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
