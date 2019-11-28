pub mod data;
pub mod parse;

use crate::data::*;
use crate::parse::parse;

fn cons_list_vec(rv: &R<V>) -> Vec<R<V>> {
    match *rv.borrow() {
        V::Cons(ref car, ref cdr) => {
            let mut vec = cons_list_vec(cdr);
            vec.push(car.clone());
            vec
        },
        _ => vec![rv.clone()]
    }
}

use std::collections::HashMap;

type Env = HashMap<String, R<V>>;

fn eval(env: &Env, rv: R<V>) -> R<V> {
    match *rv.borrow() {
        V::Nil => {
            return r(V::Nil);
        },
        V::Symbol(ref s) =>
            if let Some(rv) = env.get(s) {
                return rv.clone();
            } else {
                panic!("unbound: {:?}", s);
            },
        _ => {}
    };
    let mut vec = cons_list_vec(&rv);
    vec.reverse();
    let vec = vec;
    let r = match *vec[0].borrow() {
        V::Symbol(ref s) =>
            match s.as_str() {
                "quote" =>
                    if vec.len() == 3 && *vec[2].borrow() == V::Nil {
                        vec[1].clone()
                    } else {
                        panic!("invalid quote");
                    },
                "if" =>
                    if vec.len() == 5 && *vec[4].borrow() == V::Nil {
                        eval(env, vec[if *eval(env, vec[1].clone()).borrow() != V::Nil {2} else {3}].clone())
                    } else {
                        panic!("invalid if");
                    },
                _ => {
                    let first = eval(env, vec[0].clone());
                    let rv = if let V::Fn(ref f) = *first.borrow() {
                        let mut vec: Vec<R<V>> = vec.iter().skip(1).map(|rv| eval(env, rv.clone())).collect();
                        if *vec.last().unwrap().borrow() == V::Nil {
                            vec.pop();
                            f(vec)
                        } else {
                            panic!("improper list");
                        }
                    } else {
                        panic!("function expected, but found: {}", RVC(&first.borrow()));
                    };
                    rv
                }
            },
        _ => r(V::Nil)
    };
    r
}

fn parse_int(s: &String) -> i32 {
    s.parse().unwrap()
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

macro_rules! fun_ {
    ($call:expr, $it:ident, ()) => {
        return r(V::Any(r($call)));
    };
    ($fn:ident ($($args:expr,)*), $it:ident, (&$t:ty $(, $ts:ty)*)) => {
        if let V::Any(ref ra) = *$it.next().unwrap().borrow() {
            if let Some(v) = ra.borrow().downcast_ref::<$t>() {
                fun_!($fn ($($args,)* v,), $it, ($($ts),*))
            }
        }
    };
    ($fn:ident ($($args:expr,)*), $it:ident, ($t:ty $(, $ts:ty)*)) => {
        if let V::Any(ref ra) = *$it.next().unwrap().borrow() {
            if let Some(v) = ra.borrow().downcast_ref::<$t>() {
                fun_!($fn ($($args,)* *v,), $it, ($($ts),*))
            }
        }
    };
}

macro_rules! fun {
    ($fn:ident $params:tt) => {
        r(V::Fn(Box::new(|vec: Vec<R<V>>| {
            let mut it = vec.iter();
            fun_!($fn (), it, $params);
            panic!();
        })))
    };
}
macro_rules! macroo {
    ($i:ident : $t:ty) => {
        println!("{}", $i as $t);
    };
}

fn main() {
    println!("Hello, world!");
    println!("{:?}", parse("  (abc 東京)  "));
    println!("{}", RVC(&parse("  (abc 東京)  ").unwrap().borrow()));
    println!("{}", RVC(&parse("  ( abc () . 東京 ) ").unwrap().borrow()));
    println!("{}", RVC(&parse("  ( abc (a) . 東京 ) ").unwrap().borrow()));
    println!("{}", RVC(&parse("  ( abc (a b) . 東京 ) ").unwrap().borrow()));
    println!("{}", RVC(&parse("  ( abc ( hello world .nil) . 東京 ) ").unwrap().borrow()));

    let mut env = HashMap::new();
    env.insert("a".to_string(), r(V::Fn(Box::new(|vec: Vec<R<V>>| {
        vec.first().unwrap().clone()
    }))));
    env.insert("concat".to_string(), r(V::Fn(Box::new(|vec: Vec<R<V>>| {
        r(V::Symbol(vec.iter().map(|rv| {
            if let V::Symbol(ref s) = *rv.borrow() {
                s.clone()
            } else {
                format!("{:?}", rv)
            }
        }).collect::<Vec<String>>().join("")))
    }))));
    env.insert("symbol-string".to_string(), r(V::Fn(Box::new(|vec: Vec<R<V>>| {
        if let V::Symbol(ref s) = *vec[0].borrow() {
            r(V::Any(r(s.clone())))
        } else {
            panic!();
        }
    }))));
    env.insert("add".to_string(), r(V::Fn(Box::new(|vec: Vec<R<V>>| {
        if vec.len() != 2 {panic!();}
        if let V::Any(ref ra1) = *vec[0].borrow() {
            if let V::Any(ref ra2) = *vec[1].borrow() {
                if let Some(v1) = ra1.borrow().downcast_ref::<i32>() {
                    if let Some(v2) = ra2.borrow().downcast_ref::<i32>() {
                        return r(V::Any(r(v1 + v2)));
                    }
                }
            }
        }
        panic!();
    }))));
    env.insert("add2".to_string(), r(V::Fn(Box::new(|vec: Vec<R<V>>| {
        if vec.len() != 2 {panic!();}
        let mut it = vec.iter();
        if let V::Any(ref ra1) = *it.next().unwrap().borrow() {
            if let Some(v1) = ra1.borrow().downcast_ref::<i32>() {
                if let V::Any(ref ra2) = *it.next().unwrap().borrow() {
                    if let Some(v2) = ra2.borrow().downcast_ref::<i32>() {
                        return r(V::Any(r(v1 + v2)));
                    }
                }
            }
        }
        panic!();
    }))));
    env.insert("add3".to_string(), fun!(add(i32, i32)));
    env.insert("parse-int".to_string(), fun!(parse_int(&String)));

    println!("{}", RVC(&eval(&env, parse("(quote a)").unwrap()).borrow()));
    println!("{}", RVC(&eval(&env, parse("(if (quote a) (quote b) (quote c))").unwrap()).borrow()));
    println!("{}", RVC(&eval(&env, parse("(a (quote 123))").unwrap()).borrow()));
    println!("{}", RVC(&eval(&env, parse("(concat (quote a) (quote b))").unwrap()).borrow()));
    println!("{}", RVC(&eval(&env, parse("(add3 (parse-int (symbol-string (quote 1))) (parse-int (symbol-string (quote 2))))").unwrap()).borrow()));
    //println!("{}", pm!(1, 1 => 2, _ => 3));
    //let a = 1;
    //macroo!(a: usize);
}
