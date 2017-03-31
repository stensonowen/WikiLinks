// https://hoverbear.org/2016/10/12/rust-state-machine-pattern/

use fnv;
use slog_term;
use slog::{Logger, DrainExt};

use std::sync::Mutex;
use std::collections::HashMap;

pub mod link_db;
pub mod link_data;
pub mod rank_data;
pub mod hash_links;

const IS_SIMPLE: bool = true;


//  ------STATE--MACHINE------


pub trait State { }
impl State for LinkDb { }
impl State for LinkData { }
impl State for ProcData { }
impl State for HashLinks { }

pub struct LinkState<S: State> {
    threads: usize,     // number of threads/files to use concurrently
    size:    usize,     // number of entries
    log:     Logger,    // root logger that will be split off for components
    state:   S,         // 1 of 4 values that represent development of the data
}

pub fn new_logger() -> Logger {
    Logger::root(slog_term::streamer().compact().build().fuse(), o!())
}


//  ----------STATES----------


pub struct LinkDb {
    /// Store parsed collection of links from sql dumps
    db: link_db::parse::database::Database,
    titles: HashMap<String,u32>,
}

pub struct LinkData {
    /// Store parsed and converted links and addresses
    /// Link data can be quickly written to or read from disk
    dumps: Vec<Mutex<Vec<link_data::IndexedEntry>>>,
    addrs: Vec<(String,u32)>,
    titles: HashMap<String,u32>,
}

pub struct ProcData {
    /// Store easily searchable link and pagerank data
    /// Pagerank data can be read from, dumped to, or exported to disk 
    links: fnv::FnvHashMap<u32,Entry>,
    //ranks: Option<Vec<(u32, f64)>>,
    //titles: Option<HashMap<String,u32>>,    // TODO: not optional?
    //titles: Option<HashMap<String,rank_data::TitleLookup>>,    // TODO: not optional?
    titles: HashMap<String,u32>,
}

pub struct HashLinks {
    /// Read-only, fast-lookup container for link and rank data
    /// Interact with diesel cache and interface with website
    links: fnv::FnvHashMap<u32,Entry>,
    //ranks: Vec<(u32, f64)>,
    //titles: HashMap<String,rank_data::TitleLookup>,
    titles: HashMap<String,u32>,
}


// ------COMMON-OBJECTS------


#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub title: String,
    pub parents:  Vec<u32>,
    pub children: Vec<u32>,
}

/*
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Title {
    Caps(u32),
    Orig(u32),
}
*/
