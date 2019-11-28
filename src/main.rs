pub mod data;
pub mod parse;

use crate::data::*;
use crate::parse::parse;

use std::collections::HashMap;

type Env = HashMap<String, R<V>>;

fn eval(env: &Env, rv: R<V>) -> R<V> {
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

fn parse_int(s: &String) -> i32 {
    s.parse().unwrap()
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

macro_rules! fun_ {
    ($call:expr, $it:ident, ()) => {
        return r($call);
    };
    ($fn:ident ($($args:expr,)*), $it:ident, (&$t:ty $(, $ts:ty)*)) => {
        if let Some(v) = $it.next().unwrap().borrow_mut().downcast_mut::<$t>() {
            fun_!($fn ($($args,)* v,), $it, ($($ts),*))
        }
    };
    ($fn:ident ($($args:expr,)*), $it:ident, ($t:ty $(, $ts:ty)*)) => {
        if let Some(v) = $it.next().unwrap().borrow().downcast_ref::<$t>() {
            fun_!($fn ($($args,)* *v,), $it, ($($ts),*))
        }
    };
}

macro_rules! fun {
    ($fn:ident $params:tt) => {
        r(Box::new(|vec: Vec<R<V>>| -> R<V> {
            let mut it = vec.iter();
            fun_!($fn (), it, $params);
            panic!();
        }) as MyFn)
    };
}

macro_rules! sx {
    (($($xs:tt)*)) => {
        r(vec![$(sx!{$xs}),*]) as R<V>
    };
    ($x:tt) => {
        r(stringify!($x).to_string()) as R<V>
    };
}

fn main() {
    let mut env: Env = HashMap::new();
    env.insert("a".to_string(), r(Box::new(|vec: Vec<R<V>>| {
        vec.first().unwrap().clone()
    }) as MyFn));
    // env.insert("concat".to_string(), r(|vec: Vec<R<V>>| {
    //     r(V::Symbol(vec.iter().map(|rv| {
    //         if let V::Symbol(ref s) = *rv.borrow() {
    //             s.clone()
    //         } else {
    //             format!("{:?}", rv)
    //         }
    //     }).collect::<Vec<String>>().join("")))
    // }));
    // env.insert("symbol-string".to_string(), r(|vec: Vec<R<V>>| {
    //     if let V::Symbol(ref s) = *vec[0].borrow() {
    //         r(V::Any(r(s.clone())))
    //     } else {
    //         panic!();
    //     }
    // }));
    // env.insert("add".to_string(), r(|vec: Vec<R<V>>| {
    //     if vec.len() != 2 {panic!();}
    //     if let V::Any(ref ra1) = *vec[0].borrow() {
    //         if let V::Any(ref ra2) = *vec[1].borrow() {
    //             if let Some(v1) = ra1.borrow().downcast_ref::<i32>() {
    //                 if let Some(v2) = ra2.borrow().downcast_ref::<i32>() {
    //                     return r(V::Any(r(v1 + v2)));
    //                 }
    //             }
    //         }
    //     }
    //     panic!();
    // }));
    // env.insert("add2".to_string(), r(|vec: Vec<R<V>>| {
    //     if vec.len() != 2 {panic!();}
    //     let mut it = vec.iter();
    //     if let V::Any(ref ra1) = *it.next().unwrap().borrow() {
    //         if let Some(v1) = ra1.borrow().downcast_ref::<i32>() {
    //             if let V::Any(ref ra2) = *it.next().unwrap().borrow() {
    //                 if let Some(v2) = ra2.borrow().downcast_ref::<i32>() {
    //                     return r(V::Any(r(v1 + v2)));
    //                 }
    //             }
    //         }
    //     }
    //     panic!();
    // }));
    env.insert("add3".to_string(), fun!(add(i32, i32)));
    env.insert("parse_int".to_string(), fun!(parse_int(&String)));

    println!("{:?}", eval(&env, parse("(quote a)").unwrap()).borrow().downcast_ref::<String>());
    println!("{:?}", eval(&env, parse("(parse_int (quote 123))").unwrap()).borrow().downcast_ref::<i32>());
    println!("{:?}", eval(&env, parse("(add3 (parse_int (quote 123)) (parse_int (quote 123)))").unwrap()).borrow().downcast_ref::<i32>());
    println!("{:?}", eval(&env, sx!{
        (add3 (parse_int (quote 123)) (parse_int (quote 123)))
    }).borrow().downcast_ref::<i32>());
    // println!("{}", RVC(&eval(&env, parse("(quote a)").unwrap()).borrow()));
    // println!("{}", RVC(&eval(&env, parse("(if (quote a) (quote b) (quote c))").unwrap()).borrow()));
    // println!("{}", RVC(&eval(&env, parse("(a (quote 123))").unwrap()).borrow()));
    // println!("{}", RVC(&eval(&env, parse("(concat (quote a) (quote b))").unwrap()).borrow()));
    // println!("{}", RVC(&eval(&env, parse("(add3 (parse-int (symbol-string (quote 1))) (parse-int (symbol-string (quote 2))))").unwrap()).borrow()));

    //println!("{}", pm!(1, 1 => 2, _ => 3));
    //let a = 1;
    //macroo!(a: usize);
}
