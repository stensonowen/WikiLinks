use std::cmp::{Ord, PartialOrd, Ordering};

#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone)]
pub struct CacheElem {
    // TODO: will LLVM optimize equality comparison to compare numbers first?
    // or should we implement PartialEq ourselves?
    // TODO: investigate using &'a str instead of owned members
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
            (&self.src, &self.dst).cmp(&(&othr.src, &othr.dst))
        }
    }
}

impl CacheElem {
    pub fn new(s: &str, d: &str, l: usize) -> Self {
        Self {
            src: s.to_owned(),
            dst: d.to_owned(),
            len: l as u8,
        }
    }
    pub fn len(&self) -> u8 {
        self.len
    }
}


//  ---------------------------




