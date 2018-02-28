
/*
/// A wiki entry, including its title, parents, and children
/// The parent and child sets have a fair amount of overlap, so instead of storing both they're
/// both put in the `neighbors` field (parents first, then children), of which both parents and
/// children are subsets
/// Note that the indices are stored as `u16`s, meaning if an entry has more than 65k parents or
/// children then it will cause problems.
#[derive(Serialize, Deserialize)]
pub struct TableEntry {
    pub title: String,
    neighbors: Vec<u32>,
    last_parent: u32,
    first_child: u32,
}

use link_state::link_data::IndexedEntry;
impl From<IndexedEntry> for Entry {
    fn from(i: IndexedEntry) -> Entry {
        Entry {
            title: i.title,
            neighbors: i.neighbors,
            last_parent: i.last_parent,
            first_child: i.first_child,
        }
    }
}

impl Entry {
    #[inline]
    pub fn get_children(&self) -> &[u32] {
        let i = self.first_child as usize;
        &self.neighbors[i..]
    }
    #[inline]
    pub fn get_parents(&self) -> &[u32] {
        let i = self.last_parent as usize;
        &self.neighbors[..i]
    }
}
*/


