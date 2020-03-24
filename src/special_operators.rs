use crate::data::*;
use crate::env::Env;
use crate::macros::eval_iter;
use crate::error::GlutenError;

pub fn quote(_env: &mut Env, vec: &Vec<Val>) -> Result<Val, GlutenError> {
	if vec.len() == 2 {
		Ok(vec[1].clone())
	} else {
		Err(GlutenError::Str(format!("invalid arguments")))
	}
}

pub fn r#if(env: &mut Env, vec: &Vec<Val>) -> Result<Val, GlutenError> {
	if vec.len() == 4 {
		let cond = env.eval(vec[1].clone())?;
		env.eval(if let Some(false) = cond.ref_as::<bool>() {
			vec[3].clone()
		} else {
			vec[2].clone()
		})
	} else {
		Err(GlutenError::Str(format!("invalid arguments")))
	}
}

pub fn r#let(env: &mut Env, vec: &Vec<Val>) -> Result<Val, GlutenError> {
	if vec.len() >= 2 {
		if let Some(v) = vec[1].ref_as::<Vec<Val>>() {
			let mut env = env.child();
			for val in v.iter() {
				if let Some(v) = val.ref_as::<Vec<Val>>() {
					if v[0].is::<Symbol>() {
						let val = env.eval(v[1].clone())?;
						env.insert(v[0].clone(), val);
						continue;
					}
				}
				return Err(GlutenError::Str("illegal let".to_string()));
			}
			return eval_iter(&mut env, &mut vec.iter().skip(2));
		}
	}
	Err(GlutenError::Str(format!("invalid arguments")))
}

pub fn r#do(env: &mut Env, vec: &Vec<Val>) -> Result<Val, GlutenError> {
	eval_iter(env, &mut vec.iter().skip(1))
}

pub fn lambda(env: &mut Env, vec: &Vec<Val>) -> Result<Val, GlutenError> {
	let params = if let Some(params) = vec[1].ref_as::<Vec<Val>>() {
		params.clone()
	} else {
		return Err(GlutenError::Str("illegal lambda params".to_string()));
	};
	let body: Vec<Val> = vec.iter().skip(2).map(|val| val.clone()).collect();
	let env = env.clone();
	Ok(r(Box::new(move |args: Vec<Val>| {
		let mut env = env.child();
		for (rs, val) in params.iter().zip(args.iter()) {
			if rs.is::<Symbol>() {
				env.insert(rs.clone(), val.clone());
				continue;
			}
			panic!("illegal lambda");
		}
		eval_iter(&mut env, &mut body.iter())
	}) as NativeFn))
}

pub fn set(env: &mut Env, vec: &Vec<Val>) -> Result<Val, GlutenError> {
	if vec.len() == 3 {
		if vec[1].is::<Symbol>() {
			let val = env.eval(vec[2].clone())?;
			env.insert(vec[1].clone(), val.clone());
			return Ok(val);
		}
	}
	Err(GlutenError::Str("illegal set".to_string()))
}

pub fn insert_all(env: &mut Env) {
	let reader = env.reader();
	let mut reader = reader.borrow_mut();
	let package = &mut reader.package;
	env.insert(package.intern(&"quote".to_string()), r(Box::new(quote) as SpecialOperator));
	env.insert(package.intern(&"if".to_string()), r(Box::new(r#if) as SpecialOperator));
	env.insert(package.intern(&"let".to_string()), r(Box::new(r#let) as SpecialOperator));
	env.insert(package.intern(&"do".to_string()), r(Box::new(r#do) as SpecialOperator));
	env.insert(package.intern(&"lambda".to_string()), r(Box::new(lambda) as SpecialOperator));
	env.insert(package.intern(&"set".to_string()), r(Box::new(set) as SpecialOperator));
}
