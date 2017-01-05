extern crate wikidata;
extern crate phf;

use std::collections::{HashSet, HashMap};
use wikidata::ENTRIES;
use std::mem::swap;

const MAX_DEPTH: usize = 5;
//const START_SIZE: usize = 100;  

pub fn bfs(src: u32, dst: u32) -> Result<Vec<u32>,usize> {
    //perform breadth-first-search using wikidata::ENTRIES from src to dst
    //return vector of page_id's (i.e. the path)
    //should this be multithreaded??
    //TODO: investigate using a Bloom Filter for checking intersection quickly
    //  Might be faster: that is its thing
    //  Might be negligible: uses hashing and a lookup table, which is basically a hashmap
    //TODO: play with different approximate hints 
    //  i.e. starting/changing hashmap capacity via guessing
    //TODO: fix code duplication (top-down and bottom-up doing almost the same thing
    //TODO: use fewer than SIX hashmaps?
    //TODO: there's a fair bit of redundancy: top_down_n ⊆ src_seen ⊇ top_down_o
    
    //`seen` pages: the id we've encountered and the id that linked to it (parent OR child)
    let mut src_seen: HashMap<u32,u32> = HashMap::new();
    let mut dst_seen: HashMap<u32,u32> = HashMap::new();

    //keep track of articles we're checking in each direction
    //each direction should have an immutable bank and a mutable collector (that swap)
    //start from src, follow children
    let mut top_down_n:  HashSet<u32> = HashSet::new();
    let mut top_down_o:  HashSet<u32> = HashSet::new();
    top_down_o.insert(src);
    //start from dst, follow parents
    let mut bottom_up_n: HashSet<u32> = HashSet::new();
    let mut bottom_up_o: HashSet<u32> = HashSet::new();
    bottom_up_o.insert(dst);

    for _ in 0 .. MAX_DEPTH {
        //top-down first? Any benefit to doing both at the same time?
        for &p in &top_down_o {
            for &c in ENTRIES.get(&p).unwrap().children {
                //for each child, if unseen, mark it as interest
                if *src_seen.entry(c).or_insert(p) == p {
                    //a little wonky but I think slightly more efficient
                    // if `c` hasn't been seen, insert `seen[c]=p` and skip ahead 
                    // otherwise, `c` has been inserted into `seen`
                    //if `c` was unseen, check if it has been found
                    if dst_seen.contains_key(&c) {
                        return Ok(extract_path(src, dst, &src_seen, &dst_seen, c));
                    }
                    top_down_n.insert(c);
                }
            }
        }
        top_down_o.clear();
        swap(&mut top_down_o, &mut top_down_n);

        //now do the same thing from the other direction
        for &p in &bottom_up_o {
            for &c in ENTRIES.get(&p).unwrap().parents {
                if *dst_seen.entry(c).or_insert(p) == p {
                    if src_seen.contains_key(&p) {
                        return Ok(extract_path(src, dst, &src_seen, &dst_seen, c));
                    }
                    bottom_up_n.insert(c);
                }
            }
        }
        bottom_up_o.clear();
        swap(&mut bottom_up_o, &mut bottom_up_n);
    }

    Err(MAX_DEPTH)
}

fn extract_path(src: u32, dst: u32,
                src_seen: &HashMap<u32,u32>, 
                dst_seen: &HashMap<u32,u32>, 
                common: u32) -> Vec<u32> {
    //given entries findable from the src, those from the dst, and an intersecting entry,
    // find the path from src to dst
    //For now, the path should include the src and dst ids
    let mut path = vec![common];
    //first find the path from the midpoint to the src (will be backwards)
    let mut current = common;
    while current != src {
        //could just iterate until it finds something without a parent? no passing src/dst ?
        //this is easier to debug for now
        current = *src_seen.get(&current).unwrap();
        path.push(current);
    }
    path.reverse();
    current = common;
    while current != dst {
        current = *dst_seen.get(&current).unwrap();
        path.push(current);
    }
    path
}

pub fn print_path(res: Result<Vec<u32>,usize>) {
    if let Ok(path) = res {
        println!("Found a path with {} steps: ", path.len()-1);
        for id in &path {
            let title = ENTRIES.get(id).unwrap().title;
            println!("\t{: >12}: \t`{}`", id, title);
        }
    } else if let Err(x) = res {
        println!("Failed to find a path after {} iterations", x);
    }
}
