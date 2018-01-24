extern crate rand;

use {clap, fnv};

use super::{LinkState, ProcData, HashLinks};
use super::{LinkData, new_logger};
use super::Entry;
//use web::Node;

use std::path::{self, PathBuf};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

mod bfs;


#[derive(Debug, Clone)]
pub struct Path {
    pub src: u32,
    pub dst: u32,
    pub path: Result<Vec<u32>,PathError>,
}

#[derive(Debug, Clone)]
pub enum PathError {
    NoSuchPath,
    Terminated(u32)
}

impl Path {
    pub fn size(&self) -> Option<usize> {
        if let Ok(ref v) = self.path {
            Some(v.len())
        } else {
            None
        }
    }
    fn print(&self, entries: &fnv::FnvHashMap<u32,Entry>) {
        println!("Path from {}\t(\"{}\")", self.src, entries.get(&self.src).unwrap().title);
        println!("  to {}\t(\"{}\") :", self.dst, entries.get(&self.dst).unwrap().title);
        match self.path {
            Ok(ref v) => for i in v {
                println!("\t{}:\t\"{}\"", i, entries.get(&i).unwrap().title);
            },
            Err(PathError::NoSuchPath) => println!("\tNo such path exists"),
            Err(PathError::Terminated(i)) => 
                println!("\tSearch expired after {} iterations", i),
        }
    }
}

//impl From<LinkState<ProcData>> for LinkState<HashLinks> {
//    fn from(old: LinkState<ProcData>) -> LinkState<HashLinks> {
impl From<LinkState<LinkData>> for LinkState<HashLinks> {
    fn from(old: LinkState<LinkData>) -> LinkState<HashLinks> {
        let (threads, size) = (old.threads, old.size);
        let (links, log, titles) = old.break_down();
        LinkState {
            threads:    threads,
            size:       size,
            log:        log,
            state:      HashLinks {
                //links:  LinkState::<ProcData>::consolidate_links(old.state.dumps, old.size),
                //links:  LinkData::consolidate_links(old.state.dumps, old.size),
                //titles: old.state.titles,
                //titles: HashLinks::hash_titles(old.state.titles),
                links:  links,
                _titles: HashLinks::hash_titles(titles),
            }
        }
    }
}

impl HashLinks {
    pub fn size(&self) -> usize {
        self.links.len()
    }
    pub fn get_links(&self) -> &fnv::FnvHashMap<u32,Entry> {
        &self.links
    }
    /*
    pub fn lookup_title<'a>(&'a self, query: &'a str) -> Node<'a> {
        // Empty: unused (maybe should mean 'random'?
        // Absent: try case-insensitive version
        if query.is_empty() {
            Node::Unused
        } else {
            let curr_hash = HashLinks::hash_title(query);
            //match self.titles.get(q).or(self.titles.get(&q.to_uppercase())) {
            match self.titles.get(&curr_hash) {
                Some(&id) => Node::Found(id, query),
                //None => Node::Unknown(q),
                None => {
                    let caps_hash = HashLinks::hash_title(&query.to_uppercase());
                    match self.titles.get(&caps_hash) {
                        Some(&id) => Node::Found(id, query),
                        None => Node::Unknown(query)
                    }
                }
            }
        }
    }
        */
    fn hash_title(t: &str) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
    fn hash_titles(old: HashMap<String,u32>) -> HashMap<u64,u32> {
        old.into_iter().map(|(q,i)| (HashLinks::hash_title(&q),i)).collect()
    }
}

impl LinkState<HashLinks> {
    pub fn extract(self) -> HashLinks {
        self.state
    }
    pub fn from_args(args: clap::ArgMatches) -> Option<Self> {
        //populate complete HashLinks from command-line args

        //first, decide whether to build links from source sql or previous backup
        let ls_dt: LinkState<LinkData> = if let (Some(p), Some(r), Some(l)) = 
            (args.value_of("page.sql"), 
             args.value_of("redirect.sql"), 
             args.value_of("pagelinks.sql")) 
        {
            LinkState::new(path::Path::new(p), path::Path::new(r), path::Path::new(l))
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


            // pmap -x <PID>
            println!("\n\n\nMEMORY USED:\n");
            ::std::process::Command::new("/usr/bin/pmap")
                .arg(format!("{}", ::std::process::id()))
                .spawn().unwrap();
            //loop {}

            None
        }
    }
}
