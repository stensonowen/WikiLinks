extern crate parse;

#[macro_use]
extern crate slog;
extern crate slog_term;
use slog::DrainExt;

extern crate phf;
extern crate phf_codegen;
//use std::fs::File;
//use std::io::{BufWriter, Write};
use std::path::Path;

//        include!("/tmp/phf/codegen.rs");

fn main() {
    let path = Path::new("../wikidata/src");
    assert!(path.exists());

    let drain = slog_term::streamer().compact().build().fuse();
    let root_log = slog::Logger::root(drain, o!() );
    info!(root_log, "Entering `main`");

    let pages  = "/home/owen/shared/code/rust/wikilinks/data/simplewiki-20161201-page.sql";
    let redirs = "/home/owen/shared/code/rust/wikilinks/data/simplewiki-20161201-redirect.sql";
    let links  = "/home/owen/shared/code/rust/wikilinks/data/simplewiki-20161201-pagelinks.sql";

    let db = parse::populate_db(pages, redirs, links, &root_log);
    db.print();

    //let path = Path::new("/home/owen/shared/code/rust/wikilinks/data/kodegen/src");
    db.codegen(&path);
    //db.codegen_links(&path);
    //db.codegen_entries(&path);
    //db.codegen_addresses(&path);
    //db.fattest();
    /*
    let create: bool = false;

    if create {
        println!("Creating codegen.rs");

        //let path = Path::new(".").join("codegen.rs");
        let path = Path::new("/tmp/phf").join("codegen.rs");
        let mut file = BufWriter::new(File::create(&path).unwrap());
        write!(&mut file, "static KEYWORDS: phf::Map<&'static str, Keyword> = ").unwrap();
        phf_codegen::Map::new()
            .entry("loop", "Keyword::Loop")
            .entry("LOOP", "Keyword::Loop")
            .entry("continue", "Keyword::Continue")
            .entry("break", "Keyword::Break")
            .entry("fn", "Keyword::Fn")
            .entry("extern", "Keyword::Extern")
            .build(&mut file)
            .unwrap();
        write!(&mut file, ";\n").unwrap();

    } else {
        //include!("/tmp/phf/codegen.rs");
        //include!("../codegen.rs");
        //include!(concat!(".", "/codegen.rs"));
        //println!(" len keywds: {}", KEYWORDS.len());
        //pub fn parse_keyword(keyword: &str) -> Option<Keyword> {
        //        KEYWORDS.get(keyword).cloned()
        //}
    }

    */
}


/*
#[derive(Clone)]
enum Keyword {
    Loop,
    Continue,
    Break,
    Fn,
    Extern,
}*/
