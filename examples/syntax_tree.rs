extern crate gluten;

use gluten::{
    reader::Reader,
    syntax_tree::{Node, SyntaxTreeReader}
};

fn show_node(node: &Node, src: &str) {
    match node {
        Node::Atom { start, end, .. } => {
            println!("atom: {}", &src[*start as usize..*end as usize]);
        }
        Node::List { children, start, end } => {
            println!("list: {}", &src[*start as usize..*end as usize]);
            for child in children {
                show_node(child, src);
            }
        }
        Node::Untracked { .. } => {
            println!("untracked");
        }
    }
}

fn main() {
    let mut st_reader = SyntaxTreeReader::new(Reader::default());
    let src = r#"
    (+ 1 2 ( f hoge ) "nyan" 'fuga `(a b) '`'"(^_^)")
    "#;
    show_node(&st_reader.parse_top_level(src).unwrap()[0], src);
}
