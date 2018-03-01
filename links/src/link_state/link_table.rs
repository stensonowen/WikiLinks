//! Custom structure for article hierarchy
//! Instead of a hash table to map `page_id` to `Entry`, store `Entry`s in an
//! array and refer to them by their index. This minimizes the lookup time that
//! might be otherwise spent doing hashmap open addressing collision handling.
//! Should make bfs slightly faster at the expense of slightly slower initial
//! lookup times and slightly more memory (maybe?)

use std::ops::Index;

// could theoretically use IntegerHashMap, but probably is more important for
// this to be polished than quite so fast
use fnv::FnvHashMap;

use article::{PageId, Entry};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PageIndex(u32);


pub struct LinkTable {
    page_ids: FnvHashMap<PageId, PageIndex>,
    table: Box<[Entry]>,
}

impl LinkTable {
    pub fn from_map(old: FnvHashMap<PageId, Entry>) -> Self {
        // TODO sort in some way? maybe improve caching a little?

        let table_vec: Vec<Entry> = old.into_iter().map(|(_,e)| e).collect();
        let table: Box<[Entry]> = table_vec.into_boxed_slice(); // "drops excess capacity"
        assert!(table.len() < u32::max_value() as usize, "Table too long; use `u64`s");

        let page_ids = table.iter().enumerate().map(|(i, ref te)| {
            (te.page_id, PageIndex(i as u32))
        }).collect::<FnvHashMap<PageId, PageIndex>>();

        LinkTable { page_ids, table }
    }
    fn get_index(&self, id: PageId) -> Option<PageIndex> {
        self.page_ids.get(&id).map(|&pi| pi)
    }
    pub fn len(&self) -> usize {
        self.table.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /*
    fn get_by_index(&self, i: PageIndex) -> Option<&Entry> {
        self.table.get(i.0 as usize)
    }
    fn get_by_pageid(&self, id: PageId) -> Option<&Entry> {
        let index = self.page_ids.get(&id)?;
        self.table.get(index.0 as usize)
    }
    */
}

impl Index<PageId> for LinkTable {
    type Output = Entry;
    fn index(&self, id: PageId) -> &Entry {
        let table_index = &self.page_ids[&id];
        &self.table[table_index.0 as usize]
    }
}

impl Index<PageIndex> for LinkTable {
    type Output = Entry;
    fn index(&self, index: PageIndex) -> &Entry {
        &self.table[index.0 as usize]
    }
}

