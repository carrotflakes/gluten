#[macro_use]
extern crate gluten;

use std::io::Write;
use gluten::{
    data::*,
    reader::Reader,
    env::Env,
    macros::defmacro,
    quasiquote::quasiquote
};
use std::time::Instant;

fn parse_int(s: &String) -> i64 {
    s.parse().unwrap()
}


struct Gltn(Env);

impl Gltn {
    fn new() -> Gltn {
        let reader = std::rc::Rc::new(std::cell::RefCell::new(Reader::default()));
        let mut env = Env::new(reader.clone());
        gluten::special_operators::insert_all(&mut env);
        Gltn(env)
    }

    fn insert(&mut self, str: &str, val: R<Val>) {
        let sym = self.0.reader().borrow_mut().package.intern(&str.to_string());
        self.0.insert(sym, val);
    }

    fn rep(&mut self, str: &str) {
        println!("> {}", str);
        let forms = self.0.reader().borrow_mut().parse_top_level(str).unwrap();
        for form in forms {
            let form = self.0.macro_expand(form).unwrap();
            let form = self.0.eval(form).unwrap();
            println!("{:?}", form);
        }
    }
}

fn main() {
    let mut gltn = Gltn::new();
    gltn.insert("true", r(Val::True));

    let start = Instant::now();
    gltn.rep(r#"
    true
    "#);
    println!("{:?} msec elapsed", start.elapsed().as_millis());
}
