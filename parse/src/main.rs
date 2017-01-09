extern crate parse;

#[macro_use]
extern crate slog;
extern crate slog_term;
use slog::DrainExt;

extern crate phf;
extern crate phf_codegen;
use std::path::Path;
use std::env;

const USAGE: &'static str = "USAGE: ./parse PAGE.SQL REDIRECT.SQL PAGELINKS.SQL OUT_DIRECTORY";

fn main() {
    //set up logging
    let drain = slog_term::streamer().compact().build().fuse();
    let root_log = slog::Logger::root(drain, o!() );
    info!(root_log, "Parsing Args...");

    //get input / output paths
    let pages =  env::args().nth(1).expect(USAGE);
    let redirs = env::args().nth(2).expect(USAGE);
    let links =  env::args().nth(3).expect(USAGE);
    let out_dir =env::args().nth(4).expect(USAGE);
    let path = Path::new(&out_dir);
    assert!(path.is_dir(), "OUT_DIR must be valid directory");

    info!(root_log, "Beginning database population`");
    let db = parse::populate_db(pages, redirs, links, &root_log);
    db.print();
    db.codegen(&path);
}

