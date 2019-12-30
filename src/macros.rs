#[macro_export]
macro_rules! fun_ {
    ($call:expr, $it:ident, ()) => {
        return r($call);
    };
    ($fn:ident ($($args:expr,)*), $it:ident, (&$t:ty $(, $ts:ty)*)) => {
        if let Some(v) = $it.next().unwrap().borrow_mut().downcast_mut::<$t>() {
            fun_!($fn ($($args,)* v,), $it, ($($ts),*))
        }
    };
    ($fn:ident ($($args:expr,)*), $it:ident, ($t:ty $(, $ts:ty)*)) => {
        if let Some(v) = $it.next().unwrap().borrow().downcast_ref::<$t>() {
            fun_!($fn ($($args,)* *v,), $it, ($($ts),*))
        }
    };
}

#[macro_export]
macro_rules! fun {
    ($fn:ident $params:tt) => {
        r(Box::new(|vec: Vec<Val>| -> Val {
            let mut it = vec.iter();
            fun_!($fn (), it, $params);
            panic!("type mismatch");
        }) as MyFn)
    };
}
