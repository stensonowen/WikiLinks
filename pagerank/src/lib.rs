/*
 * Calculate the pagerank of the elements in wikidata::ENTRIES
 * Output the result to a .csv file, e.g.
 * https://gist.github.com/stensonowen/25df4124c1509a7033c5e1553c404a47
 *
 */

extern crate wikidata;
extern crate phf;
extern crate csv;

use wikidata::ENTRIES as ENTRIES;
use std::collections::HashMap;
use std::path::Path;

const DAMPING_FACTOR: f64 = 0.85;
const MAX_ERROR: f64 = std::f64::EPSILON * 10f64;
const MAX_ITER: usize = 500;   //iterations to panic! after (usually takes ~150)


pub fn wikidata_pageranks(output: &str) {
    //input: file to write results to (as csv)
    let mut g = Graph::new();
    g.compute_pageranks(true);  //be verbose
    g.export(output).unwrap();
}


struct Graph {
    pages:  &'static phf::Map<u32, wikidata::Page>,
    ranks:  HashMap<u32,f64>,
}

impl Graph {
    fn new() -> Graph {
        let size = ENTRIES.len();
        let mut pageranks = HashMap::with_capacity(size);
        let guess = (size as f64).recip();  // start each pagerank at 1/N
        for &entry in ENTRIES.keys() {
            pageranks.insert(entry,guess);
        }
        Graph {
            pages:  &ENTRIES,
            ranks:  pageranks,
        }
    }
    fn sum(&self) -> f64 {
        let mut sum = 0f64;
        for &v in self.ranks.values() {
            sum += v;
        }
        sum
    }
    fn iterate(&mut self) -> f64 {
        // Sub-optimal solution, but easiest to understand
        // It would be faster to just calculate the pagerank of each element one at a time
        //  (would mean fewer cache misses), but this is easier to understand so it's 
        //  easier to verify / debug
        //  This also makes it less convenient/efficient to find the max_change statistic
        // Iterate through pages and distribute pagerank as needed
        // Every page equally distributed its rank to all of its children
        //  Or, if it has no children, it equally distributes rank among all articles
        let starting_val = (1.0 - DAMPING_FACTOR) / (self.pages.len() as f64);

        let mut new_ranks: HashMap<u32,f64> = HashMap::with_capacity(self.ranks.capacity());
        //distribute pagerank
        for (addr,page) in self.pages.into_iter() {
            let pr = self.ranks.get(addr).unwrap();
            if page.children.len() == 0 {
                //equally distribute our pagerank to every page
                let n = self.pages.len() as f64;
                for &a in self.pages.keys() {
                    let mut x = new_ranks.entry(a).or_insert(starting_val);
                    *x += DAMPING_FACTOR * (pr / n);
                }
            } else {
                //equally distribute our pagerank to all our children
                let n = page.children.len() as f64;
                for &a in page.children {
                    let mut x = new_ranks.entry(a).or_insert(starting_val);
                    *x += DAMPING_FACTOR * (pr / n);
                }
            }
        }
        //identify the greatest change that is being made to self.ranks
        let mut max_change = 0f64;
        for (addr,rank) in &self.ranks {
            let delta = (rank - new_ranks.get(addr).unwrap()).abs();
            max_change = max_change.max(delta);
        }
        self.ranks = new_ranks;
        max_change
    }
    fn compute_pageranks(&mut self, verbose: bool) -> usize {
        //run self.iterate() until the max difference is within MAX_ERROR
        //supposedly should run ~100 - ~150 times
        //takes 148 iterations and ~8 minutes to solve simple wiki
        let mut diff = std::f64::MAX;
        let mut iter = 0;
        let every = 10; //how often to print debug output
        if verbose {
            println!(" i\t\tMax Diff\t\t\tSum");
        }
        while diff > MAX_ERROR {
            //terminate early if (sum - 1.0).abs() > diff?
            // that would indicate that floating point errors are getting meaningful
            //currently takes 148 iterations for the Simple wiki (~80 seconds)
            diff = self.iterate();
            iter += 1;
            if verbose && iter % every == 0 {
                //println!("{:03}:    {:1.20}, \t{:1.20}", iter, diff, self.sum());
                println!("{:03}:    {},\t\t{}", iter, diff, self.sum());
            } 
            if iter > MAX_ITER {
                panic!("Iter got too high");
            }
        }
        if verbose {
            println!("Finished calculating pagerank after {} iterations", iter);
            println!("The maximum change in rank since the last iteration was {}", diff);
            println!("The final total sum is {}", self.sum());
        }
        iter
    }
    fn export(&self, filepath: &str) -> csv::Result<()> {
        //output into a .csv
        // page_id, pagerank, page_title
        let path = Path::new(filepath);
        let mut writer = csv::Writer::from_file(&path)?;
        for (id,rank) in &self.ranks {
            let title = self.pages.get(id).unwrap().title;
            writer.encode((id, rank, title))?;
        }
        Ok(())
    }
}



