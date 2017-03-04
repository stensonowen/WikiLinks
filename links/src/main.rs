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

#[macro_use] 
extern crate clap;
extern crate fnv;

use std::collections::HashMap;
use std::path::{/*Path,*/ PathBuf};
//use std::path::Path;
use std::sync::{/*Arc,*/Mutex};

pub mod link_db;
pub mod link_data;
pub mod rank_data;
pub mod hash_links;

use link_data::IndexedEntry;
//use std::{thread, time};

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
        let root_log = new_logger();
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

fn new_logger() -> slog::Logger {
    let drain = slog_term::streamer().compact().build().fuse();
    slog::Logger::root(drain, o!())
}


//  ----------STATES----------


struct LinkDb {
    db: link_db::parse::database::Database,
}

struct LinkData {
    //len is desired num of threads (num cpus?)
    dumps: Vec<Mutex<Vec<IndexedEntry>>>,
    addrs: Vec<(String,u32)>,
}

struct RankData {
    links: fnv::FnvHashMap<u32,Entry>,
    ranks: HashMap<u32, f64>,
}

struct HashLinks {
    links: fnv::FnvHashMap<u32,Entry>,
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

fn argv<'a>() -> clap::ArgMatches<'a> {
    //should be able to form HashLinks from...
    //  sql dumps only
    //  sql dumps and rank backup
    //  links backup only
    //  links backup and rank backup
    // need to provide:
    //  exactly one of { sql locations | manifest location }
    //  optional rank location
    clap::App::new(crate_name!())
        .about(crate_description!())
        .author(crate_authors!("\n"))
        .version(crate_version!())
        //.help("foo")
        .arg(clap::Arg::with_name("ranks")
             .short("r")
             .long("ranks_dump")
             .help("Supply location of rank data in csv form")
             .takes_value(true)
             )
        .arg(clap::Arg::with_name("manifest")
             .short("json")
             .help("Supply dump of parsed link data manifest in json form")
             .takes_value(true)
             )
        .arg(clap::Arg::with_name("page.sql")
             .short("pages")
             .takes_value(true)
             .conflicts_with("manifest")
             .requires("redirect.sql")
             .requires("pagelinks.sql")
             )
        .arg(clap::Arg::with_name("redirect.sql")
             .short("dirs")
             .takes_value(true)
             .conflicts_with("manifest")
             .requires("page.sql")
             .requires("pagelinks.sql")
             )
        .arg(clap::Arg::with_name("pagelinks.sql")
             .short("links")
             .takes_value(true)
             .conflicts_with("manifest")
             .requires("page.sql")
             .requires("redirect.sql")
             )
        .group(clap::ArgGroup::with_name("sources")
               .required(true)
               .args(&["sql", "manifest", "page.sql", "redirects.sql", "pagelink.sql"]))
        .get_matches()
}

fn main() {
    //let pages_db = PathBuf::from("/home/owen/wikidata/simplewiki-20170201-page.sql");
    //let redir_db = PathBuf::from("/home/owen/wikidata/simplewiki-20170201-redirect.sql");
    //let links_db = PathBuf::from("/home/owen/wikidata/simplewiki-20170201-pagelinks.sql");
    //let output = PathBuf::from("/home/owen/wikidata/dumps/simple_20170201_dump2");
    //let rank_file = Path::new("/home/owen/wikidata/dumps/simple_20170201_ranks1");
    //let p_rank_file = Path::new("/home/owen/wikidata/dumps/simple_20170201_pretty_ranks1");
    //let dump = PathBuf::from("/home/owen/wikidata/dumps/simple_20170201_dump1");

    //let ld = LinkState::<LinkData>::from_file(dump, new_logger()).unwrap();
    //let rd = LinkState::<RankData>::from_ranks(ld, rank_file);
    //rd.pretty_ranks(p_rank_file).unwrap();
    
    // ====

    let hl = LinkState::<HashLinks>::from_args(argv());
    println!("Size: {}", hl.size);

    println!("{:?}", hl.bfs(232327,460509));

    //thread::sleep(time::Duration::from_secs(30));
}
