extern crate wikidata;
extern crate phf;

use wikidata::ENTRIES as ENTRIES;
//use wikidata::ADDRESSES as ADDRESSES;
//use wikidata::Page;

use std::collections::HashMap;

const DAMPING_FACTOR: f64 = 0.85;

/*
struct PageRank {
    score: f64,
    page: &'static wikidata::Page,
}

impl PageRank {
    fn len_p(&self) -> usize {
        self.page.parents.len()
    }
    fn len_c(&self) -> usize {
        self.page.children.len()
    }
}
*/


/*
struct Web2 {
    items: HashMap<u32,PageRank>,
    df: f64,
}

impl Web2 {
    fn new(guess: f64) -> Web2 {
        let mut pages = HashMap::new();
        for (&addr,page) in ENTRIES.into_iter() {
            pages.insert(addr, PageRank {
                score: guess,
                page: page
            });
        }
        Web2 {
            items: pages,
            df: DAMPING_FACTOR,
        }
    }
}
*/


pub struct Web {
    pages:  &'static phf::Map<u32, wikidata::Page>,
    ranks:  HashMap<u32,f64>,
    iter:   usize,
}

impl Web {
    pub fn new() -> Web {
        let size = ENTRIES.len();
        let mut pageranks = HashMap::with_capacity(size);
        let guess = (size as f64).recip();  // start each pagerank at 1/N
        for &entry in ENTRIES.keys() {
            pageranks.insert(entry,guess);
        }
        Web {
            pages:  &ENTRIES,
            ranks:  pageranks,
            iter:   0,
        }
    }
    pub fn sum(&self) -> f64 {
        let mut sum = 0f64;
        for addr in self.pages.keys() {
            sum += *self.ranks.get(addr).unwrap()
        }
        sum
    }
    pub fn iterate(&mut self) -> f64 {
        //all new pageranks will be calculated from the duplicated values (the old table)
        //return the maximum change in an article's pagerank
        let mut new_ranks: HashMap<u32,f64> = HashMap::with_capacity(self.ranks.capacity());
        let base_pr = (1f64 - DAMPING_FACTOR) / (self.pages.len() as f64);
        let mut max_change = 0f64;
        let mut sink_pr = 0f64;
        for (addr,page) in self.pages.into_iter() {
            if page.children.len() == 0 {
                sink_pr += *self.ranks.get(addr).unwrap()
            }
        }
        for (&addr,page) in self.pages.into_iter() {
            //track sum of `page`'s parents's pageranks divided by their number of children
            let mut extra_pr = 0f64;
            for p in page.parents {
                let parent_pr = self.ranks.get(p).unwrap();
                let parent_num_kids = self.pages.get(p).unwrap().children.len() as f64;
                //parent_num_kids should always be ≥1: our parent has at least 1 kid (us)
                extra_pr += parent_pr / parent_num_kids;
            }
            //calculate our pagerank: (1-d)/N + d*Σ(PR(p)/L(p))
            //let pr = base_pr + DAMPING_FACTOR * extra_pr;
            //sink_pr = DAMPING_FACTOR * sink_pr / (self.pages.len() as f64); 
            sink_pr = sink_pr / (self.pages.len() as f64); 
            let pr = base_pr + DAMPING_FACTOR * (extra_pr + sink_pr);
            //keep track of the magnitude of the changes made
            let change = (self.ranks.get(&addr).unwrap() - pr).abs();
            if change > max_change {
                max_change = change;
            }
            //update the new database
            new_ranks.insert(addr,pr);
        }
        self.ranks = new_ranks;
        self.iter += 1;
        max_change
    }
    pub fn iterate2(&mut self) -> f64 {
        //returns the max difference
        let mut new_ranks: HashMap<u32,f64> = HashMap::new();
        //each page transfers its pagerank equally among its children
        //unless it has no children, then it transfers equally among all pages
        let mut max_change = 0.;
        
        for (addr,page) in self.pages.into_iter() {
            let mut pr = 0.;
            for p in page.parents {
                let parent_rank = self.ranks.get(p).unwrap();
                let num_siblings = self.pages.get(p).unwrap().children.len() as f64;
                pr += parent_rank / num_siblings;
            }
            pr = 1. - DAMPING_FACTOR + DAMPING_FACTOR * pr;
            new_ranks.insert(*addr,pr);
            let delta = (pr - self.ranks.get(addr).unwrap()).abs();
            if delta > max_change {
                max_change = delta;
            }
        }
        self.ranks = new_ranks;
        max_change
    }
    pub fn iterate3(&mut self) -> f64 {
        /* wtf
         *  This simplified method is slower but seems more correct (?!)
         *  We ignore the damping factor and square our cache misses (so it's defs slower), 
         *   but our total sum osciallates around 1.000... and our max diff converges
         *  But we totally ignore the concept of a damping factor so this can't be right
         *  There should be a constant starting value for pageranks (instead of `0.`) and
         *   the influence a page has should decrease by 15% every iteration (right???)
         *  Why does this behave like iterate1 and iterate2 should?
         */
        let mut new_ranks: HashMap<u32,f64> = HashMap::with_capacity(self.ranks.capacity());
        //each page transfers its pagerank equally among its children
        //unless it has no children, then it transfers equally among all pages
        for (addr,page) in self.pages.into_iter() {
            let pr = self.ranks.get(addr).unwrap();
            if page.children.len() == 0 {
                //equally distribute our pagerank to every page
                let n = self.pages.len() as f64;
                for &a in self.pages.keys() {
                    let mut x = new_ranks.entry(a).or_insert(0.);
                    *x += pr / n;
                }
            } else {
                //equally distribute our pagerank to all our children
                let n = page.children.len() as f64;
                for &a in page.children {
                    let mut x = new_ranks.entry(a).or_insert(0.);
                    *x += pr / n;
                }
            }
        }
        let mut max_change = 0f64;
        for (addr,rank) in &self.ranks {
            let delta = (rank - new_ranks.get(addr).unwrap()).abs();
            if delta > max_change {
                max_change = delta;
            }
        }
        self.ranks = new_ranks;
        max_change
    }
}
