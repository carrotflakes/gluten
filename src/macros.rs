use crate::data::*;
use crate::env::Env;
use crate::error::GlutenError;

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
        eval_iter(&mut env, &mut body.iter())
    })));
    env.insert(name, mac.clone());
    Ok(mac)
}

pub fn eval_iter<'a>(env: &mut Env, iter: &mut impl Iterator<Item=&'a Val>) -> Result<Val, GlutenError> {
    let mut ret = r(false);
    for val in iter {
        ret = env.eval(val.clone())?;
    }
    Ok(ret)
}
