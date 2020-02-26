#[macro_use]
extern crate gluten;

use std::io::Write;
use gluten::{
    data::*,
    error::GlutenError,
    reader::Reader,
    core::{eval, Env, Macro, macro_expand, defmacro},
    quasiquote::quasiquote
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
        (|| {
            let forms = self.0.reader().borrow_mut().parse_top_level(str)?;
            for form in forms {
                let form = macro_expand(&mut self.0, form)?;
                let form = eval(self.0.clone(), form)?;
                write_val(&mut std::io::stdout().lock(), &form);
                println!("");
            }
            Ok(())
        })().unwrap_or_else(|e: GlutenError| println!("{}", e));
    }
}

fn main() {
    let mut gltn = Gltn::new();
    gltn.insert("true", r(true));
    gltn.insert("false", r(false));
    gltn.insert("a", r(Box::new(|vec: Vec<Val>| {
        vec.first().cloned().ok_or_else(|| GlutenError::Str("no argument given".to_owned()))
    }) as NativeFn));
    gltn.insert("add", fun!(add(i32, i32)));
    gltn.insert("parse_int", fun!(parse_int(&String)));
    gltn.insert("vec", r(Box::new(|vec: Vec<Val>| {
        Ok(r(vec))
    }) as NativeFn));
    gltn.insert("append", r(Box::new(|vec: Vec<Val>| {
        let mut ret = vec![];
        for v in vec.into_iter() {
            if let Some(ref v) = v.borrow().downcast_ref::<Vec<Val>>() {
                ret.extend_from_slice(v);
            } else {
                return Err(GlutenError::Str("argument type mismatch".to_owned()));
            }
        }
        Ok(r(ret))
    }) as NativeFn));
    gltn.insert("eq", r(Box::new(|vec: Vec<Val>| {
        if vec.len() == 2 {
            Ok(r(std::rc::Rc::ptr_eq(&vec[0], &vec[1])))
        } else {
            Err(GlutenError::Str(format!("eq takes 2 arguments, but given {}", vec.len())))
        }
    }) as NativeFn));
    gltn.insert("symbol?", r(Box::new(|vec: Vec<Val>| {
        Ok(r(vec.get(0).ok_or_else(|| GlutenError::Str(format!("symbol? take 1 argument")))?.borrow().is::<Symbol>()))
    }) as NativeFn));
    gltn.insert("vec?", r(Box::new(|vec: Vec<Val>| {
        Ok(r(vec.get(0).ok_or_else(|| GlutenError::Str(format!("vec? take 1 argument")))?.borrow().is::<Vec<Val>>()))
    }) as NativeFn));
    gltn.insert("vec-len", r(Box::new(|vec: Vec<Val>| {
        let first = vec.get(0).ok_or_else(|| GlutenError::Str(format!("vec-len take 1 argument")))?;
        Ok(r(first.borrow().downcast_ref::<Vec<Val>>().ok_or_else(|| GlutenError::Str(format!("vec-len 1st argument type is Vec<Val>")))?.len() as i32))
    }) as NativeFn));
    gltn.insert("vec-get", r(Box::new(|vec: Vec<Val>| {
        let first = vec.get(0).ok_or_else(|| GlutenError::Str(format!("vec-get take 1 argument")))?;
        Ok(r(first.borrow().downcast_ref::<Vec<Val>>().ok_or_else(|| GlutenError::Str(format!("vec-get 1st argument type is Vec<Val>")))?[*vec[1].borrow().downcast_ref::<i32>().unwrap() as usize].clone()))
    }) as NativeFn));

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
        Ok(hello_macro.clone())
    }))));
    gltn.rep("(hello_macro)");

    gltn.insert("defmacro", r(Macro(Box::new(defmacro))));
    gltn.rep("(do (defmacro my_quote (x) (vec 'quote x)) (my_quote aaa))");

    gltn.insert("quasiquote", r(Macro(Box::new(quasiquote))));
    gltn.rep("
        (quasiquote (1 (unquote '2) (unquote-splicing (vec '3 (quasiquote 4)))))
    ");
    gltn.rep("`(1 ,'2 ,@(vec '3 `4))");

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
        Ok(ret)
    }))));
    gltn.rep("(or (or false false) 'hello 'goodbye)");
    gltn.rep("hogehoge");
    gltn.rep("(false)");
    gltn.rep("(quote 1");
    gltn.rep("(let ((f (lambda (x) (vec-len x)))) (f '123))");
}
