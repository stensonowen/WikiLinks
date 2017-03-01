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

pub mod link_db;
pub mod link_data;
pub mod rank_data;
pub mod hash_links;

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
            size:       link_db.size(),
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
    //ranks: Vec<RankedEntry>,
    //ranks: Vec<(String,u32)>,
    ranks: HashMap<u32, f64>,
}

struct HashLinks {
    links: HashMap<u32,Entry>,
    // database?
    // HashMap<String,u32> ?
}


/*
enum Links {
    LinkDb(   LinkState<LinkDb>),
    LinkData( LinkState<LinkData>),
    RankData( LinkState<RankData>),
    HashLinks(LinkState<HashLinks>),
}

impl Links {
    fn from_sql(p: PathBuf, r: PathBuf, l: PathBuf) -> Self {
        Links::LinkDb(LinkState::new(p, r, l))
    }
    fn step(self) -> Links {
        match self {
            Links::LinkDb(ld)   => Links::LinkData(ld.into()),
            Links::LinkData(ld) => Links::RankData(ld.into()),
            Links::RankData(rd) => Links::HashLinks(rd.into()),
            Links::HashLinks(_) => panic!("Link already in its final state"),
        }
    }
}*/

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
    let pages_db = PathBuf::from("/home/owen/wikidata/simplewiki-20170201-page.sql");
    let redir_db = PathBuf::from("/home/owen/wikidata/simplewiki-20170201-redirect.sql");
    let links_db = PathBuf::from("/home/owen/wikidata/simplewiki-20170201-pagelinks.sql");
    /*
    println!("Parsing Db...");
    let links = Links::from_sql(pages_db, redir_db, links_db);
    println!("Creating Links...");
    let links = links.step();
    println!("Computing Pageranks...");
    let links = links.step();
    println!("Finalizing Data");
    let links = links.step();
    */
    println!("Parsing Db...");
    let ls_db = LinkState::new(pages_db, redir_db, links_db);
    println!("Creating Links...");
    let ls_ld: LinkState<LinkData> = ls_db.into(); 
    println!("Computing Pageranks...");
    let ls_rd: LinkState<RankData> = ls_ld.into(); 
    ls_rd.data();
    println!("Finalizing...");
    let _ls_hl: LinkState<HashLinks>= ls_rd.into(); 
    println!("Done");

}
