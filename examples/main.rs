#[macro_use]
extern crate gluten;

use gluten::{
    data::*,
    reader::Reader,
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
    let mut reader = Reader::default();
    env.insert(reader.intern("true"), r(true));
    env.insert(reader.intern("false"), r(false));
    env.insert(reader.intern("a"), r(Box::new(|vec: Vec<Val>| {
        vec.first().unwrap().clone()
    }) as MyFn));
    env.insert(reader.intern("add"), fun!(add(i32, i32)));
    env.insert(reader.intern("parse_int"), fun!(parse_int(&String)));

    println!("{:?}", eval(env.clone(), reader.parse("(quote a)").unwrap()).borrow().downcast_ref::<Symbol>());
    println!("{:?}", eval(env.clone(), reader.parse("\"こんにちは! さようなら\\n改行です\"").unwrap()).borrow().downcast_ref::<String>());
    println!("{:?}", eval(env.clone(), reader.parse("(parse_int \"123\")").unwrap()).borrow().downcast_ref::<i32>());
    println!("{:?}", eval(env.clone(), reader.parse("(add (parse_int \"123\") (parse_int \"123\"))").unwrap()).borrow().downcast_ref::<i32>());
    println!("{:?}", eval(env.clone(), reader.parse(stringify!{
        (add (parse_int "123") (parse_int "123"))
    }).unwrap()).borrow().downcast_ref::<i32>());
    println!("{:?}", eval(env.clone(), reader.parse(stringify!{
        (if true (quote yes) (quote no))
    }).unwrap()).borrow().downcast_ref::<Symbol>());
    println!("{:?}", eval(env.clone(), reader.parse("(if true (quote yes) (quote no))").unwrap()).borrow().downcast_ref::<Symbol>());
    println!("{:?}", eval(env.clone(), reader.parse("(if false (quote yes) (quote no))").unwrap()).borrow().downcast_ref::<Symbol>());
    println!("{:?}", eval(env.clone(), reader.parse("(let ((x false) (y (quote yes)) (n (quote no))) (quote 1) (if x y n))").unwrap()).borrow().downcast_ref::<Symbol>());
    println!("{:?}", eval(env.clone(), reader.parse("((lambda (a b) a b) (quote 1) (quote 2))").unwrap()).borrow().downcast_ref::<Symbol>());
    println!("{:?}", eval(env.clone(), reader.parse("(do (set f (lambda (a) a)) (f 'aaa))").unwrap()).borrow().downcast_ref::<Symbol>());
    for x in reader.parse_top_level("'1 '2 '3 (quote b) 'add").unwrap() {
        println!("{:?}", eval(env.clone(), x).borrow().downcast_ref::<Symbol>());
    }
    for x in reader.parse_top_level(r"
    ; hello
    '1
    '2
    ; it's a comment
    '3
    (quote b)
    'add
    ; bye!
    ").unwrap() {
        println!("{:?}", eval(env.clone(), x).borrow().downcast_ref::<Symbol>());
    }
}
