
use article::PageId;

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
    pub fn print2<F: Fn(&PageId)->String>(&self, get_title: F) {
        let src_title = get_title(&self.src);
        let dst_title = get_title(&self.dst);
        println!("Path from {:?}\t(\"{}\")", self.src, src_title);
        println!("\t  to {:?}\t(\"{}\") :", self.dst, dst_title);
        match self.path {
            Ok(ref v) => for i in v {
                println!("\t{:?}:\t\"{}\"", i, get_title(i));
            },
            Err(PathError::NoSuchPath) => println!("\tNo such path exists"),
            Err(PathError::Terminated(i)) => 
                println!("\tSearch expired after {} iterations", i),
        }
        println!("\tlen = {:?}", self.len());
    }
}

