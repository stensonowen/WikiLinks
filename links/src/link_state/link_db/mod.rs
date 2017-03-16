use std::path::Path;
use std::iter;

use super::link_data::IndexedEntry;
use super::{LinkState, LinkDb, new_logger};

pub mod parse;

impl LinkState<LinkDb> {
    pub fn new(pages_db: &Path, redir_db: &Path, links_db: &Path) -> Self {
        let root_log = new_logger();
        let db_log = root_log.new(o!(
                "pages" => format!("{}", pages_db.display()), 
                "redir" => format!("{}", redir_db.display()), 
                "links" => format!("{}", links_db.display())) );
        let db = parse::populate_db(pages_db, redir_db, links_db, db_log);
        LinkState { 
            size:       db.num_entries(),
            threads:    4,
            log:        root_log,
            state:      LinkDb {
                db: db,
            }
        }
    }
}

impl LinkDb {
    pub fn parts(self) -> 
        (Box<iter::Iterator<Item=IndexedEntry>>,
         Box<iter::Iterator<Item=(String,u32)>>) 
    {
        self.db.explode()
    }
}

