// https://hoverbear.org/2016/10/12/rust-state-machine-pattern/

use fnv;
use clap;
use slog_term;
use slog::{Logger, DrainExt};

use std::path::{Path as FsPath, PathBuf};
use std::collections::HashMap;

pub mod link_db;
pub mod link_data;
pub mod proc_data;
pub mod hash_links;

pub mod bfs;
pub mod entry;
pub use self::entry::Entry;
pub use self::bfs::path::Path;


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
    //dumps: Vec<Mutex<Vec<link_data::IndexedEntry>>>,
    dumps: Vec<Vec<link_data::IndexedEntry>>,
    titles: HashMap<String,u32>,
}

pub struct ProcData {
    /// Store easily searchable link and pagerank data
    /// Pagerank data can be read from, dumped to, or exported to disk 
    links: fnv::FnvHashMap<u32,Entry>,
}

pub struct HashLinks {
    /// Read-only, fast-lookup container for link and proc data
    /// Interact with diesel cache and interface with website
    links: fnv::FnvHashMap<u32,Entry>,
    _titles: HashMap<u64,u32>,
}

//  ---------- ARGS ----------

// only allow T s.t. can be `from` LinkState<LinkData>
impl<T: State> LinkState<T> where LinkState<T>: From<LinkState<LinkData>> {
    pub fn from_args(args: &clap::ArgMatches) -> LinkState<T> {
        //first, decide whether to build links from source sql or previous backup
        let ls_dt: LinkState<LinkData> = if let (Some(p), Some(r), Some(l)) = 
            (args.value_of("page.sql"), 
             args.value_of("redirect.sql"), 
             args.value_of("pagelinks.sql")) 
        {
            LinkState::new(FsPath::new(p), FsPath::new(r), FsPath::new(l))
                .into()
        } else if let Some(m) = args.value_of("import") {
            LinkState::<LinkData>::import(PathBuf::from(m), new_logger()).unwrap()
        } else {
            panic!("The data has to come from somewhere; {}", 
                   "supply either a manifest or 3 sql dumps");
        };

        if let Some(p) = args.value_of("export") {
            ls_dt.export(PathBuf::from(p)).unwrap();
        }
        ls_dt.into()

            /*
        //ls_dt.foo();
        if args.is_present("web-server") {
            Some(ls_dt.into())
        } else {
            // do analytics
            let mut ls_rd: LinkState<ProcData> = ls_dt.into();

            if let Some(r) = args.value_of("compute_ranks") {
                ls_rd.compute_ranks(&PathBuf::from(r)).unwrap();
            }

            if let Some(i) = args.value_of("farthest_ancestor") {
                let id: u32 = i.parse().unwrap();
                ls_rd.longest_path(id);
            }

            // COMPUTE NEIGHBOR REDUNDANCY: TODO
            /*
            info!(ls_rd.log, "Checking neighbor redundancy...");
            let dupes = ls_rd.neighbor_redundancy();
            info!(ls_rd.log, "REDUNDANT u32 LINKS: {}", dupes);
            */

            let num_children: usize = ls_rd.state.links.values()
                .map(|e| e.get_children().len()).sum();
            let num_parents: usize = ls_rd.state.links.values()
                .map(|e| e.get_parents().len()).sum();
            info!(ls_rd.log, "Total number of children:  {}", num_children);
            info!(ls_rd.log, "Total number of parents :  {}", num_parents);
            // SIMPLE
            //      Redundancy: 2,754,583
            //      BEFORE OPT: 
            //          CHILDREN = PARENTS = 5,067,702
            //          MEMORY: total = 154,184 K
            //      AFTER OPT:
            //          CHILDREN = PARENTS = 5,067,702
            //          MEMORY: total = 197,104 K (parse run)
            //                  total = 129,516 K (load run)
            //          
            // ENWIKI
            //      Redundancy: 229,711,548
            //      BEFORE OPT:
            //          CHILDREN = PARENTS = 426,766,968
            //          MEMORY: total = 8,724,828 K
            //      AFTER OPT:
            //          CHILDREN = PARENTS = 426,766,968
            //          MEMORY: total = 7,158,108 K
            //
            //


            // pmap -x <PID>
            println!("\n\n\nMEMORY USED:\n");
            ::std::process::Command::new("/usr/bin/pmap")
                .arg(format!("{}", ::std::process::id()))
                .spawn().unwrap();
            //loop {}

            None
        }
                */
    }
}

