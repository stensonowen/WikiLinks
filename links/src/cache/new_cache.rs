// Cache for the most recent searches that have been performed
// Must handle frequent writes /and/ frequent reads
// Should this be the default?

use std::collections::{VecDeque, HashSet};
use std::sync::{Arc, Mutex};

use super::cache_elem::CacheElem;
use super::CACHE_SIZE;

// NOTE: should this cache even be in order? Or should it be like the 'random' cache?
// If it were just random we wouldn't need a VecDequeue

#[derive(Debug)]
pub struct NewCacheOuter(Arc<Mutex<NewCacheInner>>);

impl NewCacheOuter {
    pub fn new() -> Self {
        NewCacheOuter(Arc::new(Mutex::new(NewCacheInner::new())))
    }
    pub fn from(old: Vec<(&str, i8, &str)>) -> Self {
        let new = Self::new();
        for (s,l,d) in old {
            new.insert_elem(CacheElem::new(s,d,l as usize));
        }
        new
    }
    pub fn get(&self) -> Vec<CacheElem> {
        let rfrn = self.0.clone();
        let lock = rfrn.lock().unwrap();
        lock.get()
    }
    pub fn insert_elem(&self, elem: CacheElem) {
        let rfrn = self.0.clone();
        let mut lock = rfrn.lock().unwrap();
        lock.insert_elem(elem);
    }
}

#[derive(Debug)]
struct NewCacheInner {
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


impl NewCacheInner {
    fn new() -> Self {
        NewCacheInner {
            queue: VecDeque::with_capacity(CACHE_SIZE),
            contents: HashSet::with_capacity(CACHE_SIZE),
        }
    }
    fn get(&self) -> Vec<CacheElem> {
        let slices = self.queue.as_slices();
        if slices.1.is_empty() {
            slices.0.to_vec()
        } else {
            // using push_front and pop_back makes this pretty common, unfortunately
            // but it's not worth the impaired readability of reversing inside template
            let mut v = slices.0.to_vec();
            v.extend_from_slice(slices.1);
            v
        }
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
            let rem = self.queue.pop_back().unwrap();
            self.contents.remove(&rem);
        }
        let index = self.queue.len();
        self.queue.push_front(elem);
        self.contents.insert(self.queue.get(index).unwrap().clone());
        //assert_eq!(Some(&elem), self.queue.get(index));
    }
}


