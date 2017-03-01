use std::collections::HashMap;
use super::{LinkState, LinkData, RankData};
use Entry;

mod pagerank;

//pub struct RankedEntry {
//    title: String,
//    pagerank: f64,
//    parents:  Vec<u32>,
//    children: Vec<u32>,
//}


impl From<LinkState<LinkData>> for LinkState<RankData> {
    fn from(old: LinkState<LinkData>) -> LinkState<RankData> {
        //let (entries, addresses) = old.state.db.explode();
        //let pr_log = old.log.new(o!("elenemts" => entries.len()));
        //let ranks = pagerank::Graph::new(&entries).get_ranks(pr_log);

        // addresses and ranks feed into PostgreSQL
        // entries will become into lookup table
        
        // single threaded population
        let mut links: HashMap<u32,Entry> = HashMap::with_capacity(old.size);
        for dump in old.state.dumps {
            for (id, entry) in dump.into_inner().unwrap() {
                links.insert(id, entry);
            }
        }

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
                ranks: r,
            }
        }
    }
}

impl LinkState<RankData> {
    //fn to_file(&self) -> Result<(),()> { }
    //fn from_file(mn: BufPath) -> Self { }
    pub fn data(&self) {
        info!(self.log, "State of RankData:");
        info!(self.log, "Number of entries: {}", self.state.links.len());
        info!(self.log, "Number of ranks: {}", self.state.ranks.len());
        let rank_sum: f64 = self.state.ranks.iter().map(|(_,&r)| r).sum();
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
