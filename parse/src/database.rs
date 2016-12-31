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
/*
#[derive(Debug)]
enum Address {
    Page(u32),
    Redirect(u32),
}
*/

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
    addresses: HashMap<String, u32>,
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

        if self.entries.contains_key(&page_id) {
            error!(self.log, "Tried to add an entry at a claimed location: {}", page_id);
        }

        let (entry, addr) = if is_redr {
            (Entry::new_redir(title), page_id)
        } else {
            (Entry::new_page(title), page_id)
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

        let redir_u32: u32 = data.at(1).unwrap().parse().unwrap();
        let true_title: &str = data.at(2).unwrap();

        //the redirect's target title *should* already be in self.addresses
        //if it is, its address should be of type Redirect(None), 
        // which we can update to Redirect(true_page_id)
        // The Redirect
        //We can get the redirect's target title from self.entries and its 
        
        // will be:
        //  entries[redir_u32] = Entries::Redirect( redir_title, target=None )
        //  entries[ true_u32] = Entries::Page( true_title, p=[], c=[] )
        //  addresses[redir_title] = true_u32
        //  addresses[ true_title] = true_u32
        //
        // should be: 
        //  entries[redir_u32] = Entries::Redirect( redir_title, target=Some(true_u32) )
        //  entries[ true_u32] = Entries::Page( true_title, p=[], c=[] )
        //  addresses[redir_title] = true_u32
        //  addresses[ true_title] = true_u32

        let entry = self.entries.get_mut(&redir_u32);
        if let Some(&mut Entry::Redirect{ target: ref mut t, ..} ) = entry {
            if let &mut Some(x) = t {
                error!(self.log,
                       "The page_id of a redirect already redirected to page_id `{}`", x);
            }
            if let Some(&true_u32) = self.addresses.get(true_title) {
                //success
                *t = Some(true_u32);
            } else {
                warn!(self.log,
                      "The true_title of a redirect was not in the db: `{}`", true_title);
            }
        } else if entry.is_none() {
            warn!(self.log, "A redirect source ID is missing from the table: {}", redir_u32);
        } else {
            //should not insert in:
            // we know the redirect page_id but not its redirect_title
            // without a redirect_title, no links/redirects have any way of getting a handle
            error!(self.log, 
                   "The source page_id of a redirect was somehow a `{:?}`", entry);
        }


        /*
        if let Some(&true_addr) = self.addresses.get(true_title) {
            if let &mut Entry::Redirect{ target: ref mut t, .. } 
                    = self.entries.entry(redir_u32)
                                  .or_insert(Entry::new_redir(true_title))
            {
            } else {
            }
        } else {
            warn!(self.log,
                  "The true_title of a redirect was not in the db: `{}`", true_title);
        }*/

    }
    pub fn add_pagelink(&mut self, data: &regex::Captures) {
        // only add pagelinks after adding redirects is finished
        assert!(self.state == State::AddRedirects || self.state == State::AddLinks,
                "Tried to add a link in the `{:?}` stage", self.state);
        if self.state == State::AddRedirects {
            info!(self.log, "Entering the `AddLinks` stage");
            self.state = State::AddLinks;
        }

        let mut src_id: u32 = data.at(1).unwrap().parse().unwrap();
        let dst_title: &str = data.at(2).unwrap();

        //BEFORE
        // entries[src_id] = Entry{ children: c, .. }
        // entries[dst_id] = Entry{ parents: p, ..}
        //AFTER
        // entries[src_id] = Entry{ children: c+[dst_id], ..}
        // entries[dst_id] = Entry{ parents: p+[src_id], ..}

        //lookup dst_id from dst_title
        let dst_id = match self.addresses.get(dst_title) {
            Some(&true_id) => true_id,
            None => {
                warn!(self.log, 
                      "A pagelink gave a destination title not in the db: `{}`", dst_title);
                return;
            },
        };

        // if entries[src_id] is a redirect, update src_id to what it should be
        if let Some(&Entry::Redirect{ target: t, .. }) = self.entries.get(&src_id) {
            if let Some(target) = t {
                src_id = target;
            } else {
                warn!(self.log,
                      "A pagelink gave a source id {} that redirected to None", src_id);
                return;
            }
        }

        //update entries[src_id] to have dst_id among its children
        if let Some(&mut Entry::Page{ children: ref mut c, .. }) 
                = self.entries.get_mut(&src_id) {
            c.push(dst_id);                
        } else {
            error!(self.log, 
                   "Either a pagelink or a redirect used an invalid source id {}", src_id);
            return;
        }
        //update entries[dst_id] to have src_id among its parents
        if let Some(&mut Entry::Page{ parents: ref mut p, .. }) 
                = self.entries.get_mut(&dst_id) {
            p.push(src_id);
        } else {
            error!(self.log, 
                   "The dst_id {} given by addresses[dst_title=`{}`] was a Redirect or a None",
                  dst_id, dst_title);
            return;
        }
    }
    pub fn clean_up(&mut self) {
        assert!(self.state == State::AddLinks, 
                "Clean-up must be started after links are added, not in `{:?}`", self.state);
        info!(self.log, "Entering the `Final` stage");
        self.state = State::Done;

        //I think the only thing we need to get rid of is Entry::Redirect{ target: None} s (?)
        let mut chopping_block: HashSet<u32> = HashSet::new();
        for (&addr,entry) in &self.entries {
            if let &Entry::Redirect{ target: None, .. } = entry {
                //do we also have to look these up in self.addresses values?
                chopping_block.insert(addr);
            }
        }
        info!(self.log, "Deleting {} empty redirects from self.entries", chopping_block.len());
    }
    pub fn print(&self) {
        let mut children = 0;
        let mut true_pages = 0;
        let mut redirects = 0;
        let mut blanks = 0;
        for (_,entry) in &self.entries {
            if let &Entry::Page{ children: ref c, .. } = entry {
                children += c.len();
                true_pages += 1;
            } else if let &Entry::Redirect{ .. } = entry {
                redirects += 1;
            } else {
                blanks += 1;
            }
        }
        println!("=======================================");
        println!("Currently in the state: `{:?}`", self.state);
        println!("Number of entries: {}", self.entries.len());
        println!("Number of addresses: {}", self.addresses.len());
        println!(" Number of total true pages: {}", true_pages);
        println!(" Number of redirect pages:   {}", redirects);
        println!(" Number of blank page slots: {}", blanks);
        println!(" Number of children: {}", children);
        println!("=======================================");
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
