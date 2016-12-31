extern crate parse;
#[macro_use]
extern crate slog;
extern crate slog_term;
use slog::DrainExt;


fn main() {
    let drain = slog_term::streamer().compact().build().fuse();
    let root_log = slog::Logger::root(drain, o!() );//o!("version" => "0.5"));
    //info!(server_log, "starting");
    info!(root_log, "entering `main`...");

    let pages  = "/home/owen/shared/code/rust/wikilinks/data/simplewiki-20161201-page.sql";
    let redirs = "/home/owen/shared/code/rust/wikilinks/data/simplewiki-20161201-redirect.sql";
    let links  = "/home/owen/shared/code/rust/wikilinks/data/simplewiki-20161201-pagelinks.sql";

    let mut db = parse::populate_db(pages, redirs, links, &root_log);
    //db.verify();
    db.print();
    println!();
    db.clean_up();
    println!();
    db.print();
}
