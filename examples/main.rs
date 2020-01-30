#[macro_use]
extern crate gluten;

use gluten::{
    data::*,
    reader::Reader,
    core::{eval, Env, Macro, macro_expand, defmacro}
};

fn parse_int(s: &String) -> i32 {
    s.parse().unwrap()
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let reader = std::rc::Rc::new(std::cell::RefCell::new(Reader::default()));
    let mut env = Env::new(reader.clone());
    env.insert(reader.borrow_mut().intern("true"), r(true));
    env.insert(reader.borrow_mut().intern("false"), r(false));
    env.insert(reader.borrow_mut().intern("a"), r(Box::new(|vec: Vec<Val>| {
        vec.first().unwrap().clone()
    }) as MyFn));
    env.insert(reader.borrow_mut().intern("add"), fun!(add(i32, i32)));
    env.insert(reader.borrow_mut().intern("parse_int"), fun!(parse_int(&String)));
    env.insert(reader.borrow_mut().intern("vec"), r(Box::new(|vec: Vec<Val>| {
        r(vec)
    }) as MyFn));
    env.insert(reader.borrow_mut().intern("append"), r(Box::new(|vec: Vec<Val>| {
        let mut ret = vec![];
        for v in vec.into_iter() {
            if let Some(ref v) = v.borrow().downcast_ref::<Vec<Val>>() {
                ret.extend_from_slice(v);
            } else {
                panic!();
            }
        }
        r(ret)
    }) as MyFn));
    env.insert(reader.borrow_mut().intern("eq"), r(Box::new(|vec: Vec<Val>| {
        r(std::rc::Rc::ptr_eq(&vec[0], &vec[1]))
    }) as MyFn));
    env.insert(reader.borrow_mut().intern("symbol?"), r(Box::new(|vec: Vec<Val>| {
        r(vec[0].borrow().is::<Symbol>())
    }) as MyFn));
    env.insert(reader.borrow_mut().intern("vec?"), r(Box::new(|vec: Vec<Val>| {
        r(vec[0].borrow().is::<Vec<Val>>())
    }) as MyFn));
    env.insert(reader.borrow_mut().intern("vec-len"), r(Box::new(|vec: Vec<Val>| {
        r(vec[0].borrow().downcast_ref::<Vec<Val>>().unwrap().len() as i32)
    }) as MyFn));
    env.insert(reader.borrow_mut().intern("vec-get"), r(Box::new(|vec: Vec<Val>| {
        r(vec[0].borrow().downcast_ref::<Vec<Val>>().unwrap()[*vec[1].borrow().downcast_ref::<i32>().unwrap() as usize].clone())
    }) as MyFn));

    println!("{:?}", eval(env.clone(), reader.borrow_mut().parse("(quote a)").unwrap()).borrow().downcast_ref::<Symbol>());
    println!("{:?}", eval(env.clone(), reader.borrow_mut().parse("\"こんにちは! さようなら\\n改行です\"").unwrap()).borrow().downcast_ref::<String>());
    println!("{:?}", eval(env.clone(), reader.borrow_mut().parse("(parse_int \"123\")").unwrap()).borrow().downcast_ref::<i32>());
    println!("{:?}", eval(env.clone(), reader.borrow_mut().parse("(add (parse_int \"123\") (parse_int \"123\"))").unwrap()).borrow().downcast_ref::<i32>());
    println!("{:?}", eval(env.clone(), reader.borrow_mut().parse(stringify!{
        (add (parse_int "123") (parse_int "123"))
    }).unwrap()).borrow().downcast_ref::<i32>());
    println!("{:?}", eval(env.clone(), reader.borrow_mut().parse(stringify!{
        (if true (quote yes) (quote no))
    }).unwrap()).borrow().downcast_ref::<Symbol>());
    println!("{:?}", eval(env.clone(), reader.borrow_mut().parse("(if true (quote yes) (quote no))").unwrap()).borrow().downcast_ref::<Symbol>());
    println!("{:?}", eval(env.clone(), reader.borrow_mut().parse("(if false (quote yes) (quote no))").unwrap()).borrow().downcast_ref::<Symbol>());
    println!("{:?}", eval(env.clone(), reader.borrow_mut().parse("(let ((x false) (y (quote yes)) (n (quote no))) (quote 1) (if x y n))").unwrap()).borrow().downcast_ref::<Symbol>());
    println!("{:?}", eval(env.clone(), reader.borrow_mut().parse("((lambda (a b) a b) (quote 1) (quote 2))").unwrap()).borrow().downcast_ref::<Symbol>());
    println!("{:?}", eval(env.clone(), reader.borrow_mut().parse("(do (set f (lambda (a) a)) (f 'aaa))").unwrap()).borrow().downcast_ref::<Symbol>());
    for x in reader.borrow_mut().parse_top_level("'1 '2 '3 (quote b) 'add").unwrap() {
        println!("{:?}", eval(env.clone(), x).borrow().downcast_ref::<Symbol>());
    }
    for x in reader.borrow_mut().parse_top_level(r"
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

    let hello_macro = reader.borrow_mut().parse("(quote hello_macro)").unwrap();
    env.insert(reader.borrow_mut().intern("hello_macro"), r(Macro(Box::new(move |_: &mut Env, _vec: Vec<Val>| {
        hello_macro.clone()
    }))));
    let read = reader.borrow_mut().parse("(hello_macro)").unwrap();
    let macro_expanded = macro_expand(&mut env, read);
    println!("{:?}", eval(env.clone(), macro_expanded).borrow().downcast_ref::<Symbol>());

    env.insert(reader.borrow_mut().intern("defmacro"), r(Macro(Box::new(defmacro))));
    let read = reader.borrow_mut().parse("(do (defmacro my_quote (x) (vec 'quote x)) (my_quote aaa))").unwrap();
    let macro_expanded = macro_expand(&mut env, read);
    println!("{:?}", eval(env.clone(), macro_expanded).borrow().downcast_ref::<Symbol>());
    
    env.insert(
        reader.borrow_mut().intern("quasiquote"),
        r(Macro(Box::new(move |env: &mut Env, vec: Vec<Val>| {
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
        }))));
    let read = reader.borrow_mut().parse("
        (quasiquote (1 (unquote '2) (unquote-splicing (vec '3 '4)))) ; `(1 ,2 ,@(vec 3 4)) => (append (vec (quote 1) 2) (vec 3 4))
    ").unwrap();
    let macro_expanded = macro_expand(&mut env, read);
    println!("{:?}", eval(env.clone(), macro_expanded).borrow().downcast_ref::<Vec<Val>>());
}
