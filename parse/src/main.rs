use std::io::BufReader;
use std::fs::File;

extern crate parse;

fn main() {
    //let filename = "/home/owen/shared/code/rust/wikilinks/data/simplewiki-20161201-page.sql";
    //let f = "/home/owen/shared/code/rust/wikilinks/data/simplewiki-20161201-pagelinks.sql";
    //let f = "/home/owen/shared/code/rust/wikilinks/data/simplewiki-20161201-page.sql";
    let f = "/home/owen/shared/code/rust/wikilinks/data/simplewiki-20161201-redirect.sql";
    println!("Opening `{}`...", f);
    let f = File::open(f).unwrap();
    let r = BufReader::new(f);
    //parse::parse_pagelinks_regex_lossy(r);
    //parse::parse_pages_regex_lossy(r, true);
    parse::parse_redirects_regex_lossy(r);
}
