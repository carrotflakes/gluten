use crate::{
    data::*,
    core::Env
};

pub fn quasiquote(env: &mut Env, vec: Vec<Val>) -> Val {
    fn f(env: &mut Env, val: &Val) -> Val {
        enum Q {
            V(Val),
            U(Val),
            US(Val)
        }
        if let Some(vec) = val.borrow().downcast_ref::<Vec<Val>>() {
            let mut to_append = false;
            let mut qs = vec![];
            for val in vec {
                if let Some(vec) = val.borrow().downcast_ref::<Vec<Val>>() {
                    if vec.len() == 2 {
                        if let Some(s) = vec[0].borrow().downcast_ref::<Symbol>() {
                            if s.0.as_ref() == "unquote" {
                                qs.push(Q::U(vec[1].clone()));
                                continue;
                            } else if s.0.as_ref() == "unquote-splicing" {
                                qs.push(Q::US(vec[1].clone()));
                                to_append = true;
                                continue;
                            }
                        }
                    }
                }
                qs.push(Q::V(val.clone()));
            }
            let vec_sym = r(env.reader().borrow_mut().intern("vec"));
            if to_append {
                let append_sym = r(env.reader().borrow_mut().intern("append"));
                r(vec![append_sym].into_iter().chain(qs.into_iter().map(|q| {
                    match q {
                        Q::V(val) => r(vec![vec_sym.clone(), f(env, &val)]),
                        Q::U(val) => r(vec![vec_sym.clone(), val]),
                        Q::US(val) => val
                    }
                })).collect::<Vec<Val>>())
            } else {
                r(vec![vec_sym.clone()].into_iter().chain(qs.into_iter().map(|q| {
                    match q {
                        Q::V(val) => f(env, &val),
                        Q::U(val) => val,
                        Q::US(_) => panic!()
                    }
                })).collect::<Vec<Val>>())
            }
        } else {
            r(vec![r(env.reader().borrow_mut().intern("quote")), val.clone()])
        }
    }
    f(env, &vec[0])
}
