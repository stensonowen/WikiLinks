use csv;
use slog;
use std::path::{self, PathBuf};
use std::cmp::Ordering;
use std::{f64, u64};

use super::{LinkState, LinkData, ProcData};
use super::{bfs, Path};
use self::bfs::BFS;

mod pagerank;
mod longest_path;

impl From<LinkState<LinkData>> for LinkState<ProcData> {
    fn from(old: LinkState<LinkData>) -> LinkState<ProcData> {
        // move addrs and entries from LinkData and compute pageranks
        // single threaded population for now
        let (threads, size) = (old.threads, old.size);
        let (links, log, _) = old.break_down();
        LinkState {
            threads:    threads,
            size:       size,
            log:        log,
            state:      ProcData {
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
        let r = pagerank::Graph::new(&self.state.links).get_ranks(&pr_log);
        // sort floats; will all be less than 
        // so should be the same as sorting by the negative reciprocal
        let mut sorted_r: Vec<_> = r.into_iter().collect();
        sorted_r.sort_by_key(|&(_,r)| {
            assert!(r.is_normal());
            assert!(r.is_sign_positive());
            assert!(r <= 1.0);
            r.recip() as u64
        });
        let mut csv_w = csv::Writer::from_file(path)?;
        for (id,rank) in sorted_r {
            let title = &self.state.links[&id].title;
            csv_w.encode((rank,id,title))?;
        }
        Ok(())
    }

    pub fn longest_path(&self, dst: u32) -> u8 {
        self.state.longest_path(dst)
    }

    fn _pretty_ranks(&self, ranks: &[(u32,f64)], ranks_path: &path::Path) 
        -> Result<(),csv::Error> 
    {
        //sort greatest-to-least
        // (RANK, ID, TITLE)
        let mut sorted_ranks = ranks.to_vec();
        sorted_ranks.sort_by(|&(a_i,a_r),&(b_i,b_r)| {
            //sort by floats, which Ord does not provide
            assert!(!a_r.is_nan(), "Page {} had a NaN rank", a_i);
            assert!(!b_r.is_nan(), "Page {} had a NaN rank", b_i);
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
    pub fn neighbor_redundancy(&self) -> usize {
        use std::collections::HashSet;
        // count number of nodes that are present in both `children` and `parents`
        self.state.links.values().map(|e| {
            let children: HashSet<u32> = e.get_children().iter().cloned().collect();
            assert_eq!(e.get_children().len(), children.len());
            let parents: HashSet<u32> = e.get_parents().iter().cloned().collect();
            assert_eq!(e.get_parents().len(), parents.len());
            children.intersection(&parents).count()
        }).sum()
    }
    pub fn contains(&self, n: u32) -> bool {
        self.state.links.contains_key(&n)
    }
    pub fn bfs(&self, src: u32, dst: u32) -> Path {
        let null = slog::Logger::root(slog::Discard, o!());
        let bfs = BFS::new(null, &self.state.links, src, dst);
        bfs.search()
    }
    pub fn get(&self, i: u32) -> &String {
        self.state.links.get(&i).map(|n| &n.title).unwrap()
    }
    //pub fn random_select(&self) -> u32 { }
    //pub fn bfs(&self, src: u32, dst: u32) -> {}
}
