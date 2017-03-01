//use std::collections::HashMap;
use super::{LinkState, RankData, HashLinks};
//use Entry;

impl From<LinkState<RankData>> for LinkState<HashLinks> {
    fn from(old: LinkState<RankData>) -> LinkState<HashLinks> {
        //TODO: create DB or something?
        LinkState {
            threads:    old.threads,
            size:       old.size,
            log:        old.log,
            state:      HashLinks {
                links: old.state.links
            }
        }
    }
}
