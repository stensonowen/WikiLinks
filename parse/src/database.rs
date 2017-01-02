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

//what phase the database is in 
//TODO should I get rid of the numbers? They don't matter except that without 
// them it might not be clear that the order of the values is what determines
// their inequality; here concrete values make it clearer
#[derive(PartialEq, PartialOrd, Debug)]
enum State {
    Begin           = 0,
    AddPages        = 1,
    AddRedirects    = 2,
    TidyEntries     = 3,
    AddLinks        = 4,
    Done            = 5,
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
    pub fn add_page(&mut self, data: &regex::Captures) -> bool {
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
            return false;
        }

        let entry = if is_redr {
            Entry::Redirect {
                title:  String::from(title),
                target: None,
            }
        } else {
            Entry::Page {
                title:      String::from(title),
                children:   vec![],
                parents:    vec![],
            }
        };
        self.entries.insert(page_id, entry);
        self.addresses.insert(String::from(title), page_id);
        true
    }
    pub fn add_redirect(&mut self, data: &regex::Captures) -> bool {
        // must occur after all pages have been added and before any links
        assert!(self.state == State::AddPages || self.state == State::AddRedirects, 
                "Tried to add a redirect in the `{:?}` stage", self.state);
        if self.state == State::AddPages {
            info!(self.log, "Entering the `AddRedirects` stage");
            self.state = State::AddRedirects;
        }

        let redir_id: u32 = data.at(1).unwrap().parse().unwrap();
        let dst_title: &str = data.at(2).unwrap();

        //the redirect's target title *should* already be in self.addresses
        //if it is, its address should be of type Redirect(None), 
        // which we can update to Redirect(true_page_id)
        
        // entries[redir_id] should be Entries::Redirect{ target=None, .. }
        // the `target` should be changed to `Some(dst_id)`

        let entry = self.entries.get_mut(&redir_id);
        if let Some(&mut Entry::Redirect{ target: ref mut t, ..} ) = entry {
            if let &mut Some(x) = t {
                error!(self.log,
                       "The page_id of a redirect already redirected to page_id `{}`", x);
            }
            if let Some(&true_u32) = self.addresses.get(dst_title) {
                *t = Some(true_u32);
                return true;
            } else {
                warn!(self.log,
                      "The dst_title of a redirect was not in the db: `{}`", dst_title);
            }
        } else if entry.is_none() {
            //this can happen if the source has been deleted, or is in a different namespace
            //debug!(self.log, "A redirect source ID is missing from the table: {}", redir_id);
            //warn!(self.log, "A redirect source ID is missing from the table: {}", redir_id);
        } else {
            //should not insert in:
            // we know the redirect page_id but not its redirect_title
            // without a redirect_title, no links/redirects have any way of getting a handle
            error!(self.log, 
                   "The source page_id of a redirect was somehow a `{:?}`", entry);
        }
        false
    }
    pub fn add_pagelink(&mut self, data: &regex::Captures) -> bool {
        // only add pagelinks after adding redirects is finished
        assert!(self.state == State::AddRedirects || self.state == State::AddLinks,
                "Tried to add a link in the `{:?}` stage", self.state);
        if self.state == State::AddRedirects {
            self.tidy_entries();
            info!(self.log, "Entering the `AddLinks` stage");
            self.state = State::AddLinks;
        }

        let src_id: u32 = data.at(1).unwrap().parse().unwrap();
        let dst_title: &str = data.at(2).unwrap();

        //add dst_id to entries[src_id].children and src_id to entries[dst_id].parents

        //lookup dst_id from dst_title
        let dst_id = match self.addresses.get(dst_title) {
            Some(&true_id) => true_id,
            None => {
                //this can happen if the dst article isn't in the same namespace, or has
                // been removed so it wasn't listed in page.sql
                //warn!(self.log, 
                //      "A pagelink gave a destination title not in the db: `{}`", dst_title);
                return false;
            },
        };

        let src_id_r = self.follow_redirects(src_id).unwrap_or(src_id);
        let dst_id_r = self.follow_redirects(dst_id).unwrap_or(dst_id);
        /*
        // if entries[src_id] is a redirect, update src_id to what it should be
        // this can happens if the source article redirects to something else: 
        //  follow the link first
        let src_id_r = {
            if let Some(&Entry::Redirect{ target: t, .. }) = self.entries.get(&src_id) {
                if let Some(target) = t {
                    Some(target)
                } else {
                    //TODO: these cases shouldn't be a thing. the table should be tidied first
                    // then this can be an error
                    warn!(self.log,
                          "A pagelink gave a source id {} that redirected to None", src_id);
                    return false;
                }
            } else {
                None
            }
        };*/

        //update entries[src_id] to have dst_id among its children
        if let Some(&mut Entry::Page{ children: ref mut c, .. }) 
                = self.entries.get_mut(&src_id_r.unwrap_or(src_id)) {
            c.push(dst_id);                
        } else {
            //this can happen if:
            // a pagelink used an invalid source (it wasn't in page.sql because 
            //  the page was deleted or it's in the wrong namespace (nbd)
            // we followed a redirect above which pointed at an invalid page (error)
            // TODO: make this clearer? Use Some(thing) instead of mut src_id ?
            /*
            if let Some(x) = src_id_r {
                error!(self.log, "A pagelink source was a redirect that pointed to {}", x);
            } else {
                warn!(self.log, "A pagelink used an invalid source (not in page.sql)");
            }*/
            return false;
        }

        let tmp = { 
            let tmp = self.entries.get(&dst_id);
            if let Some(&Entry::Redirect{ title: ref ti, target: ta }) = tmp {
                Some(Entry::Redirect {
                    title: ti.clone(),
                    target: ta,
                })
            } else if let Some(&Entry::Page{ title: ref ti, .. }) = tmp {
                Some(Entry::new_page(ti))
            } else {
                None
            }
        };
        //update entries[dst_id] to have src_id among its parents
        if let Some(&mut Entry::Page{ parents: ref mut p, .. }) 
                = self.entries.get_mut(&dst_id) {
            p.push(src_id_r.unwrap_or(src_id));
        } else {
            //we got dst_id from addresses.get(dst), so it *should* be valid
            //the only explanation for an invalid dst_id is a logic bug
            //the REAL cause of this 'bug' is that pagelink destinations can be redirects :/
            error!(self.log, 
                   "The dst_id {} given by addresses[dst_title=`{}`] wasn't a Page but a {:?}",
                  dst_id, dst_title, tmp);
            if let Some(Entry::Redirect{ title: ref ti, target: ta }) = tmp {
                error!(self.log, "It was a redirect (`{}`)that pointed to entry {:?}", ti, ta);
            } else {
                panic!("This shouldn't happen: `{:?}`", tmp);
            }
            return false;
        }
        ////update entries[src_id] to have dst id among its children
        //if let Some(&mut Entry::Page{ children: ref mut p, ..})
        //        = self.entries.get_mut(&src_id) {
        //    c.push(dst_id
        true
    }
    fn follow_redirects(&self, start_id: u32) -> Result<u32,()> {
        // a pagelink can give a page_id that is a redirect as its source
        // it can also give a redirect's page title as its destination (TODO: fix in cleanup)
        // both of these are valid reasons to need to follow a page_id through Redirects
        //  until we find a valid page
        let mut cur_id = start_id;
        loop {
            let entry = self.entries.get(&cur_id);
            if let Some(&Entry::Redirect{ target: t, .. }) = entry {
                if let Some(target) = t {
                    cur_id = target;
                } else {
                    error!(self.log, 
                           "Couldn't follow redirects from {}: got None-target Redirect ({})",
                           start_id, cur_id);
                    return Err(());
                }
            //} else if let Some(e) = entry {
            } else if let Some(&Entry::Page{..}) = entry {
                //return e;
                return Ok(cur_id);
            } else {
                //this can happen if we have a chain of redirects that never terminates
                //mostly the case with links to articles in other namespaces
                //error!(self.log,
                //     "Couldn't follow redirects starting from {}: self.entries[{}] = None", 
                //     start_id, cur_id);
                return Err(());
            }
        }
    }
    fn post_pages(&self) {
        //just make sure everything is in order
        //verify every redirect points to a 

    }
    pub fn tidy_entries(&mut self) {
        //delete any redirects in page.sql that didn't show up in redirects.sql
        // they'll be in self.entries of type Entry::Redirect { target=None }
        //also:
        
        assert!(self.state == State::AddRedirects, 
                "tidy_entries wasn't called after AddRedirects (but `{:?}` instead)",
                self.state);
        self.state = State::TidyEntries;

        let mut collector: HashSet<u32> = HashSet::new();
        for (&addr,entry) in &self.entries {
            if let &Entry::Redirect { target: None, .. } = entry {
                collector.insert(addr);
            }
        }
        info!(self.log, "Removing {} unresolved redirects from self.entries", collector.len());
        for c in collector {
            self.entries.remove(&c);
        }
        
        //delete any addresses that point at None-type objects
        //pretty sure this can happen if we have a chain of redirects that resolves to None
        // 5/5 random samplings show this only happening with pages from other namespaces

        //go through the entries and make sure that addresses[redirect_title] points
        // to true_id, not redirect_id
        // we never need to retrieve the page_id of a redirect; it's just a cache miss
        
        let mut collector_update: HashMap<String,u32> = HashMap::new();
        let mut collector_remove: HashSet<String> = HashSet::new();
        for (title,&addr) in &self.addresses {
            //info!(self.log, "following redirs: addresses[`{}`] = {}", title, addr);
            match self.follow_redirects(addr) {
                Ok(a) if a == addr => {},
                Ok(x)   => {
                    collector_update.insert(title.to_owned(), x);
                },
                Err(()) => {
                    collector_remove.insert(title.to_owned());
                },
            }
            //let final_addr = self.follow_redirects(addr);
            //if final_addr.is_err() || final_addr.unwrap() != addr {
            //    collector.insert(title.to_owned(),final_addr.unwrap());
            //}
        }
        info!(self.log, 
              "Updating {} addresses to point to the DESTINATION of (a) redirect(s)", 
              collector_update.len());
        info!(self.log, "Removing {} addresses that point to nonexistant redirects", 
              collector_remove.len());
        for title in collector_remove {
            self.addresses.remove(&title);
        }
        for (title,addr) in collector_update {
            self.addresses.insert(title,addr);
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
        for (_,entry) in &self.entries {
            match entry {
                Entry::Page{ children: ref c, .. } => {
                    children += c.len();
                    true_pages += 1;
                },
                Entry::Redirect{..} => {
                    redirects += 1;
                },
            }
        }
        println!("=======================================");
        println!("Currently in the state: `{:?}`", self.state);
        println!("Number of entries: {}", self.entries.len());
        println!("Number of addresses: {}", self.addresses.len());
        println!(" Number of total true pages: {}", true_pages);
        println!(" Number of redirect pages:   {}", redirects);
        println!(" Number of blank page slots: {}", self.entries.len()-true_pages-redirects);
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
