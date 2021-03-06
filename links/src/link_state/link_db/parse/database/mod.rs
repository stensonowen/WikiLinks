extern crate regex;

use slog;
mod helpers;
use self::helpers::*;
use super::super::IndexedEntry;

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::iter;


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
    // keep track of all valid titles that map to valid page_ids
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
    pub fn log<T: Display>(&self, text: T) {
        info!(self.log, text);
    }
    pub fn num_entries(&self) -> usize {
        self.entries.len()
    }
    pub fn title_table(&self) -> HashMap<String, u32> {
        // user can look up multiple variants of a title and get back its page_id
        //
        // for now this is mandatory, but it shouldn't be all that expensive
        //
        // todo: remove metadata thing; instead replace rankdata w/ analyzedata
        // all analysis is run by cmd-line args and done on this phase
        // if no analysis is to be done, then this stage is empty (but still run)
        // there should only be one dump file
        
        // list of all titles originally
        let mut starters = HashMap::with_capacity(self.entries.len());
        // list of all capitalized versions of titles
        let mut capitals = HashMap::new();
        // terms that two non-caps titles both capitalize to
        let mut cap_cols = HashSet::new();  // capital collisions; delete
        for (&id, entry) in &self.entries {
            let title = match *entry {
                Entry::Page{ title: ref ti, .. } => ti,
                Entry::Redirect{ title: ref ti, target: Some(x) } if id==x => ti,
                _ => continue,
            };
            starters.insert(title.clone(), id);
        }
        for (title, &id) in &self.addresses {
            // redirect isn't present, might as well add it
            starters.entry(title.clone()).or_insert(id);
        }
        for (title,&id) in &starters {
            // add to the 'capitals' bin if it wasn't there and the caps version 
            let ti_caps = title.to_uppercase();
            if starters.contains_key(&ti_caps) {
                continue;
            } 
            if let Some(&id2) = capitals.get(&ti_caps) {
                if id != id2 {
                    // 2 different articles capitalized to the same thing
                    cap_cols.insert(ti_caps);
                }
            } else {
                // haven't seen this capitalization before
                capitals.insert(ti_caps, id);
            }
        }
        let mut titles = starters;
        for (ti,id) in capitals {
            if cap_cols.contains(&ti) == false {
                titles.insert(ti, id);
            }
        }
        titles
    }
    //pub fn explode(self) -> 
    //    (Box<iter::Iterator<Item=IndexedEntry>>,
    //     Box<iter::Iterator<Item=(String,u32)>>)
    pub fn explode(self) -> Box<iter::Iterator<Item=IndexedEntry>> {
        // destroy ourSelf and yield our contents
        // they will go in totally different data structures so yield iterators
        let entry_iter = self.entries.into_iter().map(|e: (u32,Entry)| {
            match e.1 {
                Entry::Redirect{..} => panic!("Found redirect during explosion"),
                Entry::Page{ title: ti, parents: p, children: c } => 
                    IndexedEntry::from(e.0, ti, p, c)
            }});
        Box::new(entry_iter)
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
            if let Some(x) = *t {
                error!(self.log,
                       "The page_id of a redirect already redirected to page_id `{}`", x);
            }
            if let Some(&true_u32) = self.addresses.get(dst_title) {
                *t = Some(true_u32);
                return true;
            } else {
                //warn!(self.log,
                //      "The dst_title of a redirect was not in the db: `{}`", dst_title);
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
        //add dst_id to entries[src_id].children and src_id to entries[dst_id].parents
        
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

        //lookup dst_id from dst_title
        let dst_id = match self.addresses.get(dst_title) {
            Some(&true_id) => true_id,
            None => {
                //this can happen if the dst article isn't in the same namespace, or has
                // been removed so it wasn't listed in page.sql
                //warn!(self.log, 
                //      "A pagelink gave a destination title not in the db: `{}`", 
                //      dst_title);
                return false;
            },
        };

        let src_id_r = self.follow_redirects(src_id).unwrap_or(src_id);
        let dst_id_r = self.follow_redirects(dst_id).unwrap_or(dst_id);

        //update entries[src_id] to have dst_id among its children
        if let Some(&mut Entry::Page{ children: ref mut c, .. }) 
                = self.entries.get_mut(&src_id_r) {
            c.push(dst_id_r);                
        } else {
            //this can happen if:
            // * a pagelink used an invalid source (it wasn't in page.sql because 
            //    the page was deleted or it's in the wrong namespace (nbd))
            // * we followed a redirect above which pointed at an invalid page (error)
            return false;
        }

        //update entries[dst_id] to have src_id among its parents
        if let Some(&mut Entry::Page{ parents: ref mut p, .. }) 
                = self.entries.get_mut(&dst_id_r) {
            p.push(src_id_r);
        } else {
            //pagelink destinations can be redirects, so this is a potential code path
            //it is dealt with in self.tidy_entries to make sure the table isn't bloated
            error!(self.log, 
                   "The dst_id_r {} (dst_id={}) given by addresses[dst=`{}`] wasn't a Page",
                  dst_id_r, dst_id, dst_title);
            return false;
        }
        true
    }
    fn follow_redirects(&self, start_id: u32) -> Result<u32,()> {
        // a pagelink can give a page_id that is a redirect as its source
        // it can also give a redirect's page title as its destination 
        // so we might need to follow a page_id through Redirects until we find a valid page
        let mut cur_id = start_id;
        let mut seen: HashSet<u32> = HashSet::new();    //avoid repetitions
        loop {
            let entry = self.entries.get(&cur_id);
            if let Some(&Entry::Redirect{ target: t, .. }) = entry {
                if let Some(target) = t {
                    if seen.contains(&target) {
                        //avoid loops
                        return Err(());
                    } else {
                        seen.insert(target);
                        cur_id = target;
                    }
                } else {
                    error!(self.log, 
                           "Couldn't follow redirects from {}: got None-target Redirect ({})",
                           start_id, cur_id);
                    return Err(());
                }
            } else if let Some(&Entry::Page{..}) = entry {
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
    fn tidy_entries(&mut self) {
        //delete any redirects in page.sql that didn't show up in redirects.sql
        // they'll be in self.entries of type Entry::Redirect { target=None }
        
        assert_eq!(self.state, State::AddRedirects, 
                "tidy_entries wasn't called after AddRedirects (but `{:?}` instead)",
                self.state);
        self.state = State::TidyEntries;

        let mut collector: HashSet<u32> = HashSet::new();
        for (&addr,entry) in &self.entries {
            if let Entry::Redirect { target: None, .. } = *entry {
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
                Ok(x)   => { collector_update.insert(title.to_owned(), x); },
                Err(()) => { collector_remove.insert(title.to_owned()); },
            }
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
    fn _print(&self) {
        let mut children = 0;
        let mut true_pages = 0;
        let mut redirects = 0;
        for entry in self.entries.values() {
            match *entry {
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
        println!(" Number of blank page slots: {}",self.entries.capacity()-self.entries.len());
        println!(" Number of children: {}", children);
        println!("=======================================");
    }
    pub fn finalize(&mut self) {
        //modify the `State` so that no further modifications can be made
        assert_eq!(self.state, State::AddLinks, 
                "Tried to finalize in the `{:?}` stage", self.state);
        info!(self.log, "Entering the `Done` stage");
        self.state = State::Done;
        //clean up links
        for entry in self.entries.values_mut() {
            if let Entry::Page { 
                children: ref mut c, 
                parents: ref mut p, 
                title: ref mut t 
            } = *entry {
                //clean up children/parents/title
                //need to sort before dedup 
                t.shrink_to_fit();
                c.sort();
                c.dedup();
                c.shrink_to_fit();
                p.sort();
                p.dedup();
                p.shrink_to_fit();
            }
        }
        // delete redirects in page.sql that didn't show up in redirects.sql
        //debug!(self.log, "Delete unconfirmed redirects...");
        //self.tidy_entries();

        // get rid of the redirects in self.entries 
        info!(self.log, "\tPurge Entries of redirects...");
        self.remove_redirects();

        // make sure parent/child links go both ways
        info!(self.log, "\tAssert symmetric child/parent relationships...");
        self.validate();    // AFTER HERE

        // make sure links from addresses to entries go both ways
        // delete those that don't
        /*
        let asymmetric_references = self.asymmetric_references();
        println!("Done collecting asymmetric refs");
        info!(self.log, 
               "\tDelete {} unrequited addresseses (not echoed in Entries)...",
               asymmetric_references.len());
        self.pop_addresses(asymmetric_references);
               */
        let empty_refs = self.incomplete_addresses();
        println!("Done collecting incomplete refs");
        info!(self.log, "\tDelete {} incomplete addrs (absent in Entries)...",
               empty_refs.len());
        self.pop_addresses(empty_refs);

        // BEFORE HERE
        //make sure no children or parents links contain redirects
        info!(self.log, "\tAssert no children or parents can be redirects...");
        self.find_redirs_in_links();

        //self._children_vs_parents();
        //self.verify_children_present();

        info!(self.log, "Finalized db");
    }
    /*
    fn fattest(&self) {
        //find the most popular entry
        let mut max_addr = 0;
        let mut max_val  = 0;
        for (&addr,entry) in &self.entries {
            if let &Entry::Page{ children: ref c, ..} = entry {
                if c.len() > max_val {
                    max_addr = addr;
                    max_val = c.len();
                }
            }
        }
        if let Some(&Entry::Page{ title: ref t, .. }) = self.entries.get(&max_addr) {
            println!("The address of the entry w/ the most children is {}: `{}` with {} kids", 
                 max_addr, t, max_val);
        } else {
            panic!("bad");
        }
        if let Some(&Entry::Page{ children: ref c, parents: ref p, .. }) 
                = self.entries.get(&max_addr) {
            println!("CHILDREN:");
            for (i,j) in c.iter().enumerate() {
                if let Some(&Entry::Page{ title: ref t, .. }) = self.entries.get(j) {
                    println!("\t{}:\t{}:\t`{}`", i, *j, t);
                } else {
                    println!("\t{}:\t{}:\t`{}`", i, *j, "CHILD NOT FOUND");
                }
            }
            println!("PARENTS:");
            for (i,j) in p.iter().enumerate() {
                if let Some(&Entry::Page{ title: ref t, .. }) = self.entries.get(j) {
                    println!("\t{}:\t{}:\t`{}`", i, *j, t);
                } else {
                    println!("\t{}:\t{}:\t`{}`", i, *j, "PARENT NOT FOUND");
                }
            }
        } else {
            println!("ERror");
        }
        println!();
    }
    */

    fn validate(&self) {
        //panic if there are parent/child inconsistencies
        //verify all parent → child relationships are commutative
        //verify there are no Redirects in child/parent types
        let mut all = 0;
        let mut redirects = 0;
        let mut pages = 0;
        let mut children = 0;
        let mut parents = 0;
        info!(self.log, "Validating...");
        for(id,entry) in &self.entries {
            all += 1;
            match *entry {
                Entry::Page { title: ref t, children: ref c, parents: ref p } => {
                    pages += 1;
                    for child in c {
                        children += 1;
                        let target = self.entries.get(child);
                        assert!(target.is_some(), 
                                "Page `{}`'s child {} doesn't exist", t, child);
                        match *target.unwrap() {
                            Entry::Page { title: ref t_, parents: ref p_, .. } => {
                                assert!(p_.contains(id), 
                                        "Page `{}` has child {}, but child {} lacks parent {}",
                                        t, child, t_, id);
                            },
                            Entry::Redirect { title: ref t_, target: ref x_ } => {
                                panic!("Page `{}` pointed to child {}({}), a redir to {:?}",
                                       t, child, t_, x_);
                            }
                        }
                    }
                    for parent in p {
                        parents += 1;
                        let target = self.entries.get(parent);
                        assert!(target.is_some(), 
                                "Page `{}`'s parent {} doesn't exist", t, parent);
                        match *target.unwrap() {
                            Entry::Page { title: ref t_, children: ref c_, .. } => {
                                assert!(c_.contains(id), 
                                    "Page `{}` has parent {}, but parent {} lacks child {}",
                                        t, parent, t_, id);
                            },
                            Entry::Redirect { title: ref t_, target: ref x_ } => {
                                panic!("Page `{}` pointed to parent {}({}), a redir to {:?}",
                                       t, parent, t_, x_);
                            }
                        }
                    }
                },
                Entry::Redirect { title: ref _t, target: ref _x } => {
                    redirects += 1;
                }
            }
        }
        //println!("All good");
        info!(self.log, "Validated :)");
        info!(self.log, "All: \t\t{}", all);
        info!(self.log, "Redirects: \t\t{}", redirects);
        info!(self.log, "pages: \t\t{}", pages);
        info!(self.log, "children: \t\t{}", children);
        info!(self.log, "parents: \t\t{}", parents);
    }

    fn remove_redirects(&mut self) {
        //iterate through self.entries and remove redirects
        //set self.addresses[title] to redirect dst, not src
 
        //create collection of redirects
        let mut redirects: Vec<u32> = vec![];
        for (&id,entry) in &self.entries {
            if let Entry::Redirect { title: ref ti, target: ta } = *entry {
                //change address of title to the destination
                self.addresses.insert(ti.to_owned(), ta.unwrap());
                redirects.push(id);
            }
        }
        info!(self.log, "Removing {} redirects from entries", redirects.len());

        //get those redirect entries out
        for r in redirects {
            self.entries.remove(&r);
        }

        //TODO: remove redirects from all parent and children lists
        //  (gonna be expensive)
    }

    fn find_redirs_in_links(&self) {
        info!(self.log, "Looking for redirect family...");
        let mut redirects: HashSet<u32> = HashSet::new();
        for (&id, entry) in &self.entries {
            if let Entry::Redirect{ .. } = *entry {
                redirects.insert(id);
            }
        }
        info!(self.log, "Found {} redirects...", redirects.len());
        let mut fails = 0usize;
        let mut checked = 0usize;
        for (&id, entry) in &self.entries {
            if let Entry::Page{ title: ref ti, children: ref c, parents: ref p} = *entry {
                for child in c {
                    checked += 1;
                    if redirects.contains(child) {
                        error!(self.log, 
                               "Found a child (of {}: {}) w/ a redirect child ({})",
                               id, ti, child);
                        fails += 1;
                        if fails > 10 {
                            return;
                        }
                    }

                }
                for parent in p {
                    checked += 1;
                    if redirects.contains(parent) {
                        error!(self.log, 
                               "Found a parent (of {}: {}) w/ a redirect parent ({})",
                               id, ti, parent);
                        fails += 1;
                        if fails > 10 {
                            return;
                        }
                    }
                }
            }
        }
        info!(self.log, "NO REDIRECTS IN  {}  CHILDREN OR PARENTS :)", checked);

    }

    fn pop_addresses(&mut self, chopping_block: Vec<String>) {
        //mut/immut workaround :/
        for title in chopping_block {
            self.addresses.remove(&title);
        }
    }
    fn _asymmetric_references(&self) -> Vec<String> {
        /*
         * Welp. There are some redirects that have the redirect flag in page.sql
         *  but are not in redirects.sql. I guess it's not too bad to just purge them
         *
         */
        let mut chopping_block: Vec<String> = vec![];
        //make sure everything in addresses points to a real element
        for (id, entry) in &self.entries {
            //make sure every entry is a Page, not a redirect
            //make sure looking it up by its title gives the right id
            let title = match *entry {
                Entry::Page{ title: ref t, .. } => t,
                _ => panic!("Found a redirect in verify_links step"),
                //_ => continue
            };
            let other_id = match self.addresses.get(title) {
                Some(id) => id,
                None => {
                    //error!(self.log, 
                    //       "Found an entry ({}) whose title wasn't in addresses",
                    //       title);
                    //continue;
                    panic!("Found an entry ({}) whose title wasn't in addresses", 
                           title);
                }
            };

            //shouldn't be any of these
            assert_eq!(id, other_id, "entries[{}]->title = {}; addrs[{}] != {}", 
                    id, title, title, other_id);
        }
        for (title, id) in &self.addresses {
            let corresponding_entry = match self.entries.get(id) {
                Some(id_) => id_,
                None => {
                    // TODO: turn this back into a log entry
                    //  (a slog macro causes an irritating warning :/)
                    //error!(self.log, 
                    println!(
                     "Found an addr whose id ({}) wasn't in entries; deleting it.",
                         id);
                    chopping_block.push(title.to_owned());
                    continue;
                }
            };
            //let (other_ti, _other_c) = match self.entries.get(id).unwrap() {
            let (other_ti, _other_c) = match *corresponding_entry {
                Entry::Redirect{..} => panic!("Addr {} → redirect {}", title, id),
                //&Entry::Redirect{..} => continue,
                Entry::Page{ title: ref ti, children: ref c, .. } => (ti, c),
            };
            //assert_eq!(title, other_ti, "addrs[{}] = {} but entries[{}] = {}",
            //           title, id, id, other_ti);
            if title != other_ti {
                chopping_block.push(title.to_owned());
            }
        }
        chopping_block
    }

    fn incomplete_addresses(&self) -> Vec<String> {
        // a list of all `addresses` that point to nothing
        self.addresses.iter()
            .filter_map(|(s,i)| match self.entries.get(i) {
                Some(_) => None,
                None => Some(s.clone())
            })
            .collect() 
    }
}
