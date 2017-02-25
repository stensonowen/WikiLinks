extern crate slog;
extern crate regex;

use std::collections::{HashMap, HashSet};
use self::helpers::*;
mod helpers;

use super::super::super::Entry as PrimaryEntry;


// The actual data storing the internal link structure
pub struct Database {
    // when populating the entries/addresses fields, we may come across redirect pages
    // which means that multiple addresses (u32) can map to the same page
    //
    // Address  →  Page
    pub entries: HashMap<u32, Entry>,
    //  Title   →  Address
    pub addresses: HashMap<String, u32>,
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
                //warn!(self.log, "A pagelink gave a destination title not in the db: `{}`", dst_title);
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
    pub fn tidy_entries(&mut self) {
        //delete any redirects in page.sql that didn't show up in redirects.sql
        // they'll be in self.entries of type Entry::Redirect { target=None }
        
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
    pub fn print(&self) {
        let mut children = 0;
        let mut true_pages = 0;
        let mut redirects = 0;
        for (_,entry) in &self.entries {
            match entry {
                &Entry::Page{ children: ref c, .. } => {
                    children += c.len();
                    true_pages += 1;
                },
                &Entry::Redirect{..} => {
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
        assert!(self.state == State::AddLinks, 
                "Tried to finalize in the `{:?}` stage", self.state);
        info!(self.log, "Entering the `Done` stage");
        self.state = State::Done;
        //clean up links
        for (_,entry) in &mut self.entries {
            if let &mut Entry::Page{ 
                children: ref mut c, 
                parents: ref mut p, 
                title: ref mut t 
            } = entry {
                //clean up children/parents/title
                t.shrink_to_fit();
                c.dedup();
                c.shrink_to_fit();
                p.dedup();
                p.shrink_to_fit();
            }
        }
        info!(self.log, "Finalized db");
    }
    pub fn fattest(&self) {
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

    //fn export(self) -> HashMap<u32, PrimaryEntry> { remove any }
    
    /*
    // Codegen:
    // data crate should include `#![allow(dead_code)]
    fn codegen_links(&self, path: &Path) {
        let mut file = BufWriter::new(File::create(&path).unwrap());
        //copy all page links into static vars
        for (&addr,entry) in &self.entries {
            if let Some(s) = entry.codegen_links(addr) {
                file.write_all(s.as_bytes()).unwrap();
            }
        }
    }
    fn codegen_entries(&self, path: &Path) {
        let mut file = BufWriter::new(File::create(&path).unwrap());
        //copy the Entry type:
        /* struct Entry {
         *      title:      &'static str,
         *      children:   &'static [u32],
         *      parents:    &'static [u32],
		 *  } */
		write!(&mut file, "pub static ENTRIES: phf::Map<u32, Page> = ").unwrap();
        let mut builder = phf_codegen::Map::new();
		for (&addr,entry) in &self.entries {
            let mut i = 0;
            if let Some(s) = entry.codegen_page(addr) {
                builder.entry(addr,&s);
                i += 1;
                if i % 1_000_000 == 0 {
                    info!(self.log, "Entries Codegen Progress: {}  /  5 M", i);
                }
            }
        }
        info!(self.log, "Starting to generate ENTRIES phf table");
        builder.build(&mut file).unwrap();
        write!(&mut file, ";\n").unwrap();
    }
    fn codegen_addresses(&self, path: &Path) {
        let mut file = BufWriter::new(File::create(&path).unwrap());
        //copy the addresses type: map Strings to u32
		write!(&mut file, "pub static ADDRESSES: phf::Map<&'static str, u32> = ").unwrap();
        let mut builder = phf_codegen::Map::new();
        let mut i = 0;
        for (title,&addr) in &self.addresses {
            builder.entry(title.to_owned(), &addr.to_string());
            i += 1;
            if i % 1_000_000 == 0 {
                info!(self.log, "Entries Codegen Progress: {}  /  5 M", i);
            }
        }
        info!(self.log, "Starting to generate ADDRESS phf table");
        builder.build(&mut file).unwrap();
        write!(&mut file, ";\n").unwrap();
    }
    pub fn codegen(&self, dir: &Path) {
        //should be ../../wikidata/src or something
        
        //info!(self.log, "Starting LINKS codegen");
        info!(self.log, "Skipping LINKS codegen");
        //self.codegen_links(&dir.join(FILE_LINK_CG));
        //info!(self.log, "Starting ENTRIES codegen");
        info!(self.log, "Skipping ENTRIES codegen");
        //self.codegen_entries(&dir.join(FILE_ENTRIES_CG));
        info!(self.log, "Starting ADDRS codegen");
        self.codegen_addresses(&dir.join(FILE_ADDRS_CG));
        info!(self.log, "Finished codegen");
    }
    */
}
