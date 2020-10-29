use crate::data::*;
use crate::env::Env;
use crate::error::GlutenError;

pub fn defmacro(env: &mut Env, vec: Vec<R<Val>>) -> Result<R<Val>, GlutenError> {
    if let Val::Symbol(_) = *vec[0].borrow() {
    } else {
        return Err(GlutenError::Str("macro name must be a symbol".to_string()));
    }
    let params = if let Val::Vec(ref params) = *vec[1].borrow() {
        params.clone()
    } else {
        return Err(GlutenError::Str("illegal macro params".to_string()));
    };
    let body: Vec<R<Val>> = vec.iter().skip(2).cloned().collect();
    let mac = r(Val::Macro(Macro(Box::new(move |env: &mut Env, args: Vec<R<Val>>| {
        let mut env = env.child();
        for (rs, val) in params.iter().zip(args.iter()) {
            if let Val::Symbol(_) = *rs.borrow() {
                env.insert(rs.clone(), val.clone());
                continue;
            }
            return Err(GlutenError::Str("illegal macro".to_string()));
        }
        eval_iter(&mut env, &mut body.iter())
    }))));
    env.insert(vec[0].clone(), mac.clone());
    Ok(mac)
}

pub fn eval_iter<'a>(env: &mut Env, iter: &mut impl Iterator<Item=&'a R<Val>>) -> Result<R<Val>, GlutenError> {
    let mut ret = r(Val::Vec(Vec::new()));
    for val in iter {
        ret = env.eval(val.clone())?;
    }
    Ok(ret)
}
