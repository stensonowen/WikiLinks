
use std::collections::{HashSet, HashMap, LinkedList, BTreeSet, BinaryHeap, VecDeque};
use std::cmp::{Ord, PartialOrd, Ordering};
use std::sync::{Mutex, RwLock};

const CACHE_SIZE: usize = 16;

pub struct StackCache<'a> {
    recent: Mutex<RecentCache<'a>>,

    // Very common read op: check smallest length
    // Quite common: random insert, pop last element
    length: RwLock<LengthCache<'a>>,
}

impl<'a> StackCache<'a> {
    //fn insert_and_get_recent(&'a mut self, elem: CacheElem<'a>) -> &'a [CacheElem<'a>] {
    //fn insert(&'a mut self, elem: CacheElem<'a>) {
    fn insert(&'a mut self, elem: CacheElem<'a>) {
        // always (try to) update recent list
        if let Ok(r) = self.recent.get_mut() {
            r.insert_elem(elem);
        }
        // if pertinent, update long list
    }
    /*
    fn update_and_get_mut(&'a mut self, elem: CacheElem<'a>) -> Vec<CacheElem<'a>> {
        if let Ok(r) = self.recent.get_mut() {
            r.insert_elem(elem)
        }
        if let Ok(r) = self.recent.lock() {
            //r.get().clone().to_vec()
            vec![]
        } else {
            vec![]
        }
    }*/
    //fn insert_and_get_recent(&'a mut self, elem: CacheElem<'a>) -> Vec<CacheElem<'a>> {
    //fn get_recent(&'a mut self) -> Vec<CacheElem<'a>> {
    fn get_recent(&'a self) -> Vec<CacheElem<'a>> {
        let r = self.recent.lock().unwrap();
        //r.get().to_vec()
        unimplemented!()
    }
    //fn get_longest(&'a self) -> &'a [&'a CacheElem<'a>] {
    fn get_longest(&'a self) -> Vec<&'a CacheElem<'a>> {
        let l = self.length.read().unwrap();
        l.get().to_vec()
    }
}

struct RecentCache<'a> {
    // keep track of all cached elements that appeared most recently
    // every page load will require 
    //  1) look up a search result in this collection
    //  2) get a list of the cache elements in order
    //  3) add a search result to the top
    //  4) remove a search result from the bottom (or maybe the middle)

    // Very common read op: check if search is contained
    // Most common: pop from the top and push to the bottom
    // Somewhat common: move around
    
    //list: LinkedList<CacheElem<'a>>,
    //set:  HashSet<&'a CacheElem<'a>>,
    //temp: Option<Vec<&'a CacheElem<'a>>>,

    queue: VecDeque<CacheElem<'a>>,
    // rely on VecDeque.contains() every iteration? Or use a redundant hashset?
    //set:   HashSet<&'a CacheElem<'a>>,
    contents:   HashSet<&'a CacheElem<'a>>,
    //contents: HashMap<(u32,u32), &'a CacheElem<'a>>,
    //contents: HashMap<CacheElem<'a>, &'a CacheElem<'a>>,
}

struct LengthCache<'a> {
    // keep track of the longest searches we've seen
    // every page load will require
    //  1) lookup of the shortest value contained (can be worked around)
    //  2) get a list of the cache elements in order
    // occasional modifications require
    //  1) insert into middle of the list (maintain in sorted order)
    //  2) pop the shortest element from the list

    // handle efficient lookups/inserts/etc.
    heap: BinaryHeap<CacheElem<'a>>,
    temp: Option<Vec<&'a CacheElem<'a>>>,
}

impl<'a> LengthCache<'a> {
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
    pub fn get(&'a mut self) -> &[&CacheElem] {
        match self.temp {
            Some(ref v) => v,
            None => self.rebuild_cache(),
        }
    }
    fn rebuild_cache(&'a mut self) -> &[&CacheElem] {
        let iter = self.heap.iter();
        self.temp = Some(iter.collect());
        match self.temp {
            Some(ref v) => v,
            None => unreachable!(),
        }
    }
    fn insert_elem(&mut self, elem: CacheElem<'a>) {
        // ONLY call iff should_insert
        self.temp = None;
        self.heap.push(elem);
        if self.heap.len() > CACHE_SIZE {
            self.heap.pop();
        }
    }
}

impl<'a> RecentCache<'a> {
    //fn get(&self) -> &[CacheElem<'a>] {   // elides to what?
    //pub fn get(&'a self) -> &'a [CacheElem<'a>] {
    pub fn get(&self) -> &[CacheElem] {
        // we never `push_front()`, so the contents will always be in the first array
        // I think
        let slices = self.queue.as_slices();
        assert!(slices.1.is_empty());
        slices.0
    }
    //pub fn insert_elem(&'a mut self, elem: CacheElem<'a>) {
    pub fn insert_elem(&'a mut self, elem: CacheElem<'a>) {
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
        self.contents.insert(self.queue.get(index).unwrap());
        //assert_eq!(Some(&elem), self.queue.get(index));
    }
}

/*
impl<'a> StackCache<'a> {
    fn blank() -> Self {
        StackCache {
            recent_ll: LinkedList::new(),
            recent_hs: HashSet::new(),
            recent_tmp:None,
            length: Vec::new(),
        }
    }

    pub fn get_recent(&'a mut self) -> &Vec<&'a CacheElem<'a>> {
        // get a reference to the `recent` cache
        // rebuild it iff necessary
        if let Some(ref v) = self.recent_tmp {
            v
        } else {
            self.rebuild_recent_cache()
        }
    }
    pub fn get_length(&'a mut self) -> &Vec<CacheElem<'a>> {
        &self.length
    }

    fn insert_long(&mut self, elem: CacheElem<'a>) {
        // remove the shortest element
        // only do this if it's necessary
        // only do this if it's not already in there

    }

    fn rebuild_recent_cache(&'a mut self) -> &Vec<&'a CacheElem<'a>> {
        let iter = self.recent_ll.iter();
        self.recent_tmp = Some(iter.collect());
        match self.recent_tmp {
            Some(ref v) => v,
            None => unreachable!()  // uhhh
        }
    }

}
*/

#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone)]
pub struct CacheElem<'a> {
    // TODO: will LLVM optimize equality comparison to compare numbers first?
    // or should we implement PartialEq ourselves?
    src: &'a str,
    dst: &'a str,
    len: u8,
    // don't bother to store inconclusive searches
}

impl<'a> PartialOrd for CacheElem<'a> {
    fn partial_cmp(&self, othr: &Self) -> Option<Ordering> {
        Some(self.cmp(othr))
    }
}

impl<'a> Ord for CacheElem<'a> {
    fn cmp(&self, othr: &Self) -> Ordering {
        // Sort by length first
        // Longer sizes should be 'less', i.e. first in BTree
        if self.len != othr.len {
            self.len.cmp(&othr.len).reverse()
        } else {
            (self.src, self.dst).cmp(&(othr.src, othr.dst)).reverse()
        }
    }
}


//  ---------------------------




