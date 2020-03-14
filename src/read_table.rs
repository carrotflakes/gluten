use std::rc::Rc;
use std::str::Chars;
use std::iter::Peekable;
use std::collections::HashMap;
use crate::data::*;
use crate::reader::{Reader, skip_whitespace};
use crate::error::GlutenError;

pub type ReadTable = HashMap<char, Rc<dyn Fn(&mut Reader, &mut Peekable<Chars>) -> Result<Val, GlutenError>>>;

pub fn read_list(reader: &mut Reader, cs: &mut Peekable<Chars>) -> Result<Val, GlutenError> {
    let mut vec = vec![];
    while let Ok(val) = reader.parse_value(cs) {
        vec.push(val);
    }
    skip_whitespace(cs);
    if let Some(')') = cs.next() {
        Ok(r(vec))
    } else {
        Err(GlutenError::ReadFailed("closing parenthesis missing".to_string()))
    }
}

pub fn read_quote(reader: &mut Reader, cs: &mut Peekable<Chars>) -> Result<Val, GlutenError> {
    let val = reader.parse_value(cs)?;
    let quote = r(reader.intern("quote"));
    Ok(r(vec![quote, val]))
}

pub fn read_backquote(reader: &mut Reader, cs: &mut Peekable<Chars>) -> Result<Val, GlutenError> {
    let val = reader.parse_value(cs)?;
    let quasiquote = r(reader.intern("quasiquote"));
    Ok(r(vec![quasiquote, val]))
}
pub fn read_comma(reader: &mut Reader, cs: &mut Peekable<Chars>) -> Result<Val, GlutenError> {
    let op = if cs.peek() == Some(&'@') {
        cs.next();
        "unquote-splicing"
    } else {
        "unquote"
    };
    let op = r(reader.intern(op));
    let val = reader.parse_value(cs)?;
    Ok(r(vec![op, val]))
}

pub fn read_string(_reader: &mut Reader, cs: &mut Peekable<Chars>) -> Result<Val, GlutenError> {
    let mut vec = Vec::new();
    loop {
        match cs.next() {
            Some(c) if c == '"' => {
                break;
            },
            Some(c) if c == '\\' => {
                match cs.next() {
                    Some(c) => {
                        vec.push(match c {
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            _ => c
                        });
                    },
                    None => {
                        return Err(GlutenError::ReadFailed("expect a charactor but found EOS".to_string()));
                    }
                }
            },
            Some(c) => {
                vec.push(c);
            },
            None => {
                return Err(GlutenError::ReadFailed("closing doublequote".to_string()));
            }
        }
    }
    let s: String = vec.iter().collect();
    Ok(r(s))
}

pub fn make_default_read_table() -> ReadTable {
    let mut rt = ReadTable::new();
    rt.insert('(', Rc::new(read_list));
    rt.insert('\'', Rc::new(read_quote));
    rt.insert('`', Rc::new(read_backquote));
    rt.insert(',', Rc::new(read_comma));
    rt.insert('"', Rc::new(read_string));
    rt
}
