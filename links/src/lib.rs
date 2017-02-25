#![allow(dead_code)]

// Order:
//  1.  Read in data from sql db, resolve conflicts
//          Mmap / csv export ?
//  2.  Calculate pagerank 
//          Mmap / csv export ?
//  3.  Construct hash table (multithreaded?)
//  4.  Remove hash table cruft; it is not immutable

// Order
//  0.  LinkDb
//          field:      handles to files
//          function:   process collisions
//                          also compute pagerank data
//  1.  LinkData
//          field:      vector of vectors of article entry
//                          thread-safe: can be populated concurrently
//          function:   create
//                          be able to create from fileS not just via db
//                              JSON? mmap? 
//          function:   store
//                          output to a fileS we can re-open later
//                              JSON? mmap?
//          function:   create fast lookups (put into hash table)
//                          I think this must be thread-safe
//  2.  HashLinks
//          field:      one hash table of links 
//          function:   search
//
//  reference: https://hoverbear.org/2016/10/12/rust-state-machine-pattern/

#[macro_use] 
extern crate slog;
extern crate slog_term;
use slog::DrainExt;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc,Mutex};

trait State { }
impl State for LinkDb { }
impl State for LinkData { }
impl State for HashLinks { }

struct LinkState<S: State> {
    //shared vars go here
    log: slog::Logger,
    state: S,
}



struct LinkDb {
    db_pages: PathBuf,
    db_redirect: PathBuf,
    db_pagelinks: PathBuf,
    simple_wiki: bool,
}

struct LinkData {
    //len is desired num of threads (num cpus?)
    dumps: Vec<Mutex<Vec<(u32,Entry)>>>,
    ranks: HashMap<u32,f64>,
    //dumps: Vec<Arc<Mutex<&u32>>>, //?
}

struct HashLinks {
    links: HashMap<u32,Entry>,
    // database?
    // HashMap<String,u32> ?
}



enum Entry {
    Page {
        title: String,
        children: Vec<u32>,
        parents: Vec<u32>,
    },
    Redirect(u32),
    Absent, //default? use?
}

fn test() {
    let drain = slog_term::streamer().compact().build().fuse();
    let root_log = slog::Logger::root(drain, o!() );
    let ls = LinkState { 
        log: root_log,
        state: HashLinks {
            links: HashMap::new(),
        }
    };

}
