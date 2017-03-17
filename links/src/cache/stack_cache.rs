
use std::collections::{HashSet, LinkedList, BTreeSet, BinaryHeap, VecDeque};
use std::cmp::{Ord, PartialOrd, Ordering};
use std::sync::{Mutex, RwLock};

const CACHE_SIZE: usize = 16;

pub struct StackCache<'a> {
    recent: Mutex<RecentCache<'a>>,

    // Very common read op: check smallest length
    // Quite common: random insert, pop last element
    length: RwLock<LengthCache<'a>>,
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
    set:   HashSet<&'a CacheElem<'a>>,

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

#[derive(Debug, Serialize, PartialEq, Eq, Hash)]
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




