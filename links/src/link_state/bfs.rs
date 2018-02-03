
use fnv::{FnvHashSet, FnvHashMap};
use slog::Logger;

use std::mem;
use std::collections::hash_map;

use link_state::path::{Path, PathError};
use link_state::entry::Entry;

const MAX_DEPTH: u32 = 10;

// Find the shortest path between articles

// speed tests:
// DONE use fnv sets
// TODO replace sets w/ bloom filters
// TODO swap out naive iter functions
// TODO perf profiling :)

/* Optimizations
 *  Switch all lookup tables to fnv: 20%, 17%, 14% for large, medium, small searches
 */


/*
 *                    * ------------------------ `src`
 *                /   |    \              ____                   
 *              *     *      *                |               
 *            / | \ / | \   / \               |                    
 *           *  *  *  *  * *   *              |- `src_seen`                  
 *        /           |          \            |                  
 *                   ...                      |                     
 *   * * * * * * * * * * * * * * * * * *  ____|  } -- `row_down`
 *
 *                   ...
 *                                       ______
 *   * * * * * * * * * * * * * * * * * *       | } -- `row_up`
 *                   ...                       |
 *       \      \           /      /           |
 *        *  *  *  *  *  *  *  *  *            |
 *          \                   /              |- `dst_seen`
 *            *  *  *   *  *  *                |                                
 *              \    \ /    /                  |
 *                *   *   *              ______|
 *                  \ | /
 *                    * ------------------------ `dst`
 *
 */

type Links = FnvHashMap<u32, Entry>;
type Set = FnvHashSet<u32>;
type Map = FnvHashMap<u32, u32>;

pub struct BFS<'a> {
    // where parent/children data is found
    // use member instead of whole HashLinks?
    //links: &'a HashLinks,
    links: &'a Links,
    log: Logger,

    // indices of src and dst
    src: u32,
    dst: u32,

    // comprehensive list of page_ids reachable from each node
    // for (k,v), there is a path from src → ⋯ → v → k (through children links)
    src_seen: FnvHashMap<u32, u32>,
    // for (k,v), there is a path from dst → ⋯ → v → k (through parent links)
    dst_seen: FnvHashMap<u32, u32>,

    // the farthest reachable rows are both subsets of their respective `seen` sets
    // the lowest row reachable via the `src`'s descendents
    row_down: FnvHashSet<u32>,
    // the highest row reachable via the `dst`'s ancestors
    row_up: FnvHashSet<u32>,
}


impl<'a> BFS<'a> {

    pub fn new(log: Logger, links: &Links, src: u32, dst: u32) -> BFS {
        BFS {
            links, log,
            src, dst,
            src_seen: FnvHashMap::default(),
            dst_seen: FnvHashMap::default(),
            row_down: FnvHashSet::default(),
            row_up:   FnvHashSet::default(),
        }
    }

    fn path_from(&self, p: Result<Vec<u32>, PathError>) -> Path {
        Path {
            src: self.src,
            dst: self.dst,
            path: p,
        }
    }

    pub fn extract_path(&self, common: u32) -> Path {
        // `common` is the first entry reachable from src's children and dst's parents
        // path includes both src and dst
        let mut path = vec![common];
        let mut current = common;
        //first find path from midpoint to the src (will be backwards)
        while current != self.src {
            current = self.src_seen[&current];
            path.push(current);
        }
        path.reverse();
        current = common;
        while current != self.dst {
            current = self.dst_seen[&current];
            path.push(current);
        }
        Path {
            src: self.src,
            dst: self.dst,
            path: Ok(path),
        }
    }

    pub fn search(mut self) -> Path {
        info!(self.log, "Beginning search from {} to {}", self.src, self.dst);
        if self.src == self.dst {
            return Path { src: self.src, dst: self.dst, path: Ok(vec![self.src]) }
        }
        // do we need to add src/dst to src_seen/dst_seen ? seems inconsistent
        self.row_down.insert(self.src);
        self.row_up.insert(self.dst);

        // use one temp set rather than recreating new ones
        // would the allocator make recreating equally fast? kinda doubt it
        // TODO speed test
        let mut tmp: FnvHashSet<u32> = FnvHashSet::default();

        for i in 0..MAX_DEPTH {
            if let Some(common) = self.iter_down_2(&mut tmp) {
                info!(self.log, "Found mid {} when down row len = {}", common, tmp.len());
                return self.extract_path(common);
            }
            mem::swap(&mut self.row_down, &mut tmp);
            tmp.clear();
            info!(self.log, "Iter #{}: down row size = {}", i, self.row_down.len());

            if let Some(common) = self.iter_up_2(&mut tmp) {
                info!(self.log, "Found mid {} when up row len = {}", common, tmp.len());
                return self.extract_path(common);
            }
            mem::swap(&mut self.row_up, &mut tmp);
            tmp.clear();
            info!(self.log, "Iter #{}: up row size = {}", i, self.row_up.len());

            if self.row_up.is_empty() || self.row_down.is_empty() {
                if self.row_up.is_empty() {
                    info!(self.log, "No such path: No more parents to check");
                } else {
                    info!(self.log, "No such path: No more children to check");
                }
                return self.path_from(Err(PathError::NoSuchPath));
            }
        }
        self.path_from(Err(PathError::Terminated(MAX_DEPTH)))
    }

    fn iter_down_2(&mut self, tmp: &mut Set) -> Option<u32> {
        Self::iter(self.links, &mut self.row_down, tmp, 
                   &mut self.src_seen, &self.dst_seen, 
                   |e| e.get_children())
    }

    fn iter_up_2(&mut self, tmp: &mut Set) -> Option<u32> {
        Self::iter(self.links, &mut self.row_up, tmp,
                   &mut self.dst_seen, &self.src_seen,
                   |e| e.get_parents())
    }

    fn iter<F>(links: &'a Links, old_line: &Set, new_line: &mut Set,
               seen: &mut Map, targets: &Map, next: F)
        -> Option<u32> 
        where F: Fn(&'a Entry) -> &'a [u32]
    {
        // for each element in `old_line`, add its parents/children to `next_line`
        // as we see an entry, add it to `seen`
        // if an element is both `seen` and a `target`, a path has been found
        for &old in old_line {
            for &new in next(&links[&old]) {
                // only consider ids that haven't been `seen`
                if let hash_map::Entry::Vacant(v) = seen.entry(new) {
                    // insert `seen[new] = old`
                    v.insert(old);
                    // TODO: check a bloom filter or something here
                    if targets.contains_key(&new) {
                        // found an element reachable from both src and dst
                        return Some(new);
                    }
                    new_line.insert(new);
                }
            }
        }
        // no path was found
        None
    }

    /*
    /// Create a new `row_down` composed of all unseen nodes reachable from current `row_down`
    fn iter_down(&mut self) -> Option<u32> { 
        // for each node that is newly reachable from src's descendents
        let mut new_row_down: HashSet<u32> = HashSet::new(); // TODO add capacity guess
        for &old in &self.row_down { 
            // for each of its children
            for &new in self.links.links[&old].get_children() {
                // if it hasn't been seen before 
                if *self.src_seen.entry(new).or_insert(old) == old {
                    // we just inserted seen[new] = old (wasn't there before)
                    if self.dst_seen.contains_key(&new) {
                        // found an element reachable from both src and dst
                        return Some(new);
                    }
                    new_row_down.insert(new);
                }
            }
        }
        //self.row_down.iter().filter_map(|&i| Some(i));
        None
    }

    fn iter_up(&mut self) -> Option<u32> { None }
    */
}


pub struct BFS2<'a> {
    // where parent/children data is found
    // use member instead of whole HashLinks?
    //links: &'a HashLinks,
    links: &'a Links,
    log: Logger,

    // indices of src and dst
    src: u32,
    dst: u32,

    // comprehensive list of page_ids reachable from each node
    // for (k,v), there is a path from src → ⋯ → v → k (through children links)
    src_seen: FnvHashMap<u32, u32>,
    // for (k,v), there is a path from dst → ⋯ → v → k (through parent links)
    dst_seen: FnvHashMap<u32, u32>,

    // the farthest reachable rows are both subsets of their respective `seen` sets
    // the lowest row reachable via the `src`'s descendents
    row_down: FnvHashSet<u32>,
    // the highest row reachable via the `dst`'s ancestors
    row_up: FnvHashSet<u32>,
}

type Set2 = FnvHashSet<u32>;
type Map2 = FnvHashMap<u32, u32>;

impl<'a> BFS2<'a> {

    pub fn new(log: Logger, links: &Links, src: u32, dst: u32) -> BFS2 {
        BFS2 {
            links, log,
            src, dst,
            src_seen: FnvHashMap::default(),
            dst_seen: FnvHashMap::default(),
            row_down: FnvHashSet::default(),
            row_up:   FnvHashSet::default(),
        }
    }

    fn path_from(&self, p: Result<Vec<u32>, PathError>) -> Path {
        Path {
            src: self.src,
            dst: self.dst,
            path: p,
        }
    }

    pub fn extract_path(&self, common: u32) -> Path {
        // `common` is the first entry reachable from src's children and dst's parents
        // path includes both src and dst
        let mut path = vec![common];
        let mut current = common;
        //first find path from midpoint to the src (will be backwards)
        while current != self.src {
            current = self.src_seen[&current];
            path.push(current);
        }
        path.reverse();
        current = common;
        while current != self.dst {
            current = self.dst_seen[&current];
            path.push(current);
        }
        Path {
            src: self.src,
            dst: self.dst,
            path: Ok(path),
        }
    }

    // FIX: page 480637 "Norbergs_BK" has 1 parent that is itself
    pub fn search(mut self) -> Path {
        info!(self.log, "Beginning search from {} to {}", self.src, self.dst);
        if self.src == self.dst {
            return Path { src: self.src, dst: self.dst, path: Ok(vec![self.src]) }
        }
        // do we need to add src/dst to src_seen/dst_seen ? seems inconsistent
        self.row_down.insert(self.src);
        self.row_up.insert(self.dst);

        // use one temp set rather than recreating new ones
        // would the allocator make recreating equally fast? kinda doubt it
        // TODO speed test
        let mut tmp: FnvHashSet<u32> = FnvHashSet::default();

        for i in 0..MAX_DEPTH {
            if let Some(common) = self.iter_down_2(&mut tmp) {
                info!(self.log, "Found mid {} when down row len = {}", common, tmp.len());
                return self.extract_path(common);
            }
            mem::swap(&mut self.row_down, &mut tmp);
            tmp.clear();
            info!(self.log, "Iter #{}: down row size = {}", i, self.row_down.len());

            if let Some(common) = self.iter_up_2(&mut tmp) {
                info!(self.log, "Found mid {} when up row len = {}", common, tmp.len());
                return self.extract_path(common);
            }
            mem::swap(&mut self.row_up, &mut tmp);
            tmp.clear();
            info!(self.log, "Iter #{}: up row size = {}", i, self.row_up.len());

            if self.row_up.is_empty() || self.row_down.is_empty() {
                if self.row_up.is_empty() {
                    info!(self.log, "No such path: No more parents to check");
                } else {
                    info!(self.log, "No such path: No more children to check");
                }
                return self.path_from(Err(PathError::NoSuchPath));
            }
        }
        self.path_from(Err(PathError::Terminated(MAX_DEPTH)))
    }

    fn iter_down_2(&mut self, tmp: &mut Set2) -> Option<u32> {
        Self::iter(self.links, &mut self.row_down, tmp, 
                   &mut self.src_seen, &self.dst_seen, 
                   |e| e.get_children())
    }

    fn iter_up_2(&mut self, tmp: &mut Set2) -> Option<u32> {
        Self::iter(self.links, &mut self.row_up, tmp,
                   &mut self.dst_seen, &self.src_seen,
                   |e| e.get_parents())
    }

    fn iter<F>(links: &'a Links, old_line: &Set2, new_line: &mut Set2,
               seen: &mut Map2, targets: &Map2, next: F)
        -> Option<u32> 
        where F: Fn(&'a Entry) -> &'a [u32]
    {
        // for each element in `old_line`, add its parents/children to `next_line`
        // as we see an entry, add it to `seen`
        // if an element is both `seen` and a `target`, a path has been found
        for &old in old_line {
            for &new in next(&links[&old]) {
                // only consider ids that haven't been `seen`
                if let hash_map::Entry::Vacant(v) = seen.entry(new) {
                    // insert `seen[new] = old`
                    v.insert(old);
                    // TODO: check a bloom filter or something here
                    if targets.contains_key(&new) {
                        // found an element reachable from both src and dst
                        return Some(new);
                    }
                    new_line.insert(new);
                }
            }
        }
        // no path was found
        None
    }

    /*
    /// Create a new `row_down` composed of all unseen nodes reachable from current `row_down`
    fn iter_down(&mut self) -> Option<u32> { 
        // for each node that is newly reachable from src's descendents
        let mut new_row_down: HashSet<u32> = HashSet::new(); // TODO add capacity guess
        for &old in &self.row_down { 
            // for each of its children
            for &new in self.links.links[&old].get_children() {
                // if it hasn't been seen before 
                if *self.src_seen.entry(new).or_insert(old) == old {
                    // we just inserted seen[new] = old (wasn't there before)
                    if self.dst_seen.contains_key(&new) {
                        // found an element reachable from both src and dst
                        return Some(new);
                    }
                    new_row_down.insert(new);
                }
            }
        }
        //self.row_down.iter().filter_map(|&i| Some(i));
        None
    }

    fn iter_up(&mut self) -> Option<u32> { None }
    */
}

