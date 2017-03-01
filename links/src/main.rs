#![allow(dead_code)]

// Order
//  0.  LinkDb
//          field:      handles to files
//          function:   process collisions
//                          also compute pagerank data
//                          okay to have no IR between db and ranks?
//          create_from: databases
//
//  1.  LinkData
//          field:      vector of vectors of article entry
//                          thread-safe: can be populated concurrently
//                          easily write to or read from disk
//          field:      hash table of address lookups
//                          easily dump to or read from disk
//          convert:
//                      turn 2d vec into one hash table
//                      turn address data into PostgreSQL database?
//          create_from: (`num_threads` files AND addresses dump) OR LinkDb
//
//  2.  RankData
//          field:      hashmap of vectors: thread safe only for reading 
//          field:      page ranks
//                          easily dump to or read from disk
//          create_from: (LinkData AND addr dump) OR (LinkData AND pagerank dump)
//
//  3.  HashLinks
//          field:      one hash table of links 
//          function:   search
//
//  reference: https://hoverbear.org/2016/10/12/rust-state-machine-pattern/

// NOTE: when scaling, remember to change link_db/parse/regexes.rs/IS_SIMPLE

#[macro_use] 
extern crate slog;
extern crate slog_term;
use slog::DrainExt;

#[macro_use] 
extern crate serde_derive;
extern crate serde_json;
extern crate csv;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{/*Arc,*/Mutex};

//mod parse;
//mod pagerank;
pub mod link_db;
pub mod link_data;
pub mod rank_data;
use rank_data::RankedEntry;

const IS_SIMPLE: bool = true;

//  ------STATE--MACHINE------

trait State { }
impl State for LinkDb { }
impl State for LinkData { }
impl State for RankData { }
impl State for HashLinks { }

struct LinkState<S: State> {
    //shared vars go here
    threads: usize,     // number of threads/files to use concurrently
    size: usize,        // number of entries
    log: slog::Logger,  // root logger that will be split off for components
    state: S,           // one of 4 values that represent development of the data
}

impl LinkState<LinkDb> {
    fn new(pages_db: PathBuf, redir_db: PathBuf, links_db: PathBuf) -> Self {
        let drain = slog_term::streamer().compact().build().fuse();
        let root_log = slog::Logger::root(drain, o!() );
        let db_log = root_log.new(o!(
                "pages" => format!("{}", pages_db.display()), 
                "redir" => format!("{}", redir_db.display()), 
                "links" => format!("{}", links_db.display())) );
        let link_db = LinkDb::new(pages_db, redir_db, links_db, db_log);
        LinkState { 
            size:       0,
            threads:    4,
            log:        root_log,
            state:      link_db,
        }
    }
}


//  ----------STATES----------


struct LinkDb {
    db: link_db::parse::database::Database,
}

struct LinkData {
    //len is desired num of threads (num cpus?)
    dumps: Vec<Mutex<Vec<(u32,Entry)>>>, // Vec<Arc<Mutex<&u32>>>, //?
    //ranks: HashMap<u32,f64>,
    addrs: Vec<(String,u32)>,
}

struct RankData {
    links: HashMap<u32,Entry>,
    ranks: Vec<RankedEntry>,
}

struct HashLinks {
    links: HashMap<u32,Entry>,
    // database?
    // HashMap<String,u32> ?
}


// ------COMMON-OBJECTS------


#[derive(Serialize, Deserialize)]
pub struct Entry {
    title: String,
    parents:  Vec<u32>,
    pub children: Vec<u32>,
}


//  --------------------------


//don't need to represenent 
//enum Entry {
//    Page {
//        title: String,
//        children: Vec<u32>,
//        parents: Vec<u32>,
//    },
//    Redirect(u32),
//    Absent, //default? use?
//}

fn main() {
    println!("😄");
    let ls = LinkState::new(
        PathBuf::from("/home/owen/wikidata/simplewiki-20170201-page.sql"),
        PathBuf::from("/home/owen/wikidata/simplewiki-20170201-redirect.sql"),
        PathBuf::from("/home/owen/wikidata/simplewiki-20170201-pagelinks.sql"),
    );
}


