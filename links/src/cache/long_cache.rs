// Cache for the longest searches that have been observed
// Must handle frequent reads and infrequent writes
// Should this be the default?

// TODO: verify none of my code used here can panic!
// don't want to poison lock, which would be bad

use std::collections::{BinaryHeap};
use std::sync::{Arc, RwLock};

use super::cache_elem::CacheElem;
use super::CACHE_SIZE;

#[derive(Debug)]
pub struct LongCacheOuter(Arc<RwLock<LongCacheInner>>);

impl LongCacheOuter {
    pub fn new() -> Self {
        LongCacheOuter(Arc::new(RwLock::new(LongCacheInner::new())))
    }
    fn fast_get(&self) -> Option<Vec<CacheElem>> {
        // try to get the temporary cache
        let rfrn = self.0.clone();
        let lock = rfrn.read().unwrap();
        lock.cache()
    }
    fn slow_get(&self) -> Vec<CacheElem> {
        let rfrn = self.0.clone();
        let mut lock = rfrn.write().unwrap();
        lock.rebuild_cache()
    }
    pub fn get(&self) -> Vec<CacheElem> {
        match self.fast_get() {
            Some(v) => v,
            None => self.slow_get(),
        }
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
    // occasional modifications require
    //  1) insert into middle of the list (maintain in sorted order)
    //  2) pop the shortest element from the list

    // handle efficient lookups/inserts/etc.
    heap: BinaryHeap<CacheElem>,
    temp: Option<Vec<CacheElem>>,
}

impl LongCacheInner {
    fn new() -> Self {
        LongCacheInner {
            heap: BinaryHeap::with_capacity(CACHE_SIZE),
            temp: None,
        }
    }
    fn min_len(&self) -> Option<u8> {
        self.heap.peek().map(|e| e.len())
    }
    fn should_insert(&self, new: &CacheElem) -> bool {
        if self.heap.len() < CACHE_SIZE {
            true
        } else if let Some(shortest) = self.heap.peek() {
            new.len() > shortest.len() 
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
        self.temp = Some(iter.rev().map(|e| (*e).clone()).collect());
        match self.temp {
            Some(ref v) => v.clone(),
            None => unreachable!(),
        }
    }
    fn insert_elem(&mut self, elem: CacheElem) {
        // ONLY call iff should_insert
        assert!(self.should_insert(&elem));
        // Linear search: only add iff necessary
        // TODO: change data structure so this is faster?
        // this op is pretty uncommon, and there are max ~15 elements
        if self.heap.iter().any(|e| e == &elem) {
            return;
        }
        self.temp = None;
        self.heap.push(elem);
        if self.heap.len() > CACHE_SIZE {
            self.heap.pop();
        }
    }
    fn cache(&self) -> Option<Vec<CacheElem>> {
        self.temp.clone()
    }
}
