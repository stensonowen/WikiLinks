use std::path::{PathBuf};
use std::iter;
use Entry;

use super::slog::Logger;
use super::LinkDb;
pub mod parse;



impl LinkDb {
    pub fn new(p: PathBuf, r: PathBuf, l: PathBuf, log: Logger) -> Self {
        LinkDb {
            db: parse::populate_db(p, r, l, log),
        }
    }
    pub fn size(&self) -> usize {
        self.db.num_entries()
    }
    pub fn parts(self) -> 
        (Box<iter::Iterator<Item=(u32,Entry)>>,
         Box<iter::Iterator<Item=(String,u32)>>) 
    {
        self.db.explode()
    }
}


