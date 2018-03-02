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

use article::{PageId, Entry, GenEntry};

/// Reserved Value to be used by IntegerHashMap
// For now has to be in this module, or else it can't be constant
// until constant functions are stabilized (I think)
pub const PAGEINDEX_RESERVED: PageIndex = PageIndex(::std::u32::MAX);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct PageIndex(u32);
impl From<u32> for PageIndex {
    fn from(n: u32) -> PageIndex { PageIndex(n) }
}
impl From<PageIndex> for usize {
    fn from(i: PageIndex) -> usize { i.0 as usize }
}

impl PageIndex {
    fn to_usize(self) -> usize { self.0 as usize }
}

pub type TableEntry = GenEntry<PageIndex>;

pub struct LinkTable {
    page_ids: FnvHashMap<PageId, PageIndex>,
    table: Box<[TableEntry]>,
}

impl LinkTable {
    pub fn get_table(&self) -> &[TableEntry] {
        &self.table
    }

    pub fn convert_from_map(map: FnvHashMap<PageId, Entry>) -> Self {
        let page_ids = map.keys().enumerate().map(|(n, &id)| {
            (id, (n as u32).into())
        }).collect::<FnvHashMap<PageId, PageIndex>>();

        let table_vec: Vec<TableEntry> = map.into_iter().map(|(_, e): (_, Entry)| {
            e.map(|id: PageId| page_ids[&id])
        }).collect();
        let table: Box<[TableEntry]> = table_vec.into_boxed_slice();
        assert!(table.len() < u32::max_value() as usize, "Table too long for `u32`s");

        LinkTable { page_ids, table }
    }
    pub fn get_title<'a>(&'a self, id: &PageId) -> Option<&'a str> {
        let index = self.page_ids.get(id)?;
        let entry = self.table.get(index.to_usize())?;
        Some(&entry.title[..])
    }
    pub fn get_index(&self, id: PageId) -> Option<PageIndex> {
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
    type Output = TableEntry;
    fn index(&self, id: PageId) -> &TableEntry {
        let table_index = &self.page_ids[&id];
        &self.table[table_index.0 as usize]
    }
}

impl Index<PageIndex> for LinkTable {
    type Output = TableEntry;
    fn index(&self, index: PageIndex) -> &TableEntry {
        &self.table[index.0 as usize]
    }
}

