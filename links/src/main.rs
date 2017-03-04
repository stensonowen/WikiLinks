#![allow(dead_code)]
//#![feature(plugin, custom_derive, custom_attribute)]

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

// LOGGING
#[macro_use] 
extern crate slog;
extern crate slog_term;
use slog::{Logger, DrainExt};
// SERIALIZING
#[macro_use] 
extern crate serde_derive;
extern crate serde_json;
extern crate csv;
// MISC
#[macro_use] 
extern crate clap;
extern crate fnv;
extern crate chrono;
use clap::Arg;
// DATABASE
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate dotenv;
//use diesel::prelude::*;
//use diesel::pg::PgConnection;
//use dotenv::dotenv;

// STD
//use std::collections::HashMap;
use std::sync::Mutex;
//use std::path::PathBuf;
//use std::env;
//use std::path::Path;
//use std::sync::{/*Arc,*/Mutex};
//use std::{thread, time};

// HELPERS
pub mod link_db;
pub mod link_data;
pub mod rank_data;
pub mod hash_links;
pub mod cache;
//use link_data::IndexedEntry;

const IS_SIMPLE: bool = true;


//  ------STATE--MACHINE------


trait State { }
impl State for LinkDb { }
impl State for LinkData { }
impl State for RankData { }
impl State for HashLinks { }

struct LinkState<S: State> {
    threads: usize,     // number of threads/files to use concurrently
    size:    usize,     // number of entries
    log:     Logger,    // root logger that will be split off for components
    state:   S,         // 1 of 4 values that represent development of the data
}

fn new_logger() -> Logger {
    //let drain = slog_term::streamer().compact().build().fuse();
    //Logger::root(drain, o!())
    Logger::root(slog_term::streamer().compact().build().fuse(), o!())
}


//  ----------STATES----------


struct LinkDb {
    db: link_db::parse::database::Database,
}

struct LinkData {
    //len is desired num of threads (num cpus?)
    dumps: Vec<Mutex<Vec<link_data::IndexedEntry>>>,
    addrs: Vec<(String,u32)>,
}

struct RankData {
    links: fnv::FnvHashMap<u32,Entry>,
    ranks: fnv::FnvHashMap<u32, f64>,
}

struct HashLinks {
    links: fnv::FnvHashMap<u32,Entry>,
    ranks: fnv::FnvHashMap<u32, f64>,
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


fn argv<'a>() -> clap::ArgMatches<'a> {
    clap::App::new(crate_name!()).about(crate_description!())
        .author(crate_authors!("\n")).version(crate_version!())
        .arg(Arg::with_name("ranks").short("r").long("ranks_dump").takes_value(true)
             .help("Supply location of rank data in csv form"))
        .arg(Arg::with_name("manifest").short("json").takes_value(true)
             .help("Supply dump of parsed link data manifest in json form"))
        .arg(Arg::with_name("page.sql").short("pages").takes_value(true)
             .conflicts_with("manifest").requires("redirect.sql").requires("pagelinks.sql"))
        .arg(Arg::with_name("redirect.sql").short("dirs").takes_value(true)
             .conflicts_with("manifest").requires("page.sql").requires("pagelinks.sql"))
        .arg(Arg::with_name("pagelinks.sql").short("links").takes_value(true)
             .conflicts_with("manifest").requires("page.sql").requires("redirect.sql"))
        .group(clap::ArgGroup::with_name("sources").required(true)
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

    //println!("{:?}", hl.bfs(232327,460509));
    hl.print_bfs(232327,460509);

    //thread::sleep(time::Duration::from_secs(30));
}



