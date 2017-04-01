use csv;
use fnv::FnvHashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::cmp::Ordering;
use std::{f64, u64};

use super::{LinkState, LinkData, ProcData};
use super::Entry;
use super::link_data::IndexedEntry;

mod pagerank;


impl From<LinkState<LinkData>> for LinkState<ProcData> {
    fn from(old: LinkState<LinkData>) -> LinkState<ProcData> {
        // move addrs and entries from LinkData and compute pageranks
        // single threaded population for now
        let links = Self::consolidate_links(old.state.dumps, old.size);
        LinkState {
            threads:    old.threads,
            size:       old.size,
            log:        old.log,
            state:      ProcData {
                titles: old.state.titles,
                links:  links,
            }
        }
    }
}

impl LinkState<ProcData> {
    pub fn compute_ranks(&mut self, path: &PathBuf) -> Result<(), csv::Error> {
        let pr_log = self.log.new(o!(
                "damping" => pagerank::DAMPING_FACTOR,
                "epsilon" => pagerank::MAX_ERROR));
        let r = pagerank::Graph::new(&self.state.links).get_ranks(pr_log);
        // sort floats; will all be less than 
        // so should be the same as sorting by the negative reciprocal
        let mut sorted_r: Vec<_> = r.into_iter().collect();
        sorted_r.sort_by_key(|&(_,r)| {
            assert!(r.is_normal());
            assert!(r.is_sign_positive());
            assert!(r <= 1.0);
            u64::MAX - r.recip() as u64
        });
        let mut csv_w = csv::Writer::from_file(path)?;
        for (id,rank) in sorted_r {
            let ref title = self.state.links.get(&id).unwrap().title;
            csv_w.encode((rank,id,title))?;
        }
        Ok(())
    }

    fn consolidate_links(links: Vec<Mutex<Vec<IndexedEntry>>>, size: usize) 
        -> FnvHashMap<u32,Entry> 
    {
        let mut hm: FnvHashMap<u32,Entry> = 
            FnvHashMap::with_capacity_and_hasher(size, Default::default());
        for bucket in links {
            for ie in bucket.into_inner().unwrap() {
                let (id, entry) = ie.decompose();
                hm.insert(id, entry);
            }
        }
        hm
    }
    fn _pretty_ranks(&self, ranks: &[(u32,f64)], ranks_path: &Path) -> Result<(),csv::Error> {
        //sort greatest-to-least
        // (RANK, ID, TITLE)
        let mut sorted_ranks = ranks.to_vec();
        sorted_ranks.sort_by(|&(a_i,a_r),&(b_i,b_r)| {
            //sort by floats, which Ord does not provide
            assert!(!a_r.is_nan(), "Page {} had a NaN rank", a_i);
            assert!(!b_r.is_nan(), "Page {} had a NaN rank", b_i);
            //match (a_r > b_r, a_r == b_r) {
            match (a_r > b_r, (a_r - b_r).abs() < f64::EPSILON) {
                (true, _) => Ordering::Less,
                (_, true) => Ordering::Equal,
                _         => Ordering::Greater,
            }
        });
        
        // write using interesting csv data
        let mut csv_w = csv::Writer::from_file(ranks_path)?;
        for (id,rank) in sorted_ranks {
            let title = &self.state.links[&id].title;
            csv_w.encode((rank,id,title))?;
        }
        Ok(())
    }
}
