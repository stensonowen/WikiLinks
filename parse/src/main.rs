extern crate parse;
#[macro_use]
extern crate slog;
extern crate slog_term;
use slog::DrainExt;


fn main() {
    let drain = slog_term::streamer().compact().build().fuse();
    let root_log = slog::Logger::root(drain, o!() );
    info!(root_log, "Entering `main`");

    let pages  = "/home/owen/shared/code/rust/wikilinks/data/simplewiki-20161201-page.sql";
    let redirs = "/home/owen/shared/code/rust/wikilinks/data/simplewiki-20161201-redirect.sql";
    let links  = "/home/owen/shared/code/rust/wikilinks/data/simplewiki-20161201-pagelinks.sql";

    let db = parse::populate_db(pages, redirs, links, &root_log);
    //db.verify();
    db.print();
    //println!();
    //db.clean_up();
    //println!();
    //db.print();
}
