extern crate rand;

use fnv;

use super::{LinkState, LinkData, HashLinks};
use super::Entry;

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};


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

