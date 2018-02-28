use fnv;

use article::{PageId, Entry};

#[derive(Debug, Clone)]
pub struct Path {
    pub src: PageId,
    pub dst: PageId,
    pub path: Result<Vec<PageId>, PathError>,
}

#[derive(Debug, Clone)]
pub enum PathError {
    NoSuchPath,
    Terminated(usize)
}

impl Path {
    pub fn len(&self) -> Option<usize> {
        if let Ok(ref v) = self.path {
            Some(v.len()-1)
        } else {
            None
        }
    }
    pub fn is_empty(&self) -> bool {
        if let Ok(ref v) = self.path {
            v.len() <= 1
        } else {
            false
        }
    }
    pub fn print(&self, entries: &fnv::FnvHashMap<PageId, Entry>) {
        println!("Path from {:?}\t(\"{}\")", self.src, entries.get(&self.src).unwrap().title);
        println!("\t  to {:?}\t(\"{}\") :", self.dst, entries.get(&self.dst).unwrap().title);
        match self.path {
            Ok(ref v) => for i in v {
                println!("\t{:?}:\t\"{}\"", i, entries[i].title);
            },
            Err(PathError::NoSuchPath) => println!("\tNo such path exists"),
            Err(PathError::Terminated(i)) => 
                println!("\tSearch expired after {} iterations", i),
        }
        println!("\tlen = {:?}", self.len());
    }
}

