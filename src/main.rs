pub mod data;
pub mod parse;
pub mod core;
#[macro_use]
pub mod macros;

use crate::data::*;
use crate::parse::parse;
use crate::core::{eval, Env};


fn parse_int(s: &String) -> i32 {
    s.parse().unwrap()
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}


fn main() {
    let mut env = Env::new();
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
    println!("{:?}", eval(&env, sx!{
        (if true (quote yes) (quote no))
    }).borrow().downcast_ref::<String>());
    // println!("{}", RVC(&eval(&env, parse("(quote a)").unwrap()).borrow()));
    // println!("{}", RVC(&eval(&env, parse("(if (quote a) (quote b) (quote c))").unwrap()).borrow()));
    // println!("{}", RVC(&eval(&env, parse("(a (quote 123))").unwrap()).borrow()));
    // println!("{}", RVC(&eval(&env, parse("(concat (quote a) (quote b))").unwrap()).borrow()));
    // println!("{}", RVC(&eval(&env, parse("(add3 (parse-int (symbol-string (quote 1))) (parse-int (symbol-string (quote 2))))").unwrap()).borrow()));

    //println!("{}", pm!(1, 1 => 2, _ => 3));
    //let a = 1;
    //macroo!(a: usize);
}
