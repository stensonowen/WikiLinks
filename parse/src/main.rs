use std::io::BufReader;
use std::fs::File;

extern crate parse;

fn main() {
    let filename = "/home/owen/shared/code/rust/wikilinks/data/simplewiki-20161201-page.sql";
    println!("Opening `{}`...", filename);
    let f = File::open(filename).unwrap();
    let r = BufReader::new(f);
    parse::parse_pages_regex_lossy(r, true);
}
