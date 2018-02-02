
use fnv::FnvHashMap;

use std::collections::{HashSet, HashMap};
//use super::{Path, PathError, HashLinks};
//use link_state::hash_links::{Path, PathError};
//use link_state::path::{Path, PathError};
use super::path::{Path, PathError};
//use super::path::Foo;
use std::mem;
use link_state::entry::Entry;
// links: fnv::FnvHashMap<u32,Entry>

const MAX_DEPTH: u32 = 10;

// Find the shortest path between articles

// speed tests:
// TODO use fnv sets
// TODO replace sets w/ bloom filters
// TODO swap out naive iter functions
// TODO perf profiling :)

/*
impl HashLinks {
    fn do_bfs(&self, src: u32, dst: u32) -> Path {
        let mut bfs = BFS::new(&self.links, src, dst);
        bfs.search() // .print(&self.links);
    }
}
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

struct BFS<'a> {
    // where parent/children data is found
    // use member instead of whole HashLinks?
    //links: &'a HashLinks,
    links: &'a Links,

    // indices of src and dst
    src: u32,
    dst: u32,

    // comprehensive list of page_ids reachable from each node
    // for (k,v), there is a path from src → ⋯ → v → k (through children links)
    src_seen: HashMap<u32, u32>,
    // for (k,v), there is a path from dst → ⋯ → v → k (through parent links)
    dst_seen: HashMap<u32, u32>,

    // the farthest reachable rows are both subsets of their respective `seen` sets
    // the lowest row reachable via the `src`'s descendents
    row_down: HashSet<u32>,
    // the highest row reachable via the `dst`'s ancestors
    row_up: HashSet<u32>,
}

type Links = FnvHashMap<u32, Entry>;
type Set = HashSet<u32>;
type Map = HashMap<u32, u32>;

impl<'a> BFS<'a> {
    fn new(links: &Links, src: u32, dst: u32) -> BFS {
        BFS {
            links, src, dst,
            src_seen: HashMap::new(),
            dst_seen: HashMap::new(),
            row_down: HashSet::new(),
            row_up:   HashSet::new(),
        }
    }
    fn path_from(&self, p: Result<Vec<u32>, PathError>) -> Path {
        Path {
            src: self.src,
            dst: self.dst,
            path: p,
        }
    }

    fn extract_path(&self, common: u32) -> Path {
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

    fn search(&mut self) -> Path {
        //info!(self.links.log, "foo");
        if self.src == self.dst {
            return Path { src: self.src, dst: self.dst, path: Ok(vec![self.src]) }
        }
        // do we need to add src/dst to src_seen/dst_seen ? seems inconsistent
        self.row_down.insert(self.src);
        self.row_up.insert(self.dst);

        // use one temp set rather than recreating new ones
        // would the allocator make recreating equally fast? kinda doubt it
        // TODO speed test
        let mut tmp: HashSet<u32> = HashSet::new();

        for _ in 0..MAX_DEPTH {
            if let Some(common) = self.iter_down_2(&mut tmp) {
                return self.extract_path(common);
            }
            mem::swap(&mut self.row_down, &mut tmp);
            tmp.clear();

            if let Some(common) = self.iter_up_2(&mut tmp) {
                return self.extract_path(common);
            }
            mem::swap(&mut self.row_up, &mut tmp);
            tmp.clear();

            if self.row_up.is_empty() || self.row_down.is_empty() {
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
                if *seen.entry(new).or_insert(old) == old {
                    // just inserted `seen[new] = old`
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


/*
enum SearchDirection { Up, Down, }

impl HashLinks {
    pub fn print_bfs(&self, src: u32, dst: u32) {
        let bfs = self.bfs(src, dst);
        bfs.print(&self.links);
    }
    pub fn bfs(&self, src: u32, dst: u32) -> Path {
        use self::SearchDirection::*;
        // perform breadth-first-search using entries from src to dst
        // should this be multithreaded??
        // TODO: investigate using a Bloom Filter for checking intersection quickly
        //  Might be faster: that is its thing
        //  Might be negligible: uses hashing and a lookup table, which is basically a hashmap
        //  Might be useful if this were multithreaded?
        // TODO: play with different approximate hints
        //  i.e. starting/changing hashmap capacity via guessing
        // TODO: fix code duplication (top-down and bottom-up doing almost the same thing
        // TODO: there's a fair bit of redundancy: top_down_n ⊆ src_seen ⊇ top_down_o

        if src == dst {
            return Path{ src: src, dst: dst, path: Ok(vec![src])};
        }

        //`seen` pages: the id we've encountered and the id that linked to it (parent OR child)
        let mut src_seen: HashMap<u32, u32> = HashMap::new();
        let mut dst_seen: HashMap<u32, u32> = HashMap::new();

        // keep track of articles we're checking in each direction
        // each direction should have an immutable bank and a mutable collector (that swap)
        // start from src, follow children
        let mut top_down: HashSet<u32> = HashSet::new();
        top_down.insert(src);
        // start from dst, follow parents
        let mut bottom_up: HashSet<u32> = HashSet::new();
        bottom_up.insert(dst);

        src_seen.insert(src, src);
        dst_seen.insert(dst, dst);

        let mut tmp: HashSet<u32> = HashSet::new();

        for _ in 0..MAX_DEPTH {
            if let Some(midpoint) = self.iterate(&top_down, &mut tmp, Down,
                                            &mut src_seen, &dst_seen) {
                return Path { src: src, dst: dst,
                    path: Ok(extract_path(src, dst, &src_seen, &dst_seen, midpoint)) };
            }
            top_down.clear();
            swap(&mut top_down, &mut tmp);

            if let Some(midpoint) = self.iterate(&bottom_up, &mut tmp, Up,
                                            &mut dst_seen, &src_seen) {
                return Path { src: src, dst: dst,
                    path: Ok(extract_path(src, dst, &src_seen, &dst_seen, midpoint)) };
            }
            bottom_up.clear();
            swap(&mut bottom_up, &mut tmp);

            if bottom_up.is_empty() || top_down.is_empty() {
                return Path { src: src, dst: dst, path: Err(PathError::NoSuchPath) };
            }
        }
        Path { src: src, dst: dst, path: Err(PathError::Terminated(MAX_DEPTH)) }
    }

    fn iterate(&self, prev_line: &HashSet<u32>,
               new_line: &mut HashSet<u32>,
               direction: SearchDirection,
               reachable: &mut HashMap<u32, u32>,
               targets: &HashMap<u32, u32>)
               -> Option<u32> 
    {
        // go through every element in `prev_line`, adding parents/children to `next_line`
        // as we see each entry, add it to `reachable`.
        // If we find an element in `targets` and `reachable`, searching is over: return it
        use self::SearchDirection::*;
        for &i in prev_line {
            // link_start
            let pool = match direction {
                Up => self.links[&i].get_parents(),
                Down => self.links[&i].get_children(),
            };
            for &j in pool {
                // link_end
                // for each link_end, if unseen, mark it as reachable
                if *reachable.entry(j).or_insert(i) == i {
                    // we successfully inserted `seen[j]=i`, and it wasn't there before
                    if targets.contains_key(&j) {
                        // we found an element that is in both `reachable` and `target`
                        return Some(j);
                    }
                    new_line.insert(j);
                }
            }
        }
        None
    }
}
fn extract_path(src: u32,
                dst: u32,
                src_seen: &HashMap<u32, u32>,
                dst_seen: &HashMap<u32, u32>,
                common: u32)
                -> Vec<u32> {
    // given entries findable from the src, those from the dst, and an intersecting entry,
    // find the path from src to dst
    // For now, the path should include the src and dst ids
    let mut path = vec![common];
    // first find the path from the midpoint to the src (will be backwards)
    let mut current = common;
    while current != src {
        // could just iterate until it finds something without a parent? no passing src/dst ?
        // this is easier to debug for now
        current = *src_seen.get(&current).unwrap();
        path.push(current);
    }
    path.reverse();
    current = common;
    while current != dst {
        current = *dst_seen.get(&current).unwrap();
        path.push(current);
    }
    path
}
*/
