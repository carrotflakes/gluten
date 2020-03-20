use std::rc::Rc;
use std::str::CharIndices;
use std::iter::Peekable;
use crate::data::*;
use crate::error::GlutenError;
use crate::reader::{Reader, skip_whitespace};

pub struct SRange {
    pub start: i32,
    pub end: i32
}

pub struct SyntaxTreeReader {
    reader: Reader
}

impl SyntaxTreeReader {
    pub fn new(mut reader: Reader) -> Self {
        let read_fn = reader.read_fn.clone();
        reader.read_fn = Rc::new(move |reader: &mut Reader, cs: &mut Peekable<CharIndices>| {
            skip_whitespace(cs);
            let mut start_cs = cs.clone();
            let val = read_fn(reader, cs)?;
            let start = start_cs.peek().unwrap().0 as i32;
            let end = cs.peek().map(|x| x.0 as i32).unwrap_or(start + start_cs.count() as i32);
            Ok(r(Meta(val, Box::new(SRange {start, end}))))
        });
        SyntaxTreeReader {
            reader
        }
    }

    pub fn parse_top_level(&mut self, src: &str) -> Result<Vec<Val>, GlutenError> {
        self.reader.parse_top_level(src)
    }
}
