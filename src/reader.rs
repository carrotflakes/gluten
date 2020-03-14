use std::rc::Rc;
use std::str::Chars;
use std::iter::Peekable;
use std::collections::HashMap;
use crate::data::*;
use crate::string_pool::StringPool;
use crate::error::GlutenError;

pub type AtomReader = Box<dyn FnMut(&mut StringPool, &str) -> Result<Val, GlutenError>>;
pub type ReadTable = HashMap<char, Rc<dyn Fn(&mut Reader, &mut Peekable<Chars>) -> Result<Val, GlutenError>>>;

pub struct Reader {
    pub read_table: ReadTable,
    pub atom_reader: AtomReader,
    string_pool: StringPool
}

impl Reader {
    pub fn new(atom_reader: AtomReader) -> Self {
        Reader {
            read_table: make_default_read_table(),
            atom_reader,
            string_pool: StringPool::new()
        }
    }

    pub fn parse(&mut self, src: &str) -> Result<Val, GlutenError> {
        let mut cs = src.chars().peekable();
        self.parse_value(&mut cs)
            .and_then(|val| {
                skip_whitespace(&mut cs);
                if let None = cs.peek() {
                    Ok(val)
                } else {
                    Err(GlutenError::ReadFailed("expect EOS, but found some character".to_string()))
                }
            })
    }

    pub fn parse_top_level(&mut self, src: &str) -> Result<Vec<Val>, GlutenError> {
        let mut vec = Vec::new();
        let mut cs = src.chars().peekable();
        loop {
            match self.parse_value(&mut cs) {
                Ok(val) => {
                    vec.push(val);
                    skip_whitespace(&mut cs);
                    if let None = cs.peek() {
                        return Ok(vec);
                    }
                },
                Err(err) => {
                    return Err(err);
                }
            }
        }
    }

    fn parse_value(&mut self, cs: &mut Peekable<Chars>) -> Result<Val, GlutenError> {
        skip_whitespace(cs);
        if let Some(c) = cs.peek() {
            if let Some(f) = self.read_table.get(&c).cloned() {
                cs.next();
                f(self, cs)
            } else if !c.is_whitespace() && !['(', ')', '\'', '"', ';'].contains(&c) {
                let mut vec = vec![*c];
                cs.next();
                loop {
                    match cs.peek() {
                        Some(c) if !c.is_whitespace() && !['(', ')', '\'', '"', ';'].contains(c) => {
                            vec.push(*c);
                            cs.next();
                        },
                        _ => {
                            break;
                        }
                    }
                }
                let s: String = vec.iter().collect();
                (self.atom_reader)(&mut self.string_pool, &s).map(|val| val)
            } else {
                Err(GlutenError::ReadFailed(format!("unexpected character: {:?}", c)))
            }
        } else {
            Err(GlutenError::ReadFailed("unexpected EOS".to_string()))
        }
    }

    pub fn intern(&mut self, s: &str) -> Symbol {
        Symbol(self.string_pool.intern(s))
    }

    pub fn try_intern(&self, s: &str) -> Option<Symbol> {
        self.string_pool.try_intern(s).map(|s| Symbol(s))
    }
}

impl Default for Reader {
    fn default() -> Self {
        Reader::new(Box::new(default_atom_reader))
    }
}


fn skip_whitespace (cs: &mut Peekable<Chars>) {
    while let Some(c) = cs.peek() {
        if *c == ';' {
            while match cs.next() {
                    Some('\n') => false,
                    Some('\r') => false,
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

pub fn default_atom_reader(sp: &mut StringPool, s: &str) -> Result<Val, GlutenError> {
    Ok(r(Symbol(sp.intern(s))))
}

pub fn read_list(reader: &mut Reader, cs: &mut Peekable<Chars>) -> Result<Val, GlutenError> {
    let mut vec = vec![];
    while let Ok(val) = reader.parse_value(cs) {
        vec.push(val);
    }
    skip_whitespace(cs);
    if cs.next() == None {
        return Err(GlutenError::ReadFailed("closing parenthesis missing".to_string()));
    }
    Ok(r(vec))
}

pub fn read_quote(reader: &mut Reader, cs: &mut Peekable<Chars>) -> Result<Val, GlutenError> {
    let val = reader.parse_value(cs)?;
    let quote = r(Symbol(reader.string_pool.intern("quote")));
    Ok(r(vec![quote, val]))
}

pub fn read_backquote(reader: &mut Reader, cs: &mut Peekable<Chars>) -> Result<Val, GlutenError> {
    let val = reader.parse_value(cs)?;
    let quote = r(Symbol(reader.string_pool.intern("quasiquote")));
    Ok(r(vec![quote, val]))
}
pub fn read_comma(reader: &mut Reader, cs: &mut Peekable<Chars>) -> Result<Val, GlutenError> {
    let op = if cs.peek() == Some(&'@') {
        cs.next();
        "unquote-splicing"
    } else {
        "unquote"
    };
    let op = r(Symbol(reader.string_pool.intern(op)));
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
