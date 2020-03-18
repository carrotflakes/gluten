#[macro_use]
extern crate gluten;

use gluten::{
    data::*,
    reader::Reader,
};

fn main() {
    let mut reader = Reader::default();
    let f = |v: Val| {
        destruct! {
            (sequencer (vec {x:Symbol})) = v =>
            println!("{:?}", x);
            return;
        }
        println!("not match");
    };
    f(reader.parse("sequencer").unwrap());
    f(reader.parse("(sequencer (vec hoge))").unwrap());
    f(reader.parse("(sequencer (vec))").unwrap());
    f(reader.parse("(sequencer (vec hoge fuga))").unwrap());
    f(reader.parse("(sequencer (vec hoge) fuga)").unwrap());
}
