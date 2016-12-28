extern crate regex;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct Page {
    title: String,
    children: Vec<u32>,
    parents: Vec<u32>,  //innovation!
    redirect: bool,
}

#[allow(dead_code)]
impl Page {
    fn new() -> Page {
        Page::from("", false)
    }
    fn from(n: &str, redr: bool) -> Page {
        Page{ 
            title: String::from(n), 
            children: Vec::new(), 
            parents: Vec::new(),
            redirect: redr,
        }
    }
}

enum Entry {
    Page {
        title: String,
        children: Vec<u32>,
        parents: Vec<u32>,
    },
    Redirect(Option<u32>),
    //Unknown,
}

impl Entry {
    fn new_page(n: &str) -> Entry {
        Entry::Page {
            title: String::from(n),
            children: Vec::new(),
            parents: Vec::new(),
        }
    }
}

enum Address {
    Page_ID(u32),
    Redirects(Vec<u32>),
}


#[allow(dead_code)]
pub struct Database {
    // when populating the entries/addresses fields, we may come across redirect pages
    // which means that multiple addresses (u32) can map to the same page
    
    // Address  →  Page
    //entries: HashMap<u32,Option<Page>>,
    entries: HashMap<u32,Entry>,
    //  Title   →  Address
    addresses: HashMap<String,Address>,
    // Replace redirects quickly
    //redirects: HashMap<u32,u32>,
}

#[allow(dead_code)]
impl Database {
    pub fn new() -> Database {
        Database{ 
            entries:   HashMap::new(), 
            addresses: HashMap::new(),
            //redirects: Some(HashMap::new()),
        }
    }
    pub fn add_page(&mut self, data: &regex::Captures) { 
        let page_id: u32 = data.at(1).unwrap().parse().unwrap();
        let title = data.at(2).unwrap();
        let is_redr = data.at(3).unwrap();
        if is_redr == "1" {
            {
                let addr = self.addresses.get_mut(title);
                if let Some(&mut Address::Page_ID(true_id)) = addr {
                    //the destination is already in our database
                    //we just need to point to it
                    self.entries.insert(page_id, Entry::Redirect(Some(true_id)));
                } else if let Some(&mut Address::Redirects(ref mut v)) = addr {
                    //the destination already has some redirects pointing to it
                    //'entries' db will be updated in the other block
                    v.push(page_id);
                } else {
                    //no record of an article with this name
                    //create one of type Redirect so when it gets added the entry at page_id can be
                    //modified to point to it
                    self.entries.insert(page_id, Entry::Redirect(None));
                }
            }

            //if let Some(&Address::Redirects(ref mut v)) = self.addresses.get(title) {
            //    v.push(page_id);
            //} else if let Some(&Address::Page_ID(true_id)) = self.addresses.get(title) {

            //}

            //make a note that this title has been seen
            self.addresses.insert(String::from(title), Address::Redirects(vec![page_id]));
            //point this address at a redirect
            //self.entries.insert(page_id, Entry::Redirect(None));
        } else {
            //this is not a redirect; it's a real article
            //check if we've already seen any redirects that point to this title
            //if we've never seen it before, just add the page_id and be done
            if let Some(&Address::Redirects(ref v)) = self.addresses.get(title) {
                //entry had redirects we have to deal with; set their destinations.
                //`v` is a list of addresses which are of type Entry::Redirect(None)
                //they need to be set to Entry::Redirect(page_id)
                for redirect in v {
                    self.entries.insert(*redirect, Entry::Redirect(Some(page_id)));
                }
            }
            //now insert this valid address/article into the tables
            self.addresses.insert(String::from(title), Address::Page_ID(page_id));
            self.entries.insert(page_id, Entry::new_page(title));
        }
    }
    pub fn add_redirect(&mut self, data: &regex::Captures) { 
        let src_id: u32 = data.at(1).unwrap().parse().unwrap();
        let dst: &str = data.at(2).unwrap();
        //if let Some(dst_id) = self.addresses.get(dst) {
        //    //redirects to something we care about
        //    self.redirects.insert(src_id, Some(*dst_id));
        //}
    }
    pub fn add_pagelink(&mut self, data: &regex::Captures) {
        let src_id: u32 = data.at(1).unwrap().parse().unwrap();
        let dst: &str = data.at(2).unwrap();
        //if let Some(rdr_entry) = self.redirects.get_mut(&src_id) {
        //    //if 
        //}
        if let Some(&Address::Page_ID(dst_id)) = self.addresses.get(dst) {
            //it is not uncommon (>20%) for the destination title to be absent
            //I think this means there are broken links / links to other wikis / namespaces
            //pretty sure they should just be discarded
            
            if let Some(&mut Entry::Page{children: ref mut c,..}) = self.entries.get_mut(&src_id) {
                //add parent-to-child link
                //src_entry.children.push(*dst_id);
                c.push(dst_id);
            } else {
                // src id was not found, then there's nothing left to do
                return;
            }
            if let Some(&mut Entry::Page{parents: ref mut p, ..}) = self.entries.get_mut(&dst_id) {
                //add child-to-parent link
                //dst_entry.parents.push(src_id);
                p.push(src_id);
            }
        }
    }
    pub fn print(&self) {
        let mut children = 0;
        let mut parents = 0;
        for (_,entry) in &self.entries {
            if let &Entry::Page{ children: ref c, parents: ref p, ..} = entry {
                children += c.len();
                parents += p.len();
            }
        }
        println!("=============================================");
        println!("Number of addresses: {}", self.addresses.len());
        println!("Number of entries:   {}", self.entries.len());
        println!("Number of parents:  {}", parents);
        println!("Number of children: {}", children);
        //println!("Number of redirects: {}", self.redirects.len());
        println!("=============================================");

    }
}


