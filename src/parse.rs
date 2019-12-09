use std::str::Chars;
use crate::data::*;

pub fn parse(src: &str) -> Result<R<V>, String> {
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

fn parse_value<'a>(cs: &Chars<'a>) -> Result<(R<V>, Chars<'a>), String> {
    let mut cs = skip_whitespace(cs);
    match cs.next() {
        Some('(') => {
            let mut vec = vec![];
            loop {
                match parse_value(&cs) {
                    Ok((rv, ncs)) => {
                        vec.push(rv);
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
        Some(c) if c.is_alphanumeric() || c == '_' => {
            let mut vec = vec![c];
            let mut ncs = cs.clone();
            loop {
                match ncs.next() {
                    Some(c) if c.is_alphanumeric() || c == '_' => {
                        vec.push(c);
                        cs = ncs.clone();
                    },
                    _ => {
                        break;
                    }
                }
            }
            let s: String = vec.iter().collect();
            match s.as_str() {
                "true" => Ok((r(true), cs)),
                "false" => Ok((r(false), cs)),
                _ => Ok((r(s), cs))
            }
        },
        _ => Err("unexpected EOS".to_string())
    }
}
