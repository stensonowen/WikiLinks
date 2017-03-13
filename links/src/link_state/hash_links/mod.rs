extern crate rand;

use {clap, fnv};

use super::{LinkState, RankData, HashLinks};
use super::{LinkData, new_logger};
use super::Entry;

use std::path::{self, PathBuf};
use std::collections::HashMap;

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

impl From<LinkState<RankData>> for LinkState<HashLinks> {
    fn from(old: LinkState<RankData>) -> LinkState<HashLinks> {
        LinkState {
            threads:    old.threads,
            size:       old.size,
            log:        old.log,
            state:      HashLinks {
                links: old.state.links,
                //ranks: old.state.ranks,
                titles: HashMap::new(),
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
    pub fn get_ranks(&self) -> &Vec<(u32,f64)> {
        &self.ranks
    }
    pub fn random_id(&self) -> u32 {
        //find random element in table; return its id
        let index = rand::random::<usize>() % self.ranks.len();
        self.ranks[index].0
    }
    pub fn random_id_and_title(&self) -> (u32, &str) {
        let id = self.random_id();
        let entry = self.links.get(&id).unwrap();
        (id, &entry.title)
    }
    */
}

impl LinkState<HashLinks> {
    pub fn extract(self) -> HashLinks {
        self.state
    }
    pub fn from_args(args: clap::ArgMatches) -> Self {
        //populate complete HashLinks from command-line args

        //first, decide whether to build links from source sql or previous backup
        let ls_dt: LinkState<LinkData> = if let (Some(p), Some(r), Some(l)) = 
            (args.value_of("page.sql"), 
             args.value_of("redirect.sql"), 
             args.value_of("pagelinks.sql")) 
        {
            LinkState::new(PathBuf::from(p), PathBuf::from(r), PathBuf::from(l))
                .into()
        } else if let Some(m) = args.value_of("import_links") {
            LinkState::<LinkData>::import(PathBuf::from(m), new_logger()).unwrap()
        } else {
            //clap should make this impossible
            unreachable!()
        };

        if let Some(p) = args.value_of("export_links") {
            ls_dt.export(PathBuf::from(p)).unwrap();
        }

        let mut ls_rd: LinkState<RankData> = ls_dt.into();

        if let Some(md) = args.value_of("import_md") {
            ls_rd.import(path::Path::new(md));
        } else {
            ls_rd.build_title_table();
        }
        if args.is_present("compute_ranks") {
            ls_rd.compute_ranks();
        }
        if let Some(md) = args.value_of("export_md") {
            ls_rd.export(&PathBuf::from(md)).unwrap();
        }

        //convert to HashLinks
        ls_rd.into()
    }
}


/*
 * ARGS:
 *      Link Data
 *          3 databases or
 *          a manifest 
 *          optional instructions for outputting a manifest
 *      Rank Data
 *          optional import manifest
 *          optional export manifest
 *          optional "compute_ranks" argument
 */
