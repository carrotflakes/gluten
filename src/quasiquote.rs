use crate::{
    data::*,
    env::Env,
    error::GlutenError
};

pub fn quasiquote(env: &mut Env, vec: Vec<R<Val>>) -> Result<R<Val>, GlutenError> {
    fn f(env: &mut Env, val: &R<Val>) -> R<Val> {
        enum Q {
            V(R<Val>),
            U(R<Val>),
            US(R<Val>)
        }
        r(Val::Vec(if let Val::Vec(ref vec) = *val.borrow() {
            let mut to_append = false;
            let mut qs = vec![];
            for val in vec {
                if let Val::Vec(ref vec) = *val.borrow() {
                    if vec.len() == 2 {
                        if let Val::Symbol(ref s) = *vec[0].borrow() {
                            if s.0.as_str() == "unquote" {
                                qs.push(Q::U(vec[1].clone()));
                                continue;
                            } else if s.0.as_str() == "unquote-splicing" {
                                qs.push(Q::US(vec[1].clone()));
                                to_append = true;
                                continue;
                            }
                        }
                    }
                }
                qs.push(Q::V(val.clone()));
            }
            let vec_sym = env.reader().borrow_mut().package.intern(&"vec".to_string());
            if to_append {
                let append_sym = env.reader().borrow_mut().package.intern(&"append".to_string());
                vec![append_sym].into_iter().chain(qs.into_iter().map(|q| {
                    match q {
                        Q::V(val) => r(Val::Vec(vec![vec_sym.clone(), f(env, &val)])),
                        Q::U(val) => r(Val::Vec(vec![vec_sym.clone(), val])),
                        Q::US(val) => val
                    }
                })).collect::<Vec<R<Val>>>()
            } else {
                vec![vec_sym.clone()].into_iter().chain(qs.into_iter().map(|q| {
                    match q {
                        Q::V(val) => f(env, &val),
                        Q::U(val) => val,
                        Q::US(_) => panic!()
                    }
                })).collect::<Vec<R<Val>>>()
            }
        } else {
            vec![env.reader().borrow_mut().package.intern(&"quote".to_string()), val.clone()]
        }))
    }
    Ok(f(env, &vec[0]))
}
