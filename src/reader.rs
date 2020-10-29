use std::rc::Rc;
use std::str::CharIndices;
use std::iter::Peekable;
use std::collections::HashMap;
use crate::data::*;
use crate::read_table::make_default_read_table;
use crate::error::GlutenError;

pub type AtomReader = Box<dyn FnMut(&mut Package, &String) -> Result<R<Val>, GlutenError>>;
pub type ReadFn = Rc<dyn Fn(&mut Reader, &mut Peekable<CharIndices>) -> Result<R<Val>, GlutenError>>;
pub type ReadTable = HashMap<char, ReadFn>;

pub struct Reader {
    pub read_table: ReadTable,
    pub atom_reader: AtomReader,
    pub read_fn: ReadFn,
    pub package: Package
}

impl Reader {
    pub fn new(atom_reader: AtomReader, package: Package) -> Self {
        Reader {
            read_table: make_default_read_table(),
            atom_reader,
            read_fn: Rc::new(default_read_fn),
            package
        }
    }

    pub fn parse(&mut self, src: &str) -> Result<R<Val>, GlutenError> {
        let mut cs = src.char_indices().peekable();
        self.parse_value(&mut cs)
            .and_then(|val| {
                skip_whitespace(&mut cs);
                if cs.peek().is_none() {
                    Ok(val)
                } else {
                    Err(GlutenError::ReadFailed("expect EOS, but found some character".to_string()))
                }
            })
    }

    pub fn parse_top_level(&mut self, src: &str) -> Result<Vec<R<Val>>, GlutenError> {
        let mut vec = Vec::new();
        let mut cs = src.char_indices().peekable();
        while cs.peek().is_some() {
            vec.push(self.parse_value(&mut cs)?);
            skip_whitespace(&mut cs);
        }
        Ok(vec)
    }

    pub fn parse_value(&mut self, cs: &mut Peekable<CharIndices>) -> Result<R<Val>, GlutenError> {
        self.read_fn.clone()(self, cs)
    }
}

impl Default for Reader {
    fn default() -> Self {
        Reader::new(Box::new(default_atom_reader), Package::new())
    }
}

pub fn default_read_fn(reader: &mut Reader, cs: &mut Peekable<CharIndices>) -> Result<R<Val>, GlutenError> {
    const EXCEPT_CHARS: &[char] = &['(', ')', '\'', '"', ';'];
    skip_whitespace(cs);
    if let Some((i, c)) = cs.peek().cloned() {
        if let Some(f) = reader.read_table.get(&c).cloned() {
            cs.next();
            f(reader, cs)
        } else if !c.is_whitespace() && !EXCEPT_CHARS.contains(&c) {
            cs.next();
            let mut vec = vec![c];
            while let Some((_, c)) = cs.peek() {
                if c.is_whitespace() || EXCEPT_CHARS.contains(c) {
                    break;
                }
                vec.push(*c);
                cs.next();
            }
            let s: String = vec.iter().collect();
            (reader.atom_reader)(&mut reader.package, &s)
        } else {
            Err(GlutenError::ReadFailed(format!("unexpected character: {:?} as {}", c, i)))
        }
    } else {
        Err(GlutenError::ReadFailed("unexpected EOS".to_string()))
    }
}

pub fn skip_whitespace (cs: &mut Peekable<CharIndices>) {
    while let Some((_, c)) = cs.peek() {
        if *c == ';' {
            while match cs.next() {
                    Some((_, '\n')) => false,
                    Some((_, '\r')) => false,
                    Some(_) => true,
                    None => false
                } {}
        } else if c.is_whitespace() {
            cs.next();
        } else {
            return;
        }
    }
}

pub fn default_atom_reader(package: &mut Package, s: &String) -> Result<R<Val>, GlutenError> {
    Ok(package.intern(s))
}
