use std::rc::Rc;
use std::str::Chars;
use std::iter::Peekable;
use crate::data::*;
use crate::error::GlutenError;
use crate::reader::{Reader, skip_whitespace};

#[derive(Debug)]
pub enum Node {
    Atom {
        val: Val,
        start: i32,
        end: i32
    },
    List {
        children: Vec<Node>,
        start: i32,
        end: i32
    },
    Untracked {
        val: Val
    }
}

#[derive(Debug)]
struct PositionalVal {
    pub val: Val,
    pub start: i32,
    pub end: i32
}

pub struct SyntaxTreeReader {
    reader: Reader
}

impl SyntaxTreeReader {
    pub fn new(mut reader: Reader) -> Self {
        let read_fn = reader.read_fn.clone();
        reader.read_fn = Rc::new(move |reader: &mut Reader, cs: &mut Peekable<Chars>| {
            skip_whitespace(cs);
            let start = -(cs.clone().count() as i32);
            let val = read_fn(reader, cs)?;
            let end = -(cs.clone().count() as i32);
            Ok(r(PositionalVal { val, start, end }))
        });
        SyntaxTreeReader {
            reader
        }
    }

    pub fn parse_top_level(&mut self, src: &str) -> Result<Vec<Node>, GlutenError> {
        let mut vec = Vec::new();
        let mut cs = src.chars().peekable();
        while cs.peek().is_some() {
            vec.push(self.reader.parse_value(&mut cs)?);
            skip_whitespace(&mut cs);
        }
        Ok(vec.iter().map(|val| nodize(val, src.len() as i32)).collect::<Vec<Node>>())
    }
}

fn nodize(val: &Val, src_len: i32) -> Node {
    if let Some(pval) = val.downcast_ref::<PositionalVal>() {
        if let Some(vec) = pval.val.downcast_ref::<Vec<Val>>() {
            Node::List {
                children: vec.into_iter().map(|val| nodize(val, src_len)).collect(),
                start: pval.start + src_len,
                end: pval.end + src_len,
            }
        } else {
            Node::Atom {
                val: pval.val.clone(),
                start: pval.start + src_len,
                end: pval.end + src_len,
            }
        }
    } else {
        Node::Untracked {
            val: val.clone()
        }
    }
}
