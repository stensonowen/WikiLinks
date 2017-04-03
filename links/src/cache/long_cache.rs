// Cache for the longest searches that have been observed
// Must handle frequent reads and infrequent writes
// Should this be the default?

// TODO: verify none of my code used here can panic!
// don't want to poison lock, which would be bad

//use std::collections::{BinaryHeap};
use std::sync::{Arc, RwLock};

use super::cache_elem::CacheElem;
use super::CACHE_SIZE;

#[derive(Debug)]
pub struct LongCacheOuter(Arc<RwLock<LongCacheInner>>);

impl LongCacheOuter {
    pub fn new() -> Self {
        LongCacheOuter(Arc::new(RwLock::new(LongCacheInner::new())))
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
        let lock = rfrn.read().unwrap();
        lock.get()
    }
    pub fn should_insert(&self, new: &CacheElem) -> bool {
        let rfrn = self.0.clone();
        let lock = rfrn.read().unwrap();
        lock.should_insert(new)
    }
    pub fn insert_elem(&self, new: CacheElem) {
        let rfrn = self.0.clone();
        let mut lock = rfrn.write().unwrap();
        lock.insert_elem(new);
    }
}


#[derive(Debug)]
pub struct LongCacheInner {
    // keep track of the longest searches we've seen
    // every page load will require
    //  1) lookup of the shortest value contained (can be worked around)
    //  2) get a list of the cache elements in order
    // This is almost entirely going to be read, not written
    // so a sorted vector is probably fine
    elems: Vec<CacheElem>,
    // could alternately use a BTreeSet and a temp vec cache
}

impl LongCacheInner {
    fn new() -> Self {
        LongCacheInner {
            elems: Vec::with_capacity(CACHE_SIZE+1),
        }
    }
    fn min_len(&self) -> Option<u8> {
        self.elems.last().map(|e| e.len())
    }
    fn should_insert(&self, new: &CacheElem) -> bool {
        if self.elems.len() < CACHE_SIZE {
            self.elems.contains(new) == false
        } else if let Some(shortest) = self.min_len() {
            new.len() > shortest && self.elems.contains(new) == false
        } else {
            self.elems.contains(new) == false
        }
    }
    fn get(&self) -> Vec<CacheElem> {
        self.elems.clone()
    }
    fn insert_elem(&mut self, elem: CacheElem) {
        // ONLY call iff should_insert
        assert!(self.should_insert(&elem));
        // Linear search: only add iff necessary
        // TODO: change data structure so this is faster?
        // this op is pretty uncommon, and there are max ~15 elements
        self.elems.push(elem);
        //self.elems.sort_by_key(|e| e.len());
        self.elems.sort();
        self.elems.dedup();
        while self.elems.len() > CACHE_SIZE {
            self.elems.pop();
        }
    }
}
