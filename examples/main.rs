#[macro_use]
extern crate gluten;

use gluten::{
    data::*,
    parse::parse,
    core::{eval, Env}
};

fn parse_int(s: &String) -> i32 {
    s.parse().unwrap()
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let mut env = Env::new();
    env.insert("true".to_string(), r(true));
    env.insert("false".to_string(), r(false));
    env.insert("a".to_string(), r(Box::new(|vec: Vec<Val>| {
        vec.first().unwrap().clone()
    }) as MyFn));
    env.insert("add".to_string(), fun!(add(i32, i32)));
    env.insert("parse_int".to_string(), fun!(parse_int(&String)));

    println!("{:?}", eval(env.clone(), parse("(quote a)").unwrap()).borrow().downcast_ref::<String>());
    println!("{:?}", eval(env.clone(), parse("(parse_int (quote 123))").unwrap()).borrow().downcast_ref::<i32>());
    println!("{:?}", eval(env.clone(), parse("(add (parse_int (quote 123)) (parse_int (quote 123)))").unwrap()).borrow().downcast_ref::<i32>());
    println!("{:?}", eval(env.clone(), sx!{
        (add (parse_int (quote 123)) (parse_int (quote 123)))
    }).borrow().downcast_ref::<i32>());
    println!("{:?}", eval(env.clone(), sx!{
        (if true (quote yes) (quote no))
    }).borrow().downcast_ref::<String>());
    println!("{:?}", eval(env.clone(), parse("(if true (quote yes) (quote no))").unwrap()).borrow().downcast_ref::<String>());
    println!("{:?}", eval(env.clone(), parse("(if false (quote yes) (quote no))").unwrap()).borrow().downcast_ref::<String>());
    println!("{:?}", eval(env.clone(), parse("(let ((x false) (y (quote yes)) (n (quote no))) (quote 1) (if x y n))").unwrap()).borrow().downcast_ref::<String>());
    println!("{:?}", eval(env.clone(), parse("((lambda (a b) a b) (quote 1) (quote 2))").unwrap()).borrow().downcast_ref::<String>());
    println!("{:?}", eval(env.clone(), parse("(do (set f (lambda (a) a)) (f 'aaa))").unwrap()).borrow().downcast_ref::<String>());
}
