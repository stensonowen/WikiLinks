//use std::collections::HashMap;
extern crate rand;
use {clap, fnv};
use super::{LinkState, RankData, HashLinks};
use super::{LinkData, new_logger};
use super::Entry;
use std::path::{self, PathBuf};
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
        //TODO: create DB or something?
        LinkState {
            threads:    old.threads,
            size:       old.size,
            log:        old.log,
            state:      HashLinks {
                links: old.state.links,
                ranks: old.state.ranks,
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
        } else if let Some(m) = args.value_of("manifest") {
            LinkState::<LinkData>::from_file(PathBuf::from(m), new_logger()).unwrap()
        } else {
            //clap should make this impossible
            unreachable!()
        };

        //then decide whether to build pagelinks from data or import from backup
        let ls_rd = match args.value_of("ranks") {
            Some(r) => LinkState::<RankData>::from_ranks(ls_dt, path::Path::new(r)),
            None => ls_dt.into(),
        };

        //convert to HashLinks
        ls_rd.into()
    }
}
