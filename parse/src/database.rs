extern crate regex;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct Page {
    title: String,
    children: Vec<u32>,
}

#[allow(dead_code)]
impl Page {
    fn new() -> Page {
        Page{ title: String::new(), children: Vec::new() }
    }
}

#[allow(dead_code)]
pub struct Database {
    // Address  →  Page
    entries: HashMap<u32,Page>,
    //  Title   →  Address
    addresses: HashMap<String,u32>,
}

#[allow(dead_code)]
impl Database {
    pub fn new() -> Database {
        Database{ entries: HashMap::new(), addresses: HashMap::new() }
    }
    pub fn add_page(&mut self, data: &regex::Captures) { 
        let src_id: u32 = data.at(1).unwrap().parse().unwrap();
        let dst = data.at(2).unwrap();
        let _redr = data.at(3).unwrap() == "1";
        self.addresses.insert(String::from(dst), src_id);
    }
    pub fn add_redirect(&self, data: &regex::Captures) { 
        let _src_id: u32 = data.at(1).unwrap().parse().unwrap();
        let _dst: &str = data.at(2).unwrap();
    }
    pub fn add_pagelink(&self, data: &regex::Captures) {
        let _src_id: u32 = data.at(1).unwrap().parse().unwrap();
        let _dst: &str = data.at(2).unwrap();
    }
    pub fn len(&self) -> usize {
        self.addresses.len()
    }
}
