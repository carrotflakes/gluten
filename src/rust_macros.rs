#[macro_export]
macro_rules! fun_ {
    ($call:expr, $it:ident, ()) => {
        return r($call);
    };
    ($fn:ident ($($args:expr,)*), $it:ident, (&$t:ty $(, $ts:ty)*)) => {
        if let Some(v) = $it.next().unwrap().ref_as::<$t>() {
            fun_!($fn ($($args,)* v,), $it, ($($ts),*))
        }
    };
    ($fn:ident ($($args:expr,)*), $it:ident, ($t:ty $(, $ts:ty)*)) => {
        if let Some(v) = $it.next().unwrap().ref_as::<$t>() {
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
        if let Some(v) = $expr.ref_as::<Symbol>() {
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
        if let Some($ident) = $expr.ref_as::<$ty>() {
            destruct_!([$($tts)*] $body)
        }
    };
    ([(($($vec_pats:tt)*) = $expr:expr) $($tts:tt)*] $body:block) => {
        if let Some(vec) = $expr.ref_as::<Vec<Val>>() {
            let mut it = vec.iter();
            destruct_!([(vec ($($vec_pats)*) it) $($tts)*] $body)
        }
    };
    ([(vec () $it:ident) $($tts:tt)*] $body:block) => {{
        if let None = $it.next() {
            destruct_!([$($tts)*] $body)
        }
    }};
    ([(vec (.. $ident:ident) $it:ident) $($tts:tt)*] $body:block) => {{
        let $ident = $it;
        destruct_!([$($tts)*] $body)
    }};
    ([(vec ($pat:tt $($vec_pats:tt)*) $it:ident) $($tts:tt)*] $body:block) => {
        if let Some(v) = $it.next() {
            destruct_!([($pat = v) (vec ($($vec_pats)*) $it) $($tts)*] $body)
        }
    };
}

#[macro_export]
macro_rules! destruct_vec {
    ($($pat:tt),* = $expr:expr => $($body:tt)*) => {
        let mut it = $expr.iter();
        destruct_!([$((Some($pat) = it.next()))* (end it)] {$($body)*})
    };
}
