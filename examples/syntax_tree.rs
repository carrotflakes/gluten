extern crate gluten;

use gluten::{
    data::{Val, ValInterface},
    reader::Reader,
    syntax_tree::{make_syntax_tree_reader, SRange}
};

fn show_node(val: &Val, src: &str) {
    if let Some(sr) = val.get_meta::<SRange>() {
        println!(r#""{}""#, &src[sr.start as usize..sr.end as usize]);
    }
    if let Some(vec) = val.ref_as::<Vec<Val>>() {
        println!("(");
        for v in vec {
            show_node(v, src);
        }
        println!(")");
    }
}

fn main() {
    let mut st_reader = make_syntax_tree_reader(Reader::default());
    let src = r#"
    (+ 1 2 ( f hoge ) "nyan" 'fuga `(a b) '`'"(^_^)")
    "#;
    show_node(&st_reader.parse_top_level(src).unwrap()[0], src);
}
