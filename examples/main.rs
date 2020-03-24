#[macro_use]
extern crate gluten;

use std::rc::Rc;
use std::io::Write;
use std::iter::Peekable;
use std::str::CharIndices;
use gluten::{
    data::*,
    error::GlutenError,
    reader::Reader,
    env::Env,
    macros::defmacro,
    quasiquote::quasiquote
};

struct Gltn(Env);

impl Gltn {
    fn new() -> Gltn {
        let mut reader = Reader::default();
        {
            let mut read_table: gluten::reader::ReadTable = std::collections::HashMap::new();
            read_table.insert('#', Rc::new(read_raw_string));
            let r = move |reader: &mut Reader, cs: &mut Peekable<CharIndices>| {
                if let Some((_, c)) = cs.next() {
                   if let Some(f) = read_table.get(&c).cloned() {
                        f(reader, cs)
                    } else {
                        Err(GlutenError::ReadFailed(format!("Expect a read_table charactor")))
                    }
                } else {
                    Err(GlutenError::ReadFailed(format!("Expect a read_table charactor, but found EOS")))
                }
            };
            reader.read_table.insert('#', Rc::new(r));
        }
        let reader = std::rc::Rc::new(std::cell::RefCell::new(reader));
        let mut env = Env::new(reader.clone());
        gluten::special_operators::insert_all(&mut env);
        Gltn(env)
    }

    fn insert(&mut self, str: &str, val: Val) {
        let sym = self.0.reader().borrow_mut().package.intern(&str.to_string());
        self.0.insert(sym, val);
    }

    fn rep(&mut self, str: &str) {
        println!("> {}", str);
        (|| {
            let forms = self.0.reader().borrow_mut().parse_top_level(str)?;
            for form in forms {
                let form = self.0.macro_expand(form)?;
                let form = self.0.eval(form)?;
                write_val(&mut std::io::stdout().lock(), &form);
                println!("");
            }
            Ok(())
        })().unwrap_or_else(|e: GlutenError| {
            if let GlutenError::Frozen(val, continuation) = e {
                println!("Frozen continuation:");
                write_val(&mut std::io::stdout().lock(), &val);
                println!("");
                write_val(&mut std::io::stdout().lock(), &continuation);
                println!(" => ");
                let val = self.0.eval(continuation).unwrap();
                write_val(&mut std::io::stdout().lock(), &val);
                println!("");
                println!("Frozen continuation end");
            } else {
                println!("{}", e);
            }
        });
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
            if let Some(ref v) = v.downcast_ref::<Vec<Val>>() {
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
        Ok(r(vec.get(0).ok_or_else(|| GlutenError::Str(format!("symbol? take 1 argument")))?.is::<Symbol>()))
    }) as NativeFn));
    gltn.insert("vec?", r(Box::new(|vec: Vec<Val>| {
        Ok(r(vec.get(0).ok_or_else(|| GlutenError::Str(format!("vec? take 1 argument")))?.is::<Vec<Val>>()))
    }) as NativeFn));
    gltn.insert("vec-len", r(Box::new(|vec: Vec<Val>| {
        let first = vec.get(0).ok_or_else(|| GlutenError::Str(format!("vec-len take 1 argument")))?;
        Ok(r(first.downcast_ref::<Vec<Val>>().ok_or_else(|| GlutenError::Str(format!("vec-len 1st argument type is Vec<Val>")))?.len() as i32))
    }) as NativeFn));
    gltn.insert("vec-get", r(Box::new(|vec: Vec<Val>| {
        let first = vec.get(0).ok_or_else(|| GlutenError::Str(format!("vec-get take 1 argument")))?;
        Ok(r(first.downcast_ref::<Vec<Val>>().ok_or_else(|| GlutenError::Str(format!("vec-get 1st argument type is Vec<Val>")))?[*vec[1].downcast_ref::<i32>().unwrap() as usize].clone()))
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

    gltn.insert("freeze", r(Box::new(|vec: Vec<Val>| {
        let val = &vec[0];
        Err(GlutenError::Frozen(val.clone(), val.clone()))
    }) as NativeFn));
    gltn.rep("(vec 'a (freeze (vec 'b 'c)) 'd)");

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
        let let_sym = r(env.reader().borrow_mut().package.intern(&"let".to_string()));
        let if_sym = r(env.reader().borrow_mut().package.intern(&"if".to_string()));
        let mut ret = vec.last().unwrap().clone();
        let mut i = 0;
        for val in vec.iter().rev().skip(1) {
            i += 1;
            let sym = r(env.reader().borrow_mut().package.intern(&format!("#gensym{}#", i)));
            ret = r(vec![
                let_sym.clone(),
                r(vec![r(vec![sym.clone(), val.clone()])]),
                r(vec![if_sym.clone(), sym.clone(), sym.clone(), ret])
            ]);
        }
        Ok(ret)
    }))));
    gltn.rep("(or (or false false) 'hello 'goodbye)");

    gltn.rep(r###"(vec 'a ##"hoge"#fuga"##)"###);

    gltn.rep("hogehoge");
    gltn.rep("(false)");
    gltn.rep("(quote 1");
    gltn.rep("(let ((f (lambda (x) (vec-len x)))) (f '123))");
}

fn read_raw_string(_reader: &mut Reader, cs: &mut Peekable<CharIndices>) -> Result<Val, GlutenError> {
    let mut numbers = 2;
    while let Some((_, '#')) = cs.peek() {
        cs.next();
        numbers += 1;
    }
    match cs.next() {
        Some((_, '"')) => {
        }
        Some((_, c)) =>{
            return Err(GlutenError::ReadFailed(format!("Expects '\"', but found {:?}", c)));
        }
        None => {
            return Err(GlutenError::ReadFailed(format!("Expects '\"', but found EOS")));
        }
    }
    let mut vec = Vec::new();
    let mut continual_numbers = 0;
    loop {
        match cs.next() {
            Some((_, c)) if c == '#' => {
                vec.push(c);
                continual_numbers += 1;
                if continual_numbers == numbers {
                    if let Some('"') = vec.get(vec.len() - numbers - 1) {
                        vec.resize(vec.len() - numbers - 1, ' ');
                        break;
                    }
                }
            }
            Some((_, c)) => {
                vec.push(c);
                continual_numbers = 0;
            }
            None => {
                return Err(GlutenError::ReadFailed("raw_string is not closed".to_string()));
            }
        }
    }
    let s: String = vec.iter().collect();
    Ok(r(s))
}

fn parse_int(s: &String) -> i32 {
    s.parse().unwrap()
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn write_val<T: Write>(write: &mut T, val: &Val) {
    if let Some(s) = val.downcast_ref::<Symbol>() {
        write!(write, "{}", s.0.as_str()).unwrap();
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
