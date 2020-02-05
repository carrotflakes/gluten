#[macro_use]
extern crate gluten;

use std::io::Write;
use gluten::{
    data::*,
    reader::Reader,
    core::{eval, Env, Macro, macro_expand, defmacro},
    quasiquote::quasiquote
};
use std::time::Instant;

fn parse_int(s: &String) -> i64 {
    s.parse().unwrap()
}

fn write_val<T: Write>(write: &mut T, val: &Val) {
    let val = val.borrow();
    if let Some(s) = val.downcast_ref::<Symbol>() {
        write!(write, "{}", s.0.as_ref()).unwrap();
    } else if let Some(s) = val.downcast_ref::<String>() {
        write!(write, "{:?}", s).unwrap();
    } else if let Some(s) = val.downcast_ref::<i64>() {
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
    gltn.insert("parse_int", fun!(parse_int(&String)));
    gltn.insert("print", r(Box::new(|vec: Vec<Val>| -> Val {
        fn f<T: std::fmt::Debug + 'static>(vec: &Vec<Val>) -> Option<Val> {
            println!("{:?}", vec[0].borrow().downcast_ref::<T>()?);
            Some(r(true))
        }
        f::<f64>(&vec).or_else(|| f::<i64>(&vec)).unwrap()
    }) as MyFn));
    gltn.insert("+", r(Box::new(|vec: Vec<Val>| -> Val {
        fn f<T: std::ops::Add<Output = T> + Default + Copy + 'static>(vec: &Vec<Val>) -> Option<Val> {
            let mut acc = T::default();
            for rv in vec.iter() {
                acc = acc + *rv.borrow().downcast_ref::<T>()?;
            }
            Some(r(acc))
        }
        f::<f64>(&vec).or_else(|| f::<i64>(&vec)).unwrap()
    }) as MyFn));
    gltn.insert("-", r(Box::new(|vec: Vec<Val>| -> Val {
        fn f<T: std::ops::Sub<Output = T> + Copy + 'static>(vec: &Vec<Val>) -> Option<Val> {
            let mut acc = *vec[0].borrow().downcast_ref::<T>()?;
            for rv in vec.iter().skip(1) {
                acc = acc - *rv.borrow().downcast_ref::<T>()?;
            }
            Some(r(acc))
        }
        f::<f64>(&vec).or_else(|| f::<i64>(&vec)).unwrap()
    }) as MyFn));
    gltn.insert("=", r(Box::new(|vec: Vec<Val>| -> Val {
        fn f<T: PartialEq + Copy + 'static>(vec: &Vec<Val>) -> Option<Val> {
            let first = *vec[0].borrow().downcast_ref::<T>()?;
            for rv in vec.iter() {
                if first != *rv.borrow().downcast_ref::<T>()? {
                    return Some(r(false));
                }
            }
            Some(r(true))
        }
        f::<f64>(&vec).or_else(|| f::<i64>(&vec)).unwrap()
    }) as MyFn));
    gltn.insert("<", r(Box::new(|vec: Vec<Val>| -> Val {
        fn f<T: PartialEq + PartialOrd + Copy + 'static>(vec: &Vec<Val>) -> Option<Val> {
            let mut left = *vec[0].borrow().downcast_ref::<T>()?;
            for rv in vec.iter().skip(1) {
                let right = *rv.borrow().downcast_ref::<T>()?;
                if left >= right {
                    return Some(r(false));
                }
                left = right;
            }
            Some(r(true))
        }
        f::<f64>(&vec).or_else(|| f::<i64>(&vec)).unwrap()
    }) as MyFn));
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
    gltn.insert("vec_len", r(Box::new(|vec: Vec<Val>| {
        r(vec[0].borrow().downcast_ref::<Vec<Val>>().unwrap().len() as i64)
    }) as MyFn));
    gltn.insert("vec_get", r(Box::new(|vec: Vec<Val>| {
        r(vec[0].borrow().downcast_ref::<Vec<Val>>().unwrap()[*vec[1].borrow().downcast_ref::<i64>().unwrap() as usize].clone())
    }) as MyFn));
    gltn.insert("defmacro", r(Macro(Box::new(defmacro))));
    gltn.insert("quasiquote", r(Macro(Box::new(quasiquote))));
    gltn.insert("or", r(Macro(Box::new(|env: &mut Env, vec: Vec<Val>| {
        let let_sym = r(env.reader().borrow_mut().intern("let"));
        let if_sym = r(env.reader().borrow_mut().intern("if"));
        let mut ret = vec.last().unwrap().clone();
        let mut i = 0;
        for val in vec.iter().rev().skip(1) {
            i += 1;
            let sym = r(env.reader().borrow_mut().intern(&format!("#gensym{}#", i)));
            ret = r(vec![
                let_sym.clone(),
                r(vec![r(vec![sym.clone(), val.clone()])]),
                r(vec![if_sym.clone(), sym.clone(), sym.clone(), ret])
            ]);
        }
        ret
    }))));

    let start = Instant::now();
    gltn.rep(r#"
    (set zero (parse_int "0"))
    (set one (parse_int "1"))
    (set two (parse_int "2"))
    (set fib
        (lambda (n)
            (if (< zero n)
                (+ (fib (- n one)) (fib (- n two)))
                one)))
    (fib (parse_int "29"))
    "#);
    println!("{:?} msec elapsed", start.elapsed().as_millis());
}
