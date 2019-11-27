use std::rc::Rc;
use std::cell::RefCell;
use std::str::Chars;

type R<T> = Rc<RefCell<T>>;

enum V {
    Symbol(String),
    Cons(R<V>, R<V>),
    Fn(Box<dyn Fn(Vec<R<V>>) -> R<V>>),
    Nil
}

impl std::fmt::Debug for V {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            V::Symbol(ref s) =>
                write!(f, "{:?}", s),
            V::Cons(car, cdr) =>
                write!(f, "Cons({:?}, {:?})", *car.borrow(), *cdr.borrow()),
            V::Fn(_) =>
                write!(f, "Fn?"),
            V::Nil =>
                write!(f, "Nil")
        }
    }
}

impl PartialEq for V {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (V::Symbol(ref s1), V::Symbol(ref s2)) => s1 == s2,
            (V::Cons(car1, cdr1), V::Cons(car2, cdr2)) => car1 == car2 && cdr1 == cdr2,
            (V::Fn(_), V::Fn(_)) => false,
            (V::Nil, V::Nil) => true,
            _ => false
        }
    }
}

fn r(v: V) -> R<V> {
    Rc::new(RefCell::new(v))
}

fn parse(src: &str) -> Result<R<V>, String> {
    parse_value(&src.chars()).map(|x| x.0)
}

fn skip_whitespace<'a> (cs: &Chars<'a>) -> Chars<'a> {
    let mut ncs = cs.clone();
    match ncs.next() {
        Some(c) if c.is_whitespace() =>
            skip_whitespace(&ncs),
        _ => cs.clone()
    }
}

fn parse_value<'a>(cs: &Chars<'a>) -> Result<(R<V>, Chars<'a>), String> {
    let mut cs = skip_whitespace(cs);
    match cs.next() {
        Some('(') => {
            let mut vec = vec![];
            loop {
                match parse_value(&cs) {
                    Ok((rv, ncs)) => {
                        vec.push(rv);
                        cs = ncs.clone();
                    },
                    _ => {
                        break;
                    }
                }
            }
            cs = skip_whitespace(&cs);
            let mut ret_rv = r(V::Nil);
            while let Some(c) = cs.next() {
                if c == ')' {
                    break;
                } else if c == '.' {
                    cs = skip_whitespace(&cs);
                    if let Ok((rv, ncs)) = parse_value(&cs) {
                        ret_rv = rv;
                        cs = ncs.clone();
                        cs = skip_whitespace(&cs);
                        if let Some(')') = cs.next() {
                            break;
                        }
                    }
                }
                return Err("fail".to_string());
            }
            for rv in vec.into_iter().rev() {
                ret_rv = r(V::Cons(rv, ret_rv));
            }
            Ok((ret_rv, cs))
        },
        Some(c) if c.is_alphanumeric() => {
            let mut vec = vec![c];
            let mut ncs = cs.clone();
            loop {
                match ncs.next() {
                    Some(c) if c.is_alphanumeric() => {
                        vec.push(c);
                        cs = ncs.clone();
                    },
                    _ => {
                        break;
                    }
                }
            }
            let s = vec.iter().collect();
            Ok((r(if &s == "nil" { V::Nil } else {V::Symbol(s)}), cs))
        },
        _ => Err("fail".to_string())
    }
}

#[derive(Debug)]
struct RVC<'a>(&'a V);
struct RVCTail<'a>(&'a V);

impl <'a> std::fmt::Display for RVC<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            V::Symbol(ref s) => write!(f, "{}", s),
            v@V::Cons(_, _) =>
                write!(f, "({})", RVCTail(v)),
            V::Nil => write!(f, "nil"),
            v => write!(f, "{:?}!", v)
        }
    }
}

impl <'a> std::fmt::Display for RVCTail<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            V::Cons(ref car, ref cdr) =>
                match *cdr.borrow() {
                    V::Nil => write!(f, "{}", RVC(&car.borrow())),
                    V::Cons(_, _) =>
                        write!(f, "{} {}", RVC(&car.borrow()), RVCTail(&cdr.borrow())),
                    _ => write!(f, "{} . {}", RVC(&car.borrow()), RVC(&cdr.borrow())),
                }
            rv => write!(f, "{}", RVC(rv))
        }
    }
}

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

    println!("{}", RVC(&eval(&env, parse("(quote a)").unwrap()).borrow()));
    println!("{}", RVC(&eval(&env, parse("(if (quote a) (quote b) (quote c))").unwrap()).borrow()));
    println!("{}", RVC(&eval(&env, parse("(a (quote 123))").unwrap()).borrow()));
    //println!("{}", pm!(1, 1 => 2, _ => 3));
}
