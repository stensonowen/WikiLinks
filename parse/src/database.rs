extern crate regex;
use std::collections::{HashMap, HashSet};

use slog;
//pub type PAGE_ID = u32;

// helpers
/*
 * If a Entry::Redirect only contains its destination (and no parents/children),
 *  and a BFS that touches it just touches its children in 0 steps, 
 *  will there be errors caused by the dst searching "up" for the src through it?
 */

// An Entry is anything a page_id can represent
#[derive(Debug)]
enum Entry {
    // either a unique page and its link data
    Page {
        title: String,
        children: Vec<u32>,
        parents: Vec<u32>,
    },
    // or the redirect page and its address
    // these will eventually be taken out of db::entries
    Redirect {
        title: String,
        target: Option<u32>,
    }
}

impl Entry {
    fn new_page(n: &str) -> Entry {
        Entry::Page {
            title: String::from(n),
            children: Vec::new(),
            parents: Vec::new(),
        }
    }
    fn new_redir(n: &str) -> Entry {
        Entry::Redirect {
            title: String::from(n),
            target: None,
        }
    }
}

// An Address can point to either a real page or a redirect
#[derive(Debug)]
enum Address {
    Page(u32),
    Redirect(u32),
}

//what phase the database is in 
//TODO should I get rid of the numbers? They don't matter except that without 
// them it might not be clear that the order of the values is what determines
// their inequality; here concrete values make it clearer
#[derive(PartialEq, PartialOrd, Debug)]
enum State {
    Begin           = 0,
    AddPages        = 1,
    AddRedirects    = 2,
    AddLinks        = 3,
    Done            = 4,
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
    //internal state
    state: State,
    //logging
    log: slog::Logger,
}

impl Database {
    pub fn new(log: slog::Logger) -> Database {
        Database {
            entries: HashMap::new(),
            addresses: HashMap::new(),
            state: State::Begin,
            log: log,
        }
    }
    pub fn add_page(&mut self, data: &regex::Captures) {
        //must finish before links/redirects start
        assert!(self.state <= State::AddPages, 
                "Tried to add page in the wrong stage: `{:?}`", self.state);
        if self.state == State::Begin {
            info!(self.log, "Entering the `AddPages` phase");
            self.state = State::AddPages;
        }

        let page_id: u32 = data.at(1).unwrap().parse().unwrap();
        let title = data.at(2).unwrap();
        let is_redr = data.at(3).unwrap() == "1";

        assert!(self.entries.contains_key(&page_id) == false,
                "Tried to add an entry at a claimed location: {}",
                page_id);

        let (entry, addr) = if is_redr {
            (Entry::new_redir(title), Address::Redirect(page_id))
        } else {
            (Entry::new_page(title), Address::Page(page_id))
        };
        self.entries.insert(page_id, entry);
        self.addresses.insert(String::from(title), addr);
    }
    pub fn add_redirect(&mut self, data: &regex::Captures) {
        // must occur after all pages have been added and before any links
        assert!(self.state == State::AddPages || self.state == State::AddRedirects, 
                "Tried to add a redirect in the `{:?}` stage", self.state);
        if self.state == State::AddPages {
            info!(self.log, "Entering the `AddRedirects` stage");
            self.state = State::AddRedirects;
        }

        let src_id: u32 = data.at(1).unwrap().parse().unwrap();
        let dst: &str = data.at(2).unwrap();

        //the redirect's target title *should* already be in self.addresses
        //if it is, its address should be of type Redirect(None), 
        // which we can update to Redirect(true_page_id)
        // The Redirect
        //We can get the redirect's target title from self.entries and its 
        
        // will be:
        //  entries[redir_u32] = Entries::Redirect( redir_title, target=None )
        //  entries[ true_u32] = Entries::Page( true_title, p=[], c=[] )
        //  addresses[redir_title] = Addresses::Redirect( ??? )
        //  addresses[ true_title] = Addresses::Page(true_u32)
        //
        // should be: 
        //  entries[redir_u32] = Entries::Redirect( redir_title, target=Some(true_u32) )
        //  entries[ true_u32] = Entries::Page( true_title, p=[], c=[] )
        //  addresses[redir_title] = Addresses::Redirect(redir_u32)
        //  addresses[ true_title] = Addresses::Page(true_u32)
        //
        //  What if addresses[redir_title] = Addresses::Page(true_u32) ?
        //  then we couldn't get a handle to the redirect_u32
        //  So I'm pretty sure we never actually need an Address::Redirect




        if let Some(&mut Address::Page(dst_id)) = self.addresses.get_mut(dst) {
            if let Some(&mut Entry::Redirect{ target: ref mut t, .. }) 
                    = self.entries.get_mut(&dst_id) {
                if let &mut Some(id) = t {
                    warn!(self.log, "A redirect's page_id already pointed to something");
                } else {
                    *t = Some(src_id);
                }
            } else {
                warn!(self.log, "A redirect's page_id was not marked as such");
            }
        } else {
            warn!(self.log, "Redirect to `{}` from `{}` could not be found", dst, src_id); 
        }

    }
    pub fn add_pagelink(&mut self, data: &regex::Captures) {
        let src_id: u32 = data.at(1).unwrap().parse().unwrap();
        let dst: &str = data.at(2).unwrap();
        /*

        if let Some(&Address::Page(dst_id)) = self.addresses.get(dst) {
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
    */
        /*
        else {
            assert!(self.addresses.get(dst).is_none(),
                    "Looking up a destination by its title pointed to a redirect :O");
        }*/
    }
    /*
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
    }*/
    /*
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
    */
    /*
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
            if let &Address::Page(page_id) = addr {
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
    */
}
