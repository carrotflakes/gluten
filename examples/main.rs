#[macro_use]
extern crate gluten;

use std::io::Write;
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

fn write_val<T: Write>(write: &mut T, val: &Val) {
    let val = val.borrow();
    if let Some(s) = val.downcast_ref::<Symbol>() {
        write!(write, "{}", s.0.as_ref()).unwrap();
    } else if let Some(s) = val.downcast_ref::<String>() {
        write!(write, "{:?}", s).unwrap();
    } else if let Some(s) = val.downcast_ref::<i32>() {
        write!(write, "{:?}", s).unwrap();
    } else if let Some(vec) = val.downcast_ref::<Vec<Val>>() {
        write!(write, "(").unwrap();
        let mut first = true;
        for val in vec {
            if first {
                first = false; 
            } else {
                write!(write, " ").unwrap();
            }
            write_val(write, val);
        }
        write!(write, ")").unwrap();
    } else {
        write!(write, "#?#").unwrap();
    }
}

struct Gltn(Env);

impl Gltn {
    fn new() -> Gltn {
        let reader = std::rc::Rc::new(std::cell::RefCell::new(Reader::default()));
        Gltn(Env::new(reader.clone()))
    }

    fn insert(&mut self, str: &str, val: Val) {
        let sym = self.0.reader().borrow_mut().intern(str);
        self.0.insert(sym, val);
    }

    fn rep(&mut self, str: &str) {
        println!("> {}", str);
        let forms = self.0.reader().borrow_mut().parse_top_level(str).unwrap();
        for form in forms {
            let form = macro_expand(&mut self.0, form);
            let form = eval(self.0.clone(), form);
            write_val(&mut std::io::stdout().lock(), &form);
            println!("");
        }
    }
}

fn main() {
    let mut gltn = Gltn::new();
    gltn.insert("true", r(true));
    gltn.insert("false", r(false));
    gltn.insert("a", r(Box::new(|vec: Vec<Val>| {
        vec.first().unwrap().clone()
    }) as MyFn));
    gltn.insert("add", fun!(add(i32, i32)));
    gltn.insert("parse_int", fun!(parse_int(&String)));
    gltn.insert("vec", r(Box::new(|vec: Vec<Val>| {
        r(vec)
    }) as MyFn));
    gltn.insert("append", r(Box::new(|vec: Vec<Val>| {
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
    gltn.insert("eq", r(Box::new(|vec: Vec<Val>| {
        r(std::rc::Rc::ptr_eq(&vec[0], &vec[1]))
    }) as MyFn));
    gltn.insert("symbol?", r(Box::new(|vec: Vec<Val>| {
        r(vec[0].borrow().is::<Symbol>())
    }) as MyFn));
    gltn.insert("vec?", r(Box::new(|vec: Vec<Val>| {
        r(vec[0].borrow().is::<Vec<Val>>())
    }) as MyFn));
    gltn.insert("vec-len", r(Box::new(|vec: Vec<Val>| {
        r(vec[0].borrow().downcast_ref::<Vec<Val>>().unwrap().len() as i32)
    }) as MyFn));
    gltn.insert("vec-get", r(Box::new(|vec: Vec<Val>| {
        r(vec[0].borrow().downcast_ref::<Vec<Val>>().unwrap()[*vec[1].borrow().downcast_ref::<i32>().unwrap() as usize].clone())
    }) as MyFn));

    gltn.rep("(quote a)");
    gltn.rep("\"こんにちは! さようなら\\n改行です\"");
    gltn.rep("(parse_int \"123\")");
    gltn.rep("(add (parse_int \"123\") (parse_int \"123\"))");
    gltn.rep(stringify!{
        (add (parse_int "123") (parse_int "123"))
    });
    gltn.rep(stringify!{
        (if true (quote yes) (quote no))
    });
    gltn.rep("(if true (quote yes) (quote no))");
    gltn.rep("(if false (quote yes) (quote no))");
    gltn.rep("(let ((x false) (y (quote yes)) (n (quote no))) (quote 1) (if x y n))");
    gltn.rep("((lambda (a b) a b) (quote 1) (quote 2))");
    gltn.rep("(do (set f (lambda (a) a)) (f 'aaa))");
    gltn.rep("'1 '2 '3 (quote b) 'add");
    gltn.rep(r"
    ; hello
    '1
    '2
    ; it's a comment
    '3
    (quote b)
    'add
    ; bye!
    ");

    let hello_macro = gltn.0.reader().borrow_mut().parse("(quote hello_macro)").unwrap();
    gltn.insert("hello_macro", r(Macro(Box::new(move |_: &mut Env, _vec: Vec<Val>| {
        hello_macro.clone()
    }))));
    gltn.rep("(hello_macro)");

    gltn.insert("defmacro", r(Macro(Box::new(defmacro))));
    gltn.rep("(do (defmacro my_quote (x) (vec 'quote x)) (my_quote aaa))");

    gltn.insert(
        "quasiquote",
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
    gltn.rep("
        (quasiquote (1 (unquote '2) (unquote-splicing (vec '3 (quasiquote 4)))))
    ");
    gltn.rep("`(1 ,'2 ,@(vec '3 `4))");
}
