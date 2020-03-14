use crate::data::*;
use crate::env::Env;
use crate::error::GlutenError;

pub fn eval_iter<'a>(env: Env, iter: &mut impl Iterator<Item=&'a Val>) -> Result<Val, GlutenError> {
    let mut ret = r(false);
    for val in iter {
        ret = eval(env.clone(), val.clone())?;
    }
    Ok(ret)
}

pub fn eval(env: Env, val: Val) -> Result<Val, GlutenError> {
    if let Some(s) = val.downcast_ref::<Symbol>() {
        return env.get(s).ok_or_else(|| GlutenError::Unbound(s.clone()));
    } else if let Some(ref vec) = val.downcast_ref::<Vec<Val>>() {
        let first = eval(env.clone(), vec[0].clone())?;
        let handle_err = |err| {
            if let GlutenError::Frozen(val, continuation) = err {
                GlutenError::Frozen(val, continuation)
            } else {
                let name = vec[0].downcast_ref::<Symbol>().map(|s| format!("{}", s.0)).unwrap_or_else(|| "#UNKNOWN".to_owned());
                GlutenError::Stacked(name, Box::new(err))
            }
        };
        let r = if let Some(ref f) = first.downcast_ref::<MyFn>() {
            let args = vec.iter().skip(1).map(|val| eval(env.clone(), val.clone())).collect::<Result<Vec<Val>, GlutenError>>()?;
            f(args)
        } else if let Some(ref f) = first.downcast_ref::<SpecialOperator>() {
            return f(&mut env.clone(), vec).map_err(handle_err);
        } else if let Some(ref f) = first.downcast_ref::<NativeFn>() {
            let mut args: Vec<Val> = Vec::new();
            for val in vec.iter().skip(1) {
                match eval(env.clone(), val.clone()) {
                    Ok(val) => args.push(val),
                    Err(GlutenError::Frozen(val, continuation)) => {
                        let mut new_continuation = Vec::new();
                        new_continuation.push(quote_val(first));
                        new_continuation.extend(args.into_iter().map(quote_val));
                        new_continuation.push(quote_val(continuation));
                        new_continuation.extend(vec.iter().skip(new_continuation.len()).cloned());
                        return Err(GlutenError::Frozen(val, r(new_continuation)));
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            return f(args).map_err(handle_err);
        } else {
            return Err(GlutenError::NotFunction(vec[0].clone()));
        };
        return Ok(r);
    } else {
        return Ok(val.clone());
    }
}

pub struct Macro(pub Box<dyn Fn(&mut Env, Vec<Val>) -> Result<Val, GlutenError>>);

pub fn macro_expand(env: &mut Env, val: Val) -> Result<Val, GlutenError> {
    if let Some(ref vec) = val.downcast_ref::<Vec<Val>>() {
        let expaned_first = macro_expand(env, vec[0].clone())?;
        if let Some(ref s) = expaned_first.downcast_ref::<Symbol>() {
            if let Some(val) = env.get(s) {
                if let Some(ref mac) = val.downcast_ref::<Macro>() {
                    let args = vec.iter().skip(1).cloned().collect();
                    let expanded = (mac.0)(env, args)?;
                    return macro_expand(env, expanded);
                }
            }
        }
        let args = vec.iter().skip(1).map(|v| macro_expand(env, v.clone())).collect::<Result<Vec<Val>, GlutenError>>()?;
        return Ok(r(vec![expaned_first].into_iter().chain(args).collect::<Vec<Val>>()));
    }
    Ok(val)
}

pub fn defmacro(env: &mut Env, vec: Vec<Val>) -> Result<Val, GlutenError> {
    let name = if let Some(name) = vec[0].downcast_ref::<Symbol>() {
        name.clone()
    } else {
        return Err(GlutenError::Str("macro name must be a symbol".to_string()));
    };
    let params = if let Some(params) = vec[1].downcast_ref::<Vec<Val>>() {
        params.clone()
    } else {
        return Err(GlutenError::Str("illegal macro params".to_string()));
    };
    let body: Vec<Val> = vec.iter().skip(2).cloned().collect();
    let mac = r(Macro(Box::new(move |env: &mut Env, args: Vec<Val>| {
        let mut env = env.child();
        for (rs, val) in params.iter().zip(args.iter()) {
            if let Some(s) = (*rs).downcast_ref::<Symbol>() {
                env.insert(s.clone(), val.clone());
                continue;
            }
            return Err(GlutenError::Str("illegal macro".to_string()));
        }
        eval_iter(env, &mut body.iter())
    })));
    env.insert(name, mac.clone());
    Ok(mac)
}

pub fn quote_val(val: Val) -> Val {
    use crate::special_operators::quote;
    r(vec![r(Box::new(quote) as SpecialOperator), val])
}
