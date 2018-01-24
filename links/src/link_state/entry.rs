#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub title: String,
    neighbors: Vec<u32>,
    last_parent: u16,
    first_child: u16,
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
    pub fn get_children(&self) -> &[u32] {
        let i = self.first_child as usize;
        &self.neighbors[i..]
    }
    pub fn get_parents(&self) -> &[u32] {
        let i = self.last_parent as usize;
        &self.neighbors[..i]
    }
}


