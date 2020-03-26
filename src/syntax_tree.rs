use std::rc::Rc;
use std::str::CharIndices;
use std::iter::Peekable;
use crate::data::*;
use crate::reader::{Reader, skip_whitespace};

pub struct SRange {
    pub start: i32,
    pub end: i32
}

pub fn make_syntax_tree_reader(mut reader: Reader) -> Reader {
    let read_fn = reader.read_fn.clone();
    reader.read_fn = Rc::new(move |reader: &mut Reader, cs: &mut Peekable<CharIndices>| {
        skip_whitespace(cs);
        let mut start_cs = cs.clone();
        let val = read_fn(reader, cs)?;
        let start = start_cs.peek().unwrap().0 as i32;
        let end = cs.peek().map(|x| x.0 as i32).unwrap_or(start + start_cs.count() as i32);
        Ok(r(Meta(val, Box::new(SRange {start, end}))))
    });
    reader
}
