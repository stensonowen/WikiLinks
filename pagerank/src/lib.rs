extern crate wikidata;
extern crate phf;
extern crate ramp;

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

use ramp::int::Int;
use ramp::rational::Rational;

pub struct Web {
    pages:  &'static phf::Map<u32, wikidata::Page>,
    //ranks:  HashMap<u32,Rational>,
    ranks:  HashMap<u32,f64>,
    iter:   usize,
}

impl Web {
    pub fn new() -> Web {
        let size = ENTRIES.len();
        let mut pageranks = HashMap::with_capacity(size);
        //let guess = (size as f64).recip();  // start each pagerank at 1/N
        let guess = Rational::new(Int::one(), Int::from(size));
        for &entry in ENTRIES.keys() {
            pageranks.insert(entry,guess.clone());
        }
        Web {
            pages:  &ENTRIES,
            ranks:  pageranks,
            iter:   0,
        }
    }
    //pub fn sum(&self) -> f64 {
    pub fn sum(&self) -> Rational {
        //let mut sum = 0f64;
        let mut sum = Rational::new(Int::zero(), Int::one());
        for addr in self.pages.keys() {
            sum += self.ranks.get(addr).unwrap()
        }
        sum
    }
    //pub fn iterate(&mut self) -> f64 {
    pub fn iterate(&mut self) -> Rational {
        //all new pageranks will be calculated from the duplicated values (the old table)
        //return the maximum change in an article's pagerank
        //let mut new_ranks: HashMap<u32,Int> = HashMap::with_capacity(self.ranks.capacity());
        let mut new_ranks: HashMap<u32,Rational> = HashMap::with_capacity(self.ranks.capacity());
        //let base_pr = (1f64 - DAMPING_FACTOR) / (self.pages.len() as f64);
        //let base_pr = (1f64 - DAMPING_FACTOR) / (self.pages.len() as f64);
        // (1-.85)/N = (3/20)/N = 3/(20*N)
        let base_pr = Rational::new(Int::from(3),Int::from(20*self.pages.len()));
        //let mut max_change = 0f64;
        let mut max_change = Rational::new(Int::zero(), Int::one());
        //let mut sink_pr = 0f64;
        let mut sink_pr = Rational::new(Int::zero(), Int::one());
        for (addr,page) in self.pages.into_iter() {
            if page.children.len() == 0 {
                sink_pr += self.ranks.get(addr).unwrap()
            }
        }
        for (&addr,page) in self.pages.into_iter() {
            //track sum of `page`'s parents's pageranks divided by their number of children
            //let mut extra_pr = 0f64;
            let mut extra_pr = Rational::new(Int::zero(), Int::one());
            for p in page.parents {
                let parent_pr = self.ranks.get(p).unwrap();
                //let parent_num_kids = self.pages.get(p).unwrap().children.len() as f64;
                let parent_num_kids = Int::from(self.pages.get(p).unwrap().children.len());
                //parent_num_kids should always be ≥1: our parent has at least 1 kid (us)
                extra_pr += parent_pr / parent_num_kids;
            }
            //calculate our pagerank: (1-d)/N + d*Σ(PR(p)/L(p))
            //let pr = base_pr + DAMPING_FACTOR * extra_pr;
            //sink_pr = DAMPING_FACTOR * sink_pr / (self.pages.len() as f64); 
            //sink_pr = sink_pr / (self.pages.len() as f64); 
            sink_pr = sink_pr.clone() / Int::from(self.pages.len());
            let pr = base_pr.clone() 
                + Rational::new(Int::from(17),Int::from(20)) * 
                (extra_pr.clone() + sink_pr.clone());
            //keep track of the magnitude of the changes made
            let change = (self.ranks.get(&addr).unwrap() - pr.clone()).abs();
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

    }
}
