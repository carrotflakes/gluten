use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;

pub type R<T> = Rc<RefCell<T>>;

pub enum V {
    Nil,
    Symbol(String),
    Cons(R<V>, R<V>),
    Fn(Box<dyn Fn(Vec<R<V>>) -> R<V>>),
    Any(R<dyn Any>)
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
                write!(f, "Nil"),
            V::Any(_) =>
                write!(f, "Any?")
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

pub fn r(v: V) -> R<V> {
    Rc::new(RefCell::new(v))
}

#[derive(Debug)]
pub struct RVC<'a>(pub &'a V);
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
