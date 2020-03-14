use crate::data::*;
use crate::env::Env;
use crate::core::{eval, eval_iter};
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
		let cond = eval(env.clone(), vec[1].clone())?;
		eval(env.clone(), if let Some(false) = cond.downcast_ref::<bool>() {
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
		if let Some(v) = vec[1].downcast_ref::<Vec<Val>>() {
			let mut env = env.child();
			for val in v.iter() {
				if let Some(v) = val.downcast_ref::<Vec<Val>>() {
					if let Some(s) = v[0].downcast_ref::<Symbol>() {
						let val = eval(env.clone(), v[1].clone())?;
						env.insert(s.clone(), val);
						continue;
					}
				}
				return Err(GlutenError::Str("illegal let".to_string()));
			}
			return eval_iter(env, &mut vec.iter().skip(2));
		}
	}
	Err(GlutenError::Str(format!("invalid arguments")))
}

pub fn r#do(env: &mut Env, vec: &Vec<Val>) -> Result<Val, GlutenError> {
	eval_iter(env.clone(), &mut vec.iter().skip(1))
}

pub fn lambda(env: &mut Env, vec: &Vec<Val>) -> Result<Val, GlutenError> {
	let params = if let Some(params) = vec[1].downcast_ref::<Vec<Val>>() {
		params.clone()
	} else {
		return Err(GlutenError::Str("illegal lambda params".to_string()));
	};
	let body: Vec<Val> = vec.iter().skip(2).map(|val| val.clone()).collect();
	let env = env.clone();
	Ok(r(Box::new(move |args: Vec<Val>| {
		let mut env = env.child();
		for (rs, val) in params.iter().zip(args.iter()) {
			if let Some(s) = (*rs).downcast_ref::<Symbol>() {
				env.insert(s.clone(), val.clone());
				continue;
			}
			panic!("illegal lambda");
		}
		eval_iter(env, &mut body.iter())
	}) as NativeFn))
}

pub fn set(env: &mut Env, vec: &Vec<Val>) -> Result<Val, GlutenError> {
	if vec.len() == 3 {
		if let Some(name) = vec[1].downcast_ref::<Symbol>() {
			let val = eval(env.clone(), vec[2].clone())?;
			env.insert(name.clone(), val.clone());
			return Ok(val);
		}
	}
	Err(GlutenError::Str("illegal set".to_string()))
}

pub fn insert_all(env: &mut Env) {
	let reader = env.reader();
	env.insert(reader.borrow_mut().intern("quote"), r(Box::new(quote) as SpecialOperator));
	env.insert(reader.borrow_mut().intern("if"), r(Box::new(r#if) as SpecialOperator));
	env.insert(reader.borrow_mut().intern("let"), r(Box::new(r#let) as SpecialOperator));
	env.insert(reader.borrow_mut().intern("do"), r(Box::new(r#do) as SpecialOperator));
	env.insert(reader.borrow_mut().intern("lambda"), r(Box::new(lambda) as SpecialOperator));
	env.insert(reader.borrow_mut().intern("set"), r(Box::new(set) as SpecialOperator));
}
