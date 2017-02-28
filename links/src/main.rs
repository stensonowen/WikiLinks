#![allow(dead_code)]

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

// NOTE: when scaling, remember to change link_db/parse/regexes.rs/IS_SIMPLE

#[macro_use] 
extern crate slog;
extern crate slog_term;
use slog::DrainExt;

//#[macro_use] 
//extern crate serde_derive;
//extern crate serde_json;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{/*Arc,*/Mutex};

//mod parse;
//mod pagerank;

pub mod link_db;
pub mod link_data;

//  ------STATE--MACHINE------

trait State { }
impl State for LinkDb { }
impl State for LinkData { }
impl State for HashLinks { }

struct LinkState<S: State> {
    //shared vars go here
    threads: usize,
    log: slog::Logger,
    state: S,
}

impl LinkState<LinkDb> {
    fn new(
        pages_db: PathBuf,
        redir_db: PathBuf,
        links_db: PathBuf,
        ) -> Self {
        let drain = slog_term::streamer().compact().build().fuse();
        let root_log = slog::Logger::root(drain, o!() );
        let pages_db_s = format!("{}", pages_db.display());
        let redir_db_s = format!("{}", redir_db.display());
        let links_db_s = format!("{}", links_db.display());
        let db_log = root_log.new(
            o!("pages" => pages_db_s, "redir" => redir_db_s, "links" => links_db_s)
            );
        LinkState { 
            log: root_log,
            threads: 4,
            state: LinkDb::new(
                pages_db,
                redir_db,
                links_db,
                //simple,
                //String::from("/home/owen/wikidata/simplewiki-20170220-page.sql"),
                //String::from("/home/owen/wikidata/simplewiki-20170220-redirect.sql"),
                //String::from("/home/owen/wikidata/simplewiki-20170220-pagelinks.sql"),
                db_log,
            ),
        }
    //ls.state.db.validate();
    //let a = parse::populate_db(String::new(), String::new(), String::new(), &root_log);


    }
}


//  ----------STATES----------


struct LinkDb {
    //db_pages: PathBuf,
    //db_redirect: PathBuf,
    //db_pagelinks: PathBuf,
    //simple_wiki: bool,
    //db: parse::Database,
    db: link_db::parse::database::Database,

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


// ------COMMON-OBJECTS------


//#[derive(Serialize, Deserialize)]
struct Entry {
    title: String,
    pagerank: f64,
    parents:  Vec<u32>,
    children: Vec<u32>,
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
    let mut ls = LinkState::new(
        PathBuf::from("/home/owen/wikidata/simplewiki-20170201-page.sql"),
        PathBuf::from("/home/owen/wikidata/simplewiki-20170201-redirect.sql"),
        PathBuf::from("/home/owen/wikidata/simplewiki-20170201-pagelinks.sql"),
        );

    ls.state.db.validate();
    ls.state.db.remove_redirects();
    ls.state.db.validate();
    println!("NO REDIRECTS");
    //ls.state.db.verify_links();
    //let a = parse::populate_db(String::new(), String::new(), String::new(), &root_log);

}

