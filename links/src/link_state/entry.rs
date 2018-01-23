#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub title: String,
    //neighbors: Vec<u32>,
    parents: Vec<u32>,
    children: Vec<u32>,
}

impl Entry {
    pub fn from(title: String, parents: Vec<u32>, children: Vec<u32>) -> Self {
        Entry { title, parents, children, }
    }
    pub fn get_children(&self) -> &[u32] {
        &self.children[..]
    }
    pub fn get_parents(&self) -> &[u32] {
        &self.parents[..]
    }
}


