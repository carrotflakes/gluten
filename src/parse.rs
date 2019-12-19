use std::str::Chars;
use crate::data::*;

pub fn parse(src: &str) -> Result<Val, String> {
    parse_value(&src.chars()).map(|x| x.0)
}

fn skip_whitespace<'a> (cs: &Chars<'a>) -> Chars<'a> {
    let mut ncs = cs.clone();
    match ncs.next() {
        Some(c) if c.is_whitespace() =>
            skip_whitespace(&ncs),
        _ => cs.clone()
    }
}

fn parse_value<'a>(cs: &Chars<'a>) -> Result<(Val, Chars<'a>), String> {
    let mut cs = skip_whitespace(cs);
    match cs.next() {
        Some('(') => {
            let mut vec = vec![];
            loop {
                match parse_value(&cs) {
                    Ok((val, ncs)) => {
                        vec.push(val);
                        cs = ncs.clone();
                    },
                    _ => {
                        break;
                    }
                }
            }
            cs = skip_whitespace(&cs);
            while let Some(c) = cs.next() {
                if c == ')' {
                    break;
                }
                return Err("closing parenthesis missing".to_string());
            }
            Ok((r(vec), cs))
        },
        Some('\'') => {
            let (val, ncs) = parse_value(&cs)?;
            Ok((r(vec![r("quote".to_string()) as Val, val]), ncs))
        },
        Some(c) if c.is_alphanumeric() || c == '_' || c == '.' => {
            let mut vec = vec![c];
            let mut ncs = cs.clone();
            loop {
                match ncs.next() {
                    Some(c) if c.is_alphanumeric() || c == '_' || c == '.' => {
                        vec.push(c);
                        cs = ncs.clone();
                    },
                    _ => {
                        break;
                    }
                }
            }
            let s: String = vec.iter().collect();
            Ok((r(s), cs))
        },
        _ => Err("unexpected EOS".to_string())
    }
}
