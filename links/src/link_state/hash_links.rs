extern crate rand;

use fnv;
use slog;

use super::{LinkState, LinkData, HashLinks};
use super::Entry;
use super::bfs::{BFS,BFS2};
use super::Path;

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io;


impl LinkState<HashLinks> {
    pub fn bfs(&self, src: u32, dst: u32) -> Path {
        let null = slog::Logger::root(slog::Discard, o!());
        let bfs = BFS::new(null, &self.state.links, src, dst);
        bfs.search()
    }
    pub fn bfs2(&self, src: u32, dst: u32) -> Path {
        let null = slog::Logger::root(slog::Discard, o!());
        let bfs = BFS2::new(null, &self.state.links, src, dst);
        bfs.search()
    }
    pub fn cli_bfs(&self) -> io::Result<()> { 
        let mut buf = String::new();
        println!("Starting bfs");
        loop {
            println!("\nEnter source title:  ");
            buf.clear();
            io::stdin().read_line(&mut buf)?;
            // TODO replace spaces with underscored?
            let src = match self.state.resolve_title(&buf) {
                Some(id) => id,
                None => { println!("No such title"); continue },
            };
            println!("Enter destination title:  ");
            buf.clear();
            io::stdin().read_line(&mut buf)?;
            let dst = match self.state.resolve_title(&buf) {
                Some(id) => id,
                None => { println!("No such title"); continue },
            };
            let bfs = BFS::new(self.log.clone(), &self.state.links, src, dst);
            let path = bfs.search();
            path.print(&self.state.links);
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
    fn resolve_title(&self, t: &str) -> Option<u32> {
        let t = t.trim();
        if t.is_empty() {
            return Some(self.select_random());
        }
        let t = t.to_uppercase();
        let t = t.replace(' ', "_");
        let hash = HashLinks::hash_title(&t);
        self._titles.get(&hash).map(|&i| i)
    }
    fn select_random(&self) -> u32 {
        let mut guess: u32;
        let mut count = 0;
        loop {
            count += 1;
            guess = rand::random();
            if self.links.contains_key(&guess) {
                println!("rand took {} iters", count);
                return guess;
            }
        }
    }

}

