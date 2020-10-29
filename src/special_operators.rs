use crate::data::*;
use crate::env::Env;
use crate::macros::eval_iter;
use crate::error::GlutenError;

pub fn quote(_env: &mut Env, vec: &Vec<R<Val>>) -> Result<R<Val>, GlutenError> {
	if vec.len() == 2 {
		Ok(vec[1].clone())
	} else {
		Err(GlutenError::Str(format!("invalid arguments")))
	}
}

pub fn r#if(env: &mut Env, vec: &Vec<R<Val>>) -> Result<R<Val>, GlutenError> {
	if vec.len() == 4 {
		let cond = env.eval(vec[1].clone())?;
		let body = if let Val::False = *cond.borrow() {
			vec[3].clone()
		} else {
			vec[2].clone()
		};
		env.eval(body)
	} else {
		Err(GlutenError::Str(format!("invalid arguments")))
	}
}

pub fn r#let(env: &mut Env, vec: &Vec<R<Val>>) -> Result<R<Val>, GlutenError> {
	if vec.len() >= 2 {
		if let Val::Vec(ref v) = *vec[1].borrow() {
			let mut env = env.child();
			for val in v.iter() {
				if let Val::Vec(ref v) = *val.borrow() {
					if let Val::Symbol(_) = *v[0].borrow() {
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

pub fn r#do(env: &mut Env, vec: &Vec<R<Val>>) -> Result<R<Val>, GlutenError> {
	eval_iter(env, &mut vec.iter().skip(1))
}

pub fn lambda(env: &mut Env, vec: &Vec<R<Val>>) -> Result<R<Val>, GlutenError> {
	let params = if let Val::Vec(ref params) = *vec[1].borrow() {
		params.clone()
	} else {
		return Err(GlutenError::Str("illegal lambda params".to_string()));
	};
	let body: Vec<R<Val>> = vec.iter().skip(2).map(|val| val.clone()).collect();
	let env = env.clone();
	Ok(r(Val::Fn(Box::new(move |args: Vec<R<Val>>| {
		let mut env = env.child();
		for (rs, val) in params.iter().zip(args.iter()) {
			if let Val::Symbol(_) = *rs.borrow() {
				env.insert(rs.clone(), val.clone());
				continue;
			}
			panic!("illegal lambda");
		}
		eval_iter(&mut env, &mut body.iter())
	}))))
}

pub fn set(env: &mut Env, vec: &Vec<R<Val>>) -> Result<R<Val>, GlutenError> {
	if vec.len() == 3 {
		if let Val::Symbol(_) = *vec[1].borrow() {
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
	env.insert(package.intern(&"quote".to_string()), r(Val::SpecialOp(Box::new(quote))));
	env.insert(package.intern(&"if".to_string()), r(Val::SpecialOp(Box::new(r#if) as SpecialOperator)));
	env.insert(package.intern(&"let".to_string()), r(Val::SpecialOp(Box::new(r#let) as SpecialOperator)));
	env.insert(package.intern(&"do".to_string()), r(Val::SpecialOp(Box::new(r#do) as SpecialOperator)));
	env.insert(package.intern(&"lambda".to_string()), r(Val::SpecialOp(Box::new(lambda) as SpecialOperator)));
	env.insert(package.intern(&"set".to_string()), r(Val::SpecialOp(Box::new(set) as SpecialOperator)));
}
