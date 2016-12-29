extern crate regex;
use std::collections::HashMap;

//helpers

//Anything a page_id can represent
#[derive(Debug)]
enum Entry {
    //either a unique page and its link data
    Page { title: String, children: Vec<u32>, parents: Vec<u32> },
    // or the page_id of what it redirects to (which might not be known yet)
    Redirect(Option<u32>),
}

impl Entry {
    fn new_page(n: &str) -> Entry {
        Entry::Page { title: String::from(n), children: Vec::new(), parents: Vec::new() }
    }
}

//The id numbers associated with an article title
#[derive(Debug)]
enum Address {
    //Can be its true page_id
    PageId(u32),
    //Or can be a list of redirects
    //Once the true page_id is known, though, all redirects (and this) must be updated
    Redirects(Vec<u32>),
}


//The actual data storing the internal link structure
pub struct Database {
    // when populating the entries/addresses fields, we may come across redirect pages
    // which means that multiple addresses (u32) can map to the same page
    
    // Address  →  Page
    entries: HashMap<u32,Entry>,
    //  Title   →  Address
    addresses: HashMap<String,Address>,
}

impl Database {
    pub fn new() -> Database {
        Database{ 
            entries:   HashMap::new(), 
            addresses: HashMap::new(),
        }
    }
    pub fn add_page(&mut self, data: &regex::Captures) { 
        let page_id: u32 = data.at(1).unwrap().parse().unwrap();
        let title = data.at(2).unwrap();
        let is_redr = data.at(3).unwrap();
        if is_redr == "1" {
            //get a handle to the address this title points to
            //if it's not already in our db, add it and make note of this redirect
            let addr = self.addresses.entry(String::from(title))
                                     .or_insert(Address::Redirects(vec![]));
            //assuming it was already accounted for:
            // if we know its address, point page_id's entry at it
            // otherwise, add page_id to its list of redirects that must be updated

            if let &mut Address::PageId(true_id) = addr { 
                // `true_id -> title` is already in the database
                self.entries.insert(page_id, Entry::Redirect(Some(true_id)));
            } else if let &mut Address::Redirects(ref mut v) = addr {
                // other pages pages have already tried to redirect to `title`
                v.push(page_id);
            } else {
                //we've never seen `title` before, so we added addrs[title] = Redir([id])
                self.entries.insert(page_id, Entry::Redirect(None));
            }
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
            self.addresses.insert(String::from(title), Address::PageId(page_id));
            self.entries.insert(page_id, Entry::new_page(title));
        }
    }
    pub fn add_redirect(&mut self, data: &regex::Captures) { 
        panic!("TODO: get rid of this code segment (I think)");
        //let src_id: u32 = data.at(1).unwrap().parse().unwrap();
        //let dst: &str = data.at(2).unwrap();
        //if let Some(dst_id) = self.addresses.get(dst) {
        //    //redirects to something we care about
        //    self.redirects.insert(src_id, Some(*dst_id));
        //}
    }
    pub fn add_pagelink(&mut self, data: &regex::Captures) {
        let src_id: u32 = data.at(1).unwrap().parse().unwrap();
        let dst: &str = data.at(2).unwrap();

        if let Some(&Address::PageId(dst_id)) = self.addresses.get(dst) {
            //it is not uncommon (>20%) for the destination title to be absent
            //I think this means there are broken links / links to other wikis / namespaces
            //pretty sure they should just be discarded
            
            if let Some(&mut Entry::Page{children: ref mut c,..}) = self.entries.get_mut(&src_id) {
                //add parent-to-child link
                c.push(dst_id);
            } else {
                // src id was not found, then there's nothing left to do
                return;
            }
            if let Some(&mut Entry::Page{parents: ref mut p, ..}) = self.entries.get_mut(&dst_id) {
                //add child-to-parent link
                p.push(src_id);
            }
        }
    }
    pub fn print(&self) {
        let mut children = 0;
        let mut originals = 0;
        let mut redir_some = 0;
        let mut redir_none = 0;
        for (_,entry) in &self.entries {
            if let &Entry::Page{ children: ref c, parents: ref p, ..} = entry {
                children += c.len();
                originals += 1;
            } else if let &Entry::Redirect(r) = entry {
                if r.is_some() {
                    redir_some += 1;
                } else {
                    redir_none += 1;
                }
            }
        }
        println!("=============================================");
        println!("Number of addresses: {}", self.addresses.len());
        println!("Number of entries:   {}", self.entries.len());
        println!("Number of children: {}", children);
        println!("Number of Real Entries: {}", originals);
        println!("Number of Redirects: {}", redir_some + redir_none);
        println!(" Number of redirects with no destination: {}", redir_none);
        //println!(" Entries: `{:?}`", self.entries);
        //println!(" Addresses: `{:?}`", self.addresses);
        println!("=============================================");

    }
}


