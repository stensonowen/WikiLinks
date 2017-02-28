use std::path::{PathBuf};

use super::slog::Logger;
use super::LinkDb;
pub mod parse;


impl LinkDb {
    pub fn new(p: PathBuf, r: PathBuf, l: PathBuf, log: Logger) -> Self {
        LinkDb {
            db: parse::populate_db(p, r, l, log),
        }
    }
}
