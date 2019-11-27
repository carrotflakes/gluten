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
                        f(vec.iter().skip(1).map(|rv| eval(env, rv.clone())).collect())
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
        if let V::Nil = *vec.last().unwrap().borrow() {
            r(V::Symbol(vec.iter().take(vec.len() - 1).map(|rv| {
                if let V::Symbol(ref s) = *rv.borrow() {
                    s.clone()
                } else {
                    format!("{:?}", rv)
                }
            }).collect::<Vec<String>>().join("")))
        } else {
            panic!();
        }
    }))));


    println!("{}", RVC(&eval(&env, parse("(quote a)").unwrap()).borrow()));
    println!("{}", RVC(&eval(&env, parse("(if (quote a) (quote b) (quote c))").unwrap()).borrow()));
    println!("{}", RVC(&eval(&env, parse("(a (quote 123))").unwrap()).borrow()));
    println!("{}", RVC(&eval(&env, parse("(concat (quote a) (quote b))").unwrap()).borrow()));
    //println!("{}", pm!(1, 1 => 2, _ => 3));
}
