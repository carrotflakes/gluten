use std::str::Chars;
use std::iter::Peekable;
use std::collections::HashMap;
use crate::data::*;

pub type AtomReader = Box<dyn FnMut(String) -> Result<Val, String>>;

pub struct Reader {
    atom_reader: AtomReader
}

impl Reader {
    pub fn new(atom_reader: AtomReader) -> Self {
        Reader {atom_reader}
    }

    pub fn parse(&mut self, src: &str) -> Result<Val, String> {
        self.parse_value(src.chars().peekable())
            .and_then(|mut x| {
                skip_whitespace(&mut x.1);
                if x.1.count() == 0 {
                    Ok(x.0)
                } else {
                    Err("expect EOS, but found some character".to_string())
                }
            })
    }

    pub fn parse_top_level(&mut self, src: &str) -> Result<Vec<Val>, String> {
        let mut vec = Vec::new();
        let mut cs = src.chars().peekable();
        loop {
            match self.parse_value(cs) {
                Ok((val, ncs)) => {
                    vec.push(val);
                    cs = ncs;
                    skip_whitespace(&mut cs);
                    if let None = cs.peek() {
                        return Ok(vec);
                    }
                },
                Err(s) => {
                    return Err(s);
                }
            }
        }
    }
    
    fn parse_value<'a>(&mut self, mut cs: Peekable<Chars<'a>>) -> Result<(Val, Peekable<Chars<'a>>), String> {
        skip_whitespace(&mut cs);
        match cs.next() {
            Some('(') => {
                let mut vec = vec![];
                loop {
                    match self.parse_value(cs.clone()) {
                        Ok((val, ncs)) => {
                            vec.push(val);
                            cs = ncs;
                        },
                        _ => {
                            break;
                        }
                    }
                }
                skip_whitespace(&mut cs);
                while let Some(c) = cs.next() {
                    if c == ')' {
                        break;
                    }
                    return Err("closing parenthesis missing".to_string());
                }
                Ok((r(vec), cs))
            },
            Some('\'') => {
                let (val, ncs) = self.parse_value(cs)?;
                let quote = (self.atom_reader)("quote".to_string()).unwrap();
                Ok((r(vec![quote, val]), ncs))
            },
            Some('"') => {
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
                                    return Err("expect a charactor but found EOS".to_string());
                                }
                            }
                        },
                        Some(c) => {
                            vec.push(c);
                        },
                        None => {
                            return Err("closing doublequote".to_string());
                        }
                    }
                }
                let s: String = vec.iter().collect();
                Ok((r(s), cs))
            },
            Some(c) if !c.is_whitespace() && !['(', ')', '\'', '"'].contains(&c) => {
                let mut vec = vec![c];
                loop {
                    match cs.peek() {
                        Some(c) if !c.is_whitespace() && !['(', ')', '\'', '"'].contains(c) => {
                            vec.push(*c);
                            cs.next();
                        },
                        _ => {
                            break;
                        }
                    }
                }
                let s: String = vec.iter().collect();
                (self.atom_reader)(s).map(|val| (val, cs))
            },
            Some(c) => Err(format!("unexpected character: {:?}", c)),
            None => Err("unexpected EOS".to_string())
        }
    }
}

impl Default for Reader {
    fn default() -> Self {
        Reader::new(make_default_atom_reader())
    }
}


fn skip_whitespace<'a> (cs: &mut Peekable<Chars<'a>>) {
    while let Some(c) = cs.peek() {
        if c.is_whitespace() {
            cs.next();
        } else {
            return;
        }
    }
}

pub fn make_default_atom_reader() -> AtomReader {
    let mut symbol_table = HashMap::<String, Val>::new();
    Box::new(move |s: String| -> Result<Val, String> {
        Ok(
            if let Some(symbol) = symbol_table.get(&s) {
                symbol.clone()
            } else {
                let symbol = r(Symbol(s.clone()));
                symbol_table.insert(s, symbol.clone()); // TODO: use weak!
                symbol
            }
        )
    })
}
