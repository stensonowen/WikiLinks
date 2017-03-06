use csv;
use std::path::Path;
use std::sync::Mutex;
use super::{LinkState, LinkData, RankData};
use super::Entry;
use super::link_data::IndexedEntry;
use fnv::FnvHashMap;
use std::cmp::Ordering;

mod pagerank;


impl From<LinkState<LinkData>> for LinkState<RankData> {
    fn from(old: LinkState<LinkData>) -> LinkState<RankData> {
        // move addrs and entries from LinkData and compute pageranks
        
        // single threaded population
        let links = Self::consolidate_links(old.state.dumps, old.size); 

        // compute pageranks
        let pr_log = old.log.new(o!(
                "damping" => pagerank::DAMPING_FACTOR,
                "epsilon" => pagerank::MAX_ERROR));
        let r = pagerank::Graph::new(&links).get_ranks(pr_log);

        LinkState {
            threads:    old.threads,
            size:       old.size,
            log:        old.log,
            state:      RankData {
                links: links,
                ranks: r.into_iter().collect(),
            }
        }
    }
}

impl LinkState<RankData> {
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
    pub fn from_ranks(old: LinkState<LinkData>, ranks_path: &Path) -> Self {
        let links = Self::consolidate_links(old.state.dumps, old.size);
        //populate ranks from csv file
        //let mut ranks: HashMap<u32,f64> = HashMap::with_capacity(old.size);
        //let mut ranks: fnv::FnvHashMap<u32,f64> = 
        //    FnvHashMap::with_capacity_and_hasher(old.size, Default::default());
        let mut ranks: Vec<(u32,f64)> = Vec::with_capacity(old.size);

        let mut csv_r = csv::Reader::from_file(ranks_path)
            .unwrap().has_headers(false);
        for line in csv_r.decode() {
            let (id, rank): (u32, f64) = line.unwrap();
            //ranks.insert(id, rank);
            ranks.push((id, rank));
        }

        LinkState {
            threads: old.threads,
            size:    old.size,
            log:     old.log,
            state:   RankData {
                links: links,
                ranks: ranks.into_iter().collect(),
            }
        }
    }
    pub fn save_ranks(&self, ranks_path: &Path) -> Result<(),csv::Error> {
        // note about writing/reading files: they won't be identical
        // because we're iterating through a hashmap
        // but they are identical content-wise
        let mut csv_w = csv::Writer::from_file(ranks_path)?;
        for datum in &self.state.ranks {
            csv_w.encode(datum)?;
        }
        Ok(())
    }
    pub fn pretty_ranks(&self, ranks_path: &Path) -> Result<(),csv::Error> {
        //sort greatest-to-least
        // (RANK, ID, TITLE)
        //let mut sorted_ranks: Vec<_> = self.state.ranks.iter().collect();
        let mut sorted_ranks = self.state.ranks.clone();
        sorted_ranks.sort_by(|&(a_i,a_r),&(b_i,b_r)| {
            //sort by floats, which Ord does not provide
            assert!(!a_r.is_nan(), "Page {} had a NaN rank", a_i);
            assert!(!b_r.is_nan(), "Page {} had a NaN rank", b_i);
            match (a_r > b_r, a_r == b_r) {
                (true, _) => Ordering::Less,
                (_, true) => Ordering::Equal,
                _         => Ordering::Greater,
            }
        });
        
        // write using interesting csv data
        let mut csv_w = csv::Writer::from_file(ranks_path)?;
        for (id,rank) in sorted_ranks {
            let ref title = self.state.links.get(&id).unwrap().title;
            csv_w.encode((rank,id,title))?;
        }
        Ok(())
    }
    pub fn data(&self) {
        info!(self.log, "State of RankData:");
        info!(self.log, "Number of entries: {}", self.state.links.len());
        info!(self.log, "Number of ranks: {}", self.state.ranks.len());
        let rank_sum: f64 = self.state.ranks.iter().map(|&(_,r)| r).sum();
        let total_links: usize = self.state.links
            .iter()
            .map(
                |(_, &Entry{ children: ref c, parents: ref p, .. })| 
                c.len() + p.len())
            .sum();
        info!(self.log, "Sum of all ranks:   {}", rank_sum);
        info!(self.log, "Number of links:    {}", total_links);
    }
}
