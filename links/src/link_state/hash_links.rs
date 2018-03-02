extern crate rand;

use fst;
use slog;

use super::{LinkState, LinkData, HashLinks};
use super::link_table::{PageIndex, LinkTable};
use article::PageId;
use super::bfs::{BFS, BFS2};
use super::Path;

use std::io;


impl LinkState<HashLinks> {
    pub fn print_path(&self, path: Path) {
        path.print2(|id| self.state.links.get_title(id).unwrap().to_string())
    }
    pub fn bfs_id(&self, src: PageId, dst: PageId) -> Path {
        let src = self.state.links.get_index(src).unwrap();
        let dst = self.state.links.get_index(dst).unwrap();
        self.bfs(src, dst)
    }
    pub fn bfs(&self, src: PageIndex, dst: PageIndex) -> Path {
        let null = slog::Logger::root(slog::Discard, o!());
        let table = self.state.links.get_table();
        let bfs = BFS::new(null, table, src, dst);
        bfs.search()
    }
    pub fn bfs2(&self, src: PageIndex, dst: PageIndex) -> Path {
        let null = slog::Logger::root(slog::Discard, o!());
        let table = self.state.links.get_table();
        let bfs = BFS2::new(null, table, src, dst);
        bfs.search()
    }
    pub fn get_index(&self, id: PageId) -> Option<PageIndex> {
        self.state.links.get_index(id)
    }
    pub fn cli_bfs(&self) -> io::Result<()> { 
        let table = self.state.links.get_table();
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
            let bfs = BFS::new(self.log.clone(), table, src, dst);
            let path = bfs.search();
            self.print_path(path);
        }
    }
}

impl From<LinkState<LinkData>> for LinkState<HashLinks> {
    fn from(old: LinkState<LinkData>) -> LinkState<HashLinks> {
        let (threads, size) = (old.threads, old.size);
        let (links, log, titles_b) = old.break_down();
        let titles_map = fst::Map::from_bytes(titles_b).expect("invalid fst bytes");
        let link_table = LinkTable::convert_from_map(links);
        LinkState {
            threads:    threads,
            size:       size,
            log:        log,
            state:      HashLinks {
                links:  link_table,
                titles: titles_map,
            }
        }
    }
}

impl HashLinks {
    pub fn size(&self) -> usize {
        self.links.len()
    }
    //pub fn get_links(&self) -> &fnv::FnvHashMap<PageId,Entry> {
    pub fn get_links(&self) -> &LinkTable {
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

    /*
     * Hash-based title storage is tentatively removed in favor of fst
    fn hash_title(t: &str) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
    fn hash_titles(old: HashMap<String,u32>) -> HashMap<u64,u32> {
        old.into_iter().map(|(q,i)| (HashLinks::hash_title(&q),i)).collect()
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
    */
    fn resolve_title(&self, t: &str) -> Option<PageIndex> {
        let t = t.trim();
        //if t.is_empty() { return Some(self.select_random()); }
        //let t = t.to_uppercase();
        let t = t.replace(' ', "_");
        //let hash = HashLinks::hash_title(&t);
        //self.titles.get(&hash)
        self.titles.get(&t).map(|n| {
            assert!(n <= u64::from(u32::max_value()));
            PageId::from(n as u32)
        }).and_then(|id| {
            self.links.get_index(id)
        })
    }

}

