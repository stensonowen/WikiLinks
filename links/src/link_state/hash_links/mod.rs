extern crate rand;

use {clap, fnv};

use super::{LinkState, ProcData, HashLinks};
use super::{LinkData, new_logger};
use super::Entry;
use web::Node;

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

impl From<LinkState<ProcData>> for LinkState<HashLinks> {
    fn from(old: LinkState<ProcData>) -> LinkState<HashLinks> {
        //if old.state.titles.is_none() {
        //    old.build_title_table();
        //}
        LinkState {
            threads:    old.threads,
            size:       old.size,
            log:        old.log,
            state:      HashLinks {
                links: old.state.links,
                //ranks: old.state.ranks,
                //titles: HashMap::new(),
                titles: old.state.titles,
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
    pub fn lookup_title<'a>(&'a self, title: &'a str) -> Node<'a> {
        match self.titles.get(title) {
            Some(&id) => Node::Found(id, title),
            None => Node::Unknown(title),
        }
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
            //clap should make this impossible
            unreachable!()
        };

        if let Some(p) = args.value_of("export") {
            ls_dt.export(PathBuf::from(p)).unwrap();
        }

        let mut ls_rd: LinkState<ProcData> = ls_dt.into();

        if let Some(r) = args.value_of("compute-ranks") {
            ls_rd.compute_ranks(&PathBuf::from(r)).unwrap();
        }

        //convert to HashLinks if starting a web server
        if args.is_present("web-server") {
            Some(ls_rd.into())
        } else {
            None
        }
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
