use std::collections::{HashSet, BinaryHeap, VecDeque};
use std::cmp::{Ord, PartialOrd, Ordering};
use std::sync::{Mutex, RwLock};
use super::CacheSort;

const CACHE_SIZE: usize = 16;

#[derive(Debug)]
pub struct StackCache {
    // very commonly updated
    recent: Mutex<RecentCache>,

    // Very common read op: check smallest length
    // Quite common: random insert, pop last element
    length: RwLock<LengthCache>,
}

impl StackCache {
    pub fn blank() -> Self {
        StackCache {
            recent: Mutex::new(RecentCache::new()),
            length: RwLock::new(LengthCache::new()),
        }
    }
    pub fn get(&mut self, sort: &CacheSort) -> Vec<CacheElem> {
        match *sort {
            CacheSort::Recent => self.get_recent(),
            CacheSort::Length => self.get_longest(),
        }
    }
    fn insert(&mut self, elem: CacheElem) {
        // if pertinent, update long list
        let update_len: bool = match self.length.read() {
            // uhhh, careful refactoring this that rlock and wlock don't overlap
            Ok(r) => r.should_insert(&elem),
            Err(_) => false,
        };
        if update_len {
            if let Ok(mut w) = self.length.write() {
                w.insert_elem(elem.clone());
            }
        }
        // always (try to) update recent list
        if let Ok(r) = self.recent.get_mut() {
            r.insert_elem(elem);
        }
    }
    fn get_recent(&self) -> Vec<CacheElem> {
        let r = self.recent.lock().unwrap();
        r.get().to_vec()
    }
    fn get_longest(&mut self) -> Vec<CacheElem> {
        if let Some(l) = self.try_get_longest() {
            l
        } else {
            self.rebuild_longest()
        }
    }
    fn try_get_longest(&self) -> Option<Vec<CacheElem>> {
        self.length.read().unwrap().cache()
    }
    fn rebuild_longest(&mut self) -> Vec<CacheElem> {
        self.length.write().unwrap().get().to_vec()
    }
}

#[derive(Debug)]
struct RecentCache {
    // keep track of all cached elements that appeared most recently
    // every page load will require 
    //  1) look up a search result in this collection
    //  2) get a list of the cache elements in order
    //  3) add a search result to the top
    //  4) remove a search result from the bottom (or maybe the middle)

    // Very common read op: check if search is contained
    // Most common: pop from the top and push to the bottom
    // Somewhat common: move around
    
    queue: VecDeque<CacheElem>,
    // rely on VecDeque.contains() every iteration? Or use a redundant hashset?
    contents:   HashSet<CacheElem>,
}

#[derive(Debug)]
struct LengthCache {
    // keep track of the longest searches we've seen
    // every page load will require
    //  1) lookup of the shortest value contained (can be worked around)
    //  2) get a list of the cache elements in order
    // occasional modifications require
    //  1) insert into middle of the list (maintain in sorted order)
    //  2) pop the shortest element from the list

    // handle efficient lookups/inserts/etc.
    heap: BinaryHeap<CacheElem>,
    temp: Option<Vec<CacheElem>>,
}

impl LengthCache {
    fn new() -> Self {
        LengthCache {
            heap: BinaryHeap::with_capacity(CACHE_SIZE),
            temp: None,
        }
    }
    fn min_len(&self) -> Option<u8> {
        self.heap.peek().map(|e| e.len)
    }
    fn should_insert(&self, new: &CacheElem) -> bool {
        if self.heap.len() < CACHE_SIZE {
            true
        } else if let Some(shortest) = self.heap.peek() {
            new.len > shortest.len 
        } else {
            true 
        }
    }
    fn get(&mut self) -> Vec<CacheElem> {
        match self.temp {
            Some(ref v) => v.clone(),
            None => self.rebuild_cache(),
        }
    }
    fn rebuild_cache(&mut self) -> Vec<CacheElem> {
        let iter = self.heap.iter();
        self.temp = Some(iter.map(|e| (*e).clone()).collect());
        match self.temp {
            Some(ref v) => v.clone(),
            None => unreachable!(),
        }
    }
    fn insert_elem(&mut self, elem: CacheElem) {
        // ONLY call iff should_insert
        self.temp = None;
        self.heap.push(elem);
        if self.heap.len() > CACHE_SIZE {
            self.heap.pop();
        }
    }
    fn cache(&self) -> Option<Vec<CacheElem>> {
        if let Some(ref l) = self.temp {
            Some(l.clone())
        } else {
            None
        }
    }
}

impl RecentCache {
    fn new() -> Self {
        RecentCache {
            queue: VecDeque::with_capacity(CACHE_SIZE),
            contents: HashSet::with_capacity(CACHE_SIZE),
        }
    }
    fn get(&self) -> &[CacheElem] {
        // we never `push_front()`, so the contents will always be in the first array
        // I think
        let slices = self.queue.as_slices();
        assert!(slices.1.is_empty());
        slices.0
    }
    fn insert_elem(&mut self, elem: CacheElem) {
        // TODO: if elem is already present, move it to the from of the queue??
        // for now, never remove elements except from the back
        if self.contents.contains(&elem) {
            // if new elem is already somewhere, that's fine
            // this makes sense, right?
            return;
        }
        if self.queue.len() >= CACHE_SIZE {
            // if adding an element will make the queue too long, remove the least recent
            let rem = self.queue.pop_front().unwrap();
            self.contents.remove(&rem);
        }
        let index = self.queue.len();
        self.queue.push_back(elem);
        self.contents.insert(self.queue.get(index).unwrap().clone());
        //assert_eq!(Some(&elem), self.queue.get(index));
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone)]
pub struct CacheElem {
    // TODO: will LLVM optimize equality comparison to compare numbers first?
    // or should we implement PartialEq ourselves?
    src: String,
    dst: String,
    //src: &'a str,
    //dst: &'a str,
    len: u8,
    // don't bother to store inconclusive searches
}

impl PartialOrd for CacheElem {
    fn partial_cmp(&self, othr: &Self) -> Option<Ordering> {
        Some(self.cmp(othr))
    }
}

impl Ord for CacheElem {
    fn cmp(&self, othr: &Self) -> Ordering {
        // Sort by length first
        // Longer sizes should be 'less', i.e. first in BTree
        if self.len != othr.len {
            self.len.cmp(&othr.len).reverse()
        } else {
            (&self.src, &self.dst).cmp(&(&othr.src, &othr.dst)).reverse()
        }
    }
}


//  ---------------------------




