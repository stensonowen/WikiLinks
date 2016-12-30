extern crate regex;
use std::collections::{HashMap, HashSet};

// helpers

// Anything a page_id can represent
#[derive(Debug)]
enum Entry {
    // either a unique page and its link data
    Page {
        title: String,
        children: Vec<u32>,
        parents: Vec<u32>,
    },
    // or the page_id of what it redirects to (which might not be known yet)
    Redirect(Option<u32>),
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

// The id numbers associated with an article title
#[derive(Debug)]
enum Address {
    // Can be its true page_id
    PageId(u32),
    // Or can be a list of redirects
    // Once the true page_id is known, though, all redirects (and this) must be updated
    Redirects(Vec<u32>),
}


// The actual data storing the internal link structure
pub struct Database {
    // when populating the entries/addresses fields, we may come across redirect pages
    // which means that multiple addresses (u32) can map to the same page
    //
    // Address  →  Page
    entries: HashMap<u32, Entry>,
    //  Title   →  Address
    addresses: HashMap<String, Address>,
}

impl Database {
    pub fn new() -> Database {
        Database {
            entries: HashMap::new(),
            addresses: HashMap::new(),
        }
    }
    pub fn add_page(&mut self, data: &regex::Captures) {
        let page_id: u32 = data.at(1).unwrap().parse().unwrap();
        let title = data.at(2).unwrap();
        let is_redr = data.at(3).unwrap();
        if is_redr == "1" {
            // get a handle to the address this title points to
            // if it's not already in our db, add it and make note of this redirect
            let addr = self.addresses
                .entry(String::from(title))
                .or_insert(Address::Redirects(vec![]));
            // assuming it was already accounted for:
            // if we know its address, point page_id's entry at it
            // otherwise, add page_id to its list of redirects that must be updated

            if let &mut Address::PageId(true_id) = addr {
                // `true_id -> title` is already in the database
                self.entries.insert(page_id, Entry::Redirect(Some(true_id)));
            } else if let &mut Address::Redirects(ref mut v) = addr {
                // other pages pages have already tried to redirect to `title`
                v.push(page_id);
            } else {
                // we've never seen `title` before, so we added addrs[title] = Redir([id])
                self.entries.insert(page_id, Entry::Redirect(None));
            }
        } else {
            // this is not a redirect; it's a real article
            // check if we've already seen any redirects that point to this title
            // if we've never seen it before, just add the page_id and be done
            if let Some(&Address::Redirects(ref v)) = self.addresses.get(title) {
                //entry had redirects we have to deal with; set their destinations.
                //`v` is a list of addresses which are of type Entry::Redirect(None)
                //they need to be set to Entry::Redirect(page_id)
                for redirect in v {
                    self.entries.insert(*redirect, Entry::Redirect(Some(page_id)));
                }
            }
            // now insert this valid address/article into the tables
            self.addresses.insert(String::from(title), Address::PageId(page_id));
            self.entries.insert(page_id, Entry::new_page(title));
        }
    }
    pub fn add_pagelink(&mut self, data: &regex::Captures) {
        // must occur after all pages have been added
        let mut src_id: u32 = data.at(1).unwrap().parse().unwrap();
        let dst: &str = data.at(2).unwrap();

        if let Some(&Address::PageId(dst_id)) = self.addresses.get(dst) {
            //if the source address is a redirect, use the proper one instead
            if let Some(&Entry::Redirect(Some(redir_id))) = self.entries.get(&src_id) {
                src_id = redir_id;
            }
            //add dst_id to the list of src_id's children
            if let Some(&mut Entry::Page{ children: ref mut c, .. }) = 
                    self.entries.get_mut(&src_id) {
                c.push(dst_id);
            } else {
                println!("Source ID `{}` was not a true page (was a dredirect or None)", 
                         src_id);
            }
            /*else {
                panic!("Src ID was absent OR a redirect pointed to the wrong thing");
            }*/
            //add src_id to the list of dst_id's parents
            if let Some(&mut Entry::Page{ parents: ref mut p, .. }) = 
                    self.entries.get_mut(&dst_id) {
                p.push(src_id);
            } else {
                panic!("Dest ID didn't point to a true page, but a redirect or a none");
            }
        } else {
            println!("Could not find destination `{}`", dst);
        }
        /*
        else {
            assert!(self.addresses.get(dst).is_none(),
                    "Looking up a destination by its title pointed to a redirect :O");
        }*/
    }
    pub fn clean_up(&mut self) {
        //clear memory that we can't use
        
        //delete all Entry::Redirect(None) from self.entries
        let mut chopping_block_e: HashSet<u32> = HashSet::new();
        let mut chopping_block_a: HashSet<String> = HashSet::new();
        for (id,entry) in &self.entries {
            if let &Entry::Redirect(None) = entry {
                chopping_block_e.insert(*id);
            }
        }
        //delete all addresses which are only pointed to by redirects (and the redirects)
        for (title,addr) in &self.addresses {
            if let &Address::Redirects(ref v) = addr {
                for i in v {
                    chopping_block_e.insert(*i);
                }
                chopping_block_a.insert(title.clone()); //can be fixed w/ lifetimes?
            }
        }
        println!("Removing {} entries and {} addresses", 
                 chopping_block_e.len(), chopping_block_a.len());
        //perform the actual deletions
        for e in chopping_block_e {
            self.entries.remove(&e);
        }
        for a in chopping_block_a {
            self.addresses.remove(&a);
        }
        //recapture any memory we can
        self.entries.shrink_to_fit();
        self.addresses.shrink_to_fit();
    }
    pub fn print(&self) {
        let mut children = 0;
        let mut originals = 0;
        let mut redir_some = 0;
        let mut redir_none = 0;
        for (_, entry) in &self.entries {
            if let &Entry::Page { children: ref c, .. } = entry {
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
        println!("=============================================");

    }
    pub fn verify(&self) {
        //make sure everything is following the rules
        for (addr, entry) in &self.entries {
            if let &Entry::Page{ title: ref f, .. } = entry {
                //make sure the title lookup works the other way
                if let Some(a) = self.addresses.get(f) {
                    if let &Address::Redirects(_) = a {
                        panic!("{} points to `{}` in the entries, which points to a redirect",
                               addr, f);
                    }
                } else {
                    panic!("{} points to `{}` in the entries, but `{}` isn't in the addresses",
                           addr, f, f);
                }
            } else if let &Entry::Redirect(Some(page_id)) = entry {
                if let Some(x) = self.entries.get(&page_id) { 
                    if let &Entry::Redirect(_) = x {
                        panic!("A redirect ({}) pointed to another redirect ({})", 
                               addr, page_id);
                    }
                } else {
                    panic!("A redirect ({}) pointed to an id ({}) that was absent", 
                           addr, page_id); 

                }
            }
        }
        for (title, addr) in &self.addresses {
            //if an addr is 
            if let &Address::PageId(page_id) = addr {
                if let Some(&Entry::Page{ title: ref t, .. }) = self.entries.get(&page_id) {
                    assert!(t == title);
                //assert!(self.entries.get(&page_id).unwrap().title == title);
                } else if self.entries.get(&page_id).is_none() {
                    panic!("PageID must point to something"); //right?
                }
            }
        }


        println!("I PASSED! :)");
    }
}
