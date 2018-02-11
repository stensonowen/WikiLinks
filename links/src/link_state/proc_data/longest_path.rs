// Compute the set of articles that take the longest to get to the dst
// For example, the maximum distance from any article to the 'USA' page 
//  is probably pretty small

use std::collections::HashSet;
use std::mem::swap;
use link_state::ProcData;

impl ProcData {
    pub fn longest_path(&self, dst: u32) -> u8 {
        let cap = self.links.len();
        let mut all_seen: HashSet<u32> = HashSet::with_capacity(cap);
        let mut last_row: HashSet<u32> = HashSet::with_capacity(cap);
        let mut tmp:      HashSet<u32> = HashSet::with_capacity(cap);
        let mut height: u8 = 0;     // beware off-by-ones

        last_row.insert(dst);
        all_seen.insert(dst);

        println!("Destination article: `{}`", self.links[&dst].title);
        
        loop {
            /*
            println!();
            println!("There are {} articles that can reach {} in {} steps", 
                     last_row.len(), title, height);
            println!("Articles already seen: {}", all_seen.len());
            println!("Total articles reachable: {}", all_seen.len() + last_row.len());
            println!("Total articles: {}", self.links.len());
            let preview: Vec<_> = last_row
                .iter().take(20)
                .map(|i| &self.links.get(i).unwrap().title)
                .collect();
            println!("Articles in the last row: {:?}", preview);
            println!();
            */
            height += 1;
            for last in &last_row {
                // put all
                for next in self.links[last].get_parents() {
                    if all_seen.contains(next) == false {
                        tmp.insert(*next);
                        all_seen.insert(*next);
                    }
                }
            }
            if tmp.is_empty() && last_row.len() < 20 {
                println!("Last row: {:?}", last_row);
            }
            last_row.clear();
            swap(&mut tmp, &mut last_row);
            if last_row.is_empty() {
                println!("All {} ancestors can reach in a maximum of {} steps",
                         all_seen.len(), height);
                //if tmp.len() < 20 { println!("Last row: {:?}", tmp); }
                return height;
            }
            println!("There are {:>8} articles that reach in no fewer than {} steps",
                     last_row.len(), height);

        }
    }
}
