use std::path::Path;

use super::slog::Logger;
use super::LinkDb;
pub mod parse;


impl LinkDb {
    fn new(pgs: String, rdr: String, pl: String, smp: bool, log: &Logger) -> LinkDb {
        LinkDb {
            //db_pages:    Path::new(&pgs).to_path_buf(),
            //db_redirect: Path::new(&rdr).to_path_buf(),
            //db_pagelinks: Path::new(&pl).to_path_buf(),
            //simple_wiki: smp,
            db: parse::populate_db(pgs, rdr, pl, log),
        }
    }
}
