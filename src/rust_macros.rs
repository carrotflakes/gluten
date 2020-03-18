#[macro_export]
macro_rules! fun_ {
    ($call:expr, $it:ident, ()) => {
        return r($call);
    };
    ($fn:ident ($($args:expr,)*), $it:ident, (&$t:ty $(, $ts:ty)*)) => {
        if let Some(v) = $it.next().unwrap().downcast_ref::<$t>() {
            fun_!($fn ($($args,)* v,), $it, ($($ts),*))
        }
    };
    ($fn:ident ($($args:expr,)*), $it:ident, ($t:ty $(, $ts:ty)*)) => {
        if let Some(v) = $it.next().unwrap().downcast_ref::<$t>() {
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

#[macro_export]
macro_rules! destruct {
    ($($pat:tt = $expr:expr),* => $($body:tt)*) => {
        destruct_!([$(($pat = $expr))*] {$($body)*})
    };
}

#[macro_export]
macro_rules! destruct_ {
    ([] $body:block) => {
        $body
    };
    ([($ident:ident = $expr:expr) $($tts:tt)*] $body:block) => {
        if let Some(v) = $expr.downcast_ref::<Symbol>() {
            if v.0.as_str() == stringify!($ident) {
                destruct_!([$($tts)*] $body)
            }
        }
    };
    ([({$ident:ident} = $expr:expr) $($tts:tt)*] $body:block) => {{
        let $ident = $expr;
        destruct_!([$($tts)*] $body)
    }};
    ([({$ident:ident : $ty:ty} = $expr:expr) $($tts:tt)*] $body:block) => {
        if let Some($ident) = $expr.downcast_ref::<$ty>() {
            destruct_!([$($tts)*] $body)
        }
    };
    ([(($($vec_pats:tt)*) = $expr:expr) $($tts:tt)*] $body:block) => {
        if let Some(vec) = $expr.downcast_ref::<Vec<Val>>() {
            let mut it = vec.iter();
            destruct_!([$((Some($vec_pats) = it.next()))* (end it) $($tts)*] $body)
        }
    };
    ([(Some($pat:tt) = $expr:expr) $($tts:tt)*] $body:block) => {{
        if let Some(v) = $expr {
            destruct_!([($pat = v) $($tts)*] $body)
        }
    }};
    ([(end $it:ident) $($tts:tt)*] $body:block) => {{
        if let None = $it.next() {
            destruct_!([$($tts)*] $body)
        }
    }};
}

#[macro_export]
macro_rules! destruct_vec {
    ($($pat:tt),* = $expr:expr => $($body:tt)*) => {
        let mut it = $expr.iter();
        destruct_!([$((Some($pat) = it.next()))* (end it)] {$($body)*})
    };
}
