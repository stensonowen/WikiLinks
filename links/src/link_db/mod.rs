use std::path::{PathBuf};

use super::slog::Logger;
use super::LinkDb;
pub mod parse;


impl LinkDb {
    pub fn new(p: PathBuf, r: PathBuf, l: PathBuf, log: Logger) -> Self {
        LinkDb {
            //db_pages:    Path::new(&pgs).to_path_buf(),
            //db_redirect: Path::new(&rdr).to_path_buf(),
            //db_pagelinks: Path::new(&pl).to_path_buf(),
            //simple_wiki: smp,
            db: parse::populate_db(p, r, l, log),
        }
    }
}
