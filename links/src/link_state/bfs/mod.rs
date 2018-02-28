
use fnv::{FnvHashSet, FnvHashMap};
use slog::Logger;

use std::mem;

use article::{PageId, Entry};

const MAX_DEPTH: usize = 10;

pub mod path;
use self::path::{Path, PathError};
pub mod ihm;
use self::ihm::{IHSet, IHMap};

// Find the shortest path between articles

// speed tests:
// DONE use fnv sets
// TODO swap out generic function with 2 concrete ones
//          this seems to be a little faster :/ maybe later

/* Optimizations
 *  Switch all lookup tables to fnv: 20%, 17%, 14% for large, medium, small searches
 *  Switch HashMap::entry to contains and insert
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

type Links = FnvHashMap<PageId, Entry>;
type Set = FnvHashSet<PageId>;
type Map = FnvHashMap<PageId, PageId>;

pub struct BFS<'a> {
    // where parent/children data is found
    // use member instead of whole HashLinks?
    links: &'a Links,
    log: Logger,

    // indices of src and dst
    src: PageId,
    dst: PageId,

    // comprehensive list of page_ids reachable from each node
    // for (k,v), there is a path from src → ⋯ → v → k (through children links)
    src_seen: FnvHashMap<PageId, PageId>,
    // for (k,v), there is a path from dst → ⋯ → v → k (through parent links)
    dst_seen: FnvHashMap<PageId, PageId>,

    // the farthest reachable rows are both subsets of their respective `seen` sets
    // the lowest row reachable via the `src`'s descendents
    row_down: FnvHashSet<PageId>,
    // the highest row reachable via the `dst`'s ancestors
    row_up: FnvHashSet<PageId>,
}


impl<'a> BFS<'a> {

    pub fn new(log: Logger, links: &Links, src: PageId, dst: PageId) -> BFS {
        BFS {
            links, log,
            src, dst,
            src_seen: FnvHashMap::default(),
            dst_seen: FnvHashMap::default(),
            row_down: FnvHashSet::default(),
            row_up:   FnvHashSet::default(),
        }
    }

    fn path_from(&self, p: Result<Vec<PageId>, PathError>) -> Path {
        Path {
            src: self.src,
            dst: self.dst,
            path: p,
        }
    }

    pub fn extract_path(&self, common: PageId) -> Path {
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
        info!(self.log, "Beginning search from {:?} to {:?}", self.src, self.dst);
        if self.src == self.dst {
            return Path { src: self.src, dst: self.dst, path: Ok(vec![self.src]) }
        }
        // do we need to add src/dst to src_seen/dst_seen ? seems inconsistent
        self.row_down.insert(self.src);
        self.row_up.insert(self.dst);

        // use one temp set rather than recreating new ones
        // would the allocator make recreating equally fast? kinda doubt it
        // TODO speed test
        let mut tmp: FnvHashSet<PageId> = FnvHashSet::default();

        for i in 0..MAX_DEPTH {
            if let Some(common) = self.iter_down(&mut tmp) {
                info!(self.log, "Found mid {:?} when down row len = {}", common, tmp.len());
                return self.extract_path(common);
            }
            mem::swap(&mut self.row_down, &mut tmp);
            tmp.clear();
            info!(self.log, "Iter #{}: down row size = {}", i, self.row_down.len());

            if let Some(common) = self.iter_up(&mut tmp) {
                info!(self.log, "Found mid {:?} when up row len = {}", common, tmp.len());
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

    fn iter_down(&mut self, tmp: &mut Set) -> Option<PageId> {
        Self::iter(self.links, &self.row_down, tmp, 
                   &mut self.src_seen, &self.dst_seen, 
                   Entry::get_children)
    }

    fn iter_up(&mut self, tmp: &mut Set) -> Option<PageId> {
        Self::iter(self.links, &self.row_up, tmp,
                   &mut self.dst_seen, &self.src_seen,
                   Entry::get_parents)
    }

    fn iter<F>(links: &'a Links, old_line: &Set, new_line: &mut Set,
               seen: &mut Map, targets: &Map, next: F)
        -> Option<PageId> 
        where F: Fn(&'a Entry) -> &'a [PageId]
    {
        // for each element in `old_line`, add its parents/children to `next_line`
        // as we see an entry, add it to `seen`
        // if an element is both `seen` and a `target`, a path has been found
        for &old in old_line {
            for &new in next(&links[&old]) {
                // only consider ids that haven't been `seen`
                if seen.contains_key(&new) == false {
                    seen.insert(new, old);
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

}


// identical BFS for testing (to bench against BFS1)

pub struct BFS2<'a> {
    links: &'a Links,
    log: Logger,
    src: PageId,
    dst: PageId,
    //src_seen: FnvHashMap<PageId, PageId>,
    //dst_seen: FnvHashMap<PageId, PageId>,
    //row_down: FnvHashSet<PageId>,
    //row_up: FnvHashSet<PageId>,
    src_seen: IHMap,
    dst_seen: IHMap,
    row_down: IHSet,
    row_up: IHSet,
}

//type Set2 = FnvHashSet<PageId>;
//type Map2 = FnvHashMap<PageId, PageId>;

impl<'a> BFS2<'a> {

    pub fn new(log: Logger, links: &Links, src: PageId, dst: PageId) -> BFS2 {
        BFS2 {
            links, log, src, dst,
            src_seen: IHMap::default(), dst_seen: IHMap::default(),
            row_down: IHSet::default(), row_up:   IHSet::default(),
        }
    }

    fn path_from(&self, p: Result<Vec<PageId>, PathError>) -> Path {
        Path { src: self.src, dst: self.dst, path: p, }
    }

    pub fn extract_path(&self, common: PageId) -> Path {
        let mut path = vec![common];
        let mut current = common;
        while current != self.src {
            //current = self.src_seen[&current];
            current = self.src_seen.get(current).unwrap();
            path.push(current);
        }
        path.reverse();
        current = common;
        while current != self.dst {
            current = self.dst_seen.get(current).unwrap();
            path.push(current);
        }
        Path {
            src: self.src,
            dst: self.dst,
            path: Ok(path),
        }
    }

    pub fn search(mut self) -> Path {
        info!(self.log, "Beginning search from {:?} to {:?}", self.src, self.dst);
        if self.src == self.dst {
            return Path { src: self.src, dst: self.dst, path: Ok(vec![self.src]) }
        }
        self.row_down.insert(self.src);
        self.row_up.insert(self.dst);
        let mut tmp: IHSet = IHSet::default();
        for i in 0..MAX_DEPTH {
            if let Some(common) = self.iter_down(&mut tmp) {
                info!(self.log, "Found mid {:?} when down row len = {}", common, tmp.len());
                return self.extract_path(common);
            }
            mem::swap(&mut self.row_down, &mut tmp);
            tmp.clear();
            info!(self.log, "Iter #{}: down row size = {}", i, self.row_down.len());

            if let Some(common) = self.iter_up(&mut tmp) {
                info!(self.log, "Found mid {:?} when up row len = {}", common, tmp.len());
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

    //#[inline]
    fn iter_down(&mut self, tmp: &mut IHSet) -> Option<PageId> {
        Self::iter(self.links, &self.row_down, tmp, 
                   &mut self.src_seen, &self.dst_seen,
                   Entry::get_children)
    }

    //#[inline]
    fn iter_up(&mut self, tmp: &mut IHSet) -> Option<PageId> {
        Self::iter(self.links, &self.row_up, tmp,
                   &mut self.dst_seen, &self.src_seen,
                   Entry::get_parents)
    }

    //#[inline]
    fn iter<F>(links: &'a Links, old_line: &IHSet, new_line: &mut IHSet,
               seen: &mut IHMap, targets: &IHMap, next: F)
        -> Option<PageId> 
        where F: Fn(&'a Entry) -> &'a [PageId]
    {
        for old in old_line.keys() {
            for &new in next(&links[&old]) {
                if seen.contains_key(new) == false {
                    seen.insert(new, old);
                    if targets.contains_key(new) {
                        return Some(new);
                    }
                    new_line.insert(new);
                }
            }
        }
        None
    }

}

