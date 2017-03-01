use std::collections::HashMap;
use super::{LinkState, LinkData, RankData};

mod pagerank;

//#[derive(Serialize, Deserialize)]
pub struct RankedEntry {
    title: String,
    pagerank: f64,
    parents:  Vec<u32>,
    children: Vec<u32>,
}


impl From<LinkState<LinkData>> for LinkState<RankData> {
    fn from(old: LinkState<LinkData>) -> LinkState<RankData> {
        //let (entries, addresses) = old.state.db.explode();
        //let pr_log = old.log.new(o!("elenemts" => entries.len()));
        //let ranks = pagerank::Graph::new(&entries).get_ranks(pr_log);

        // addresses and ranks feed into PostgreSQL
        // entries will become into lookup table

        LinkState {
            threads:    old.threads,
            size:       old.size,
            log:        old.log,
            state:      RankData {
                links: HashMap::new(),
                ranks: vec![],
            }
        }
    }
}

