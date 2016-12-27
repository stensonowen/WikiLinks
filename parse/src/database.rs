extern crate regex;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(PartialEq, Eq, Hash)]
struct Address {
    ns: i16,    // page_namespace
    id: u32,    // page_id
}

impl Address {
    fn from(n: i16, i: u32) -> Address {
        Address{ ns: n, id: i }
    }
}

#[allow(dead_code)]
pub struct Page {
    title: String,
    children: Vec<Address>,
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
    entries: HashMap<Address,Page>,
    //  Title   →  Address
    addresses: HashMap<String,Address>,
}

#[allow(dead_code)]
impl Database {
    pub fn new() -> Database {
        Database{ entries: HashMap::new(), addresses: HashMap::new() }
    }
    pub fn add_page(&mut self, data: &regex::Captures) { 
        let src_id: u32 = data.at(1).unwrap().parse().unwrap();
        let src_ns: i16 = data.at(2).unwrap().parse().unwrap();
        let dst = data.at(3).unwrap();
        let _redr = data.at(4).unwrap() == "1";
        if self.addresses.contains_key(dst) {
            //why does this happen? It occurred ~10% of the time
            //it's not just titles with spaces/underscores or with unknown utf-8 chars
            println!("DUPLICATE: `{}`", dst);
        }
        self.addresses.insert(String::from(dst), Address::from(src_ns, src_id));
    }
    pub fn add_redirect(&self, data: &regex::Captures) { 
        let _src_id: u32 = data.at(1).unwrap().parse().unwrap();
        let _src_ns: i16 = data.at(2).unwrap().parse().unwrap();
        let _dst: &str = data.at(3).unwrap();
    }
    pub fn add_pagelink(&self, data: &regex::Captures) {
        let _src_id: u32 = data.at(1).unwrap().parse().unwrap();
        let _src_ns: i16 = data.at(2).unwrap().parse().unwrap();
        let _dst: &str = data.at(3).unwrap();
    }
    pub fn len(&self) -> usize {
        self.addresses.len()
    }
}
