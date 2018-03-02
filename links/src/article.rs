
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PageId(u32);

impl From<PageId> for u32 { 
    fn from(id: PageId) -> u32 { id.0 } 
}
impl From<u32> for PageId { 
    fn from(val: u32) -> PageId { PageId(val) } 
}

/// Entry that refers to its children by their `page_id`
pub type Entry = GenEntry<PageId>;


/// A wiki entry, including its title, parents, and children
/// The parent and child sets have a fair amount of overlap, so instead of storing both they're
/// both put in the `neighbors` field (parents first, then children), of which both parents and
/// children are subsets
/// Note that the indices are stored as `u16`s meaning if an entry has more than 65k parents 
/// or children then it will cause problems.
#[derive(Clone, Debug, Serialize, Deserialize)] // TODO remove clone
pub struct GenEntry<T: From<u32>> {
    pub title: String,
    pub page_id: PageId,
    neighbors: Vec<T>,
    last_parent: u32,
    first_child: u32,
}


impl<T: From<u32>> GenEntry<T> {
    #[inline]
    pub fn get_children(&self) -> &[T] {
        let i = self.first_child as usize;
        &self.neighbors[i..]
    }
    #[inline]
    pub fn get_parents(&self) -> &[T] {
        let i = self.last_parent as usize;
        &self.neighbors[..i]
    }

    /// Convert from GenEntry<T> to GenEntry<U>
    pub fn map<F: Fn(T)->U, U: From<u32>>(self, f: F) -> GenEntry<U> {
        GenEntry {
            title:          self.title,
            page_id:        self.page_id,
            last_parent:    self.last_parent,
            first_child:    self.first_child,
            neighbors:      self.neighbors.into_iter().map(f).collect()
        }
    }
}

impl Entry {
    pub fn from_integers<T>(id: T, t: String, parents: Vec<T>, children: Vec<T>) -> Self
        where PageId: From<T>
    {
        //let id = id.into();
        let parents: Vec<PageId> = parents.into_iter().map(PageId::from).collect();
        let children: Vec<PageId> = children.into_iter().map(PageId::from).collect();
        Self::from(id.into(), t, parents, children)
    }
    pub fn from(id: PageId, t: String, parents: Vec<PageId>, children: Vec<PageId>) -> Self {
        use std::collections::HashSet;
        let parent_set: HashSet<PageId> = parents.iter().cloned().collect();
        assert_eq!(parent_set.len(), parents.len(), "Entry `{}`", t);
        assert!(parents.len() < u32::max_value() as usize, 
                "Entry `{}` has {} parents", t, parents.len());
        let child_set: HashSet<PageId> = children.iter().cloned().collect();
        assert_eq!(child_set.len(), children.len(), "Entry `{}`", t);
        assert!(children.len() < u32::max_value() as usize, 
                "Entry `{}` has {} children", t, children.len());
        let last_parent = parents.len() as u32;
        let num_children = children.len();
        let parents_hm: HashSet<PageId> = parents.iter().cloned().collect();
        let common: HashSet<PageId> = children.iter().cloned().filter(|i| {
            parents_hm.contains(i)
        }).collect();
        let unique_pars =  parents.into_iter().filter(|i| !common.contains(i));
        let unique_kids = children.into_iter().filter(|i| !common.contains(i));
        let neighbors: Vec<PageId> = unique_pars
            .chain(common.iter().cloned())
            .chain(unique_kids).collect();
        let first_child = (neighbors.len() - num_children) as u32;

        Entry {
            title: t,
            page_id: id,
            neighbors, last_parent, first_child
        }
    }
}
