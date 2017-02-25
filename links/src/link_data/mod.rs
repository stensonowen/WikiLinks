use super::{LinkState, LinkDb, LinkData};
use std::collections::HashMap;
use super::Entry;

mod pagerank;

impl From<LinkState<LinkDb>> for LinkState<LinkData> {
    fn from(old: LinkState<LinkDb>) -> LinkState<LinkData> {
        //split into vector 
        //let threads: Vec<Vec<(u32, Entry)>> = Vec::with_capacity(old.threads);
        //let entries_iter = old.state.db.entries.iter();
        //let size = old.state.db.entries.len();
        //for i in 0 .. old.threads-1 {
        //    //threads.push(entries_iter.take(size).map(|e| ).collect());

        //}
        

        LinkState {
            threads: old.threads,
            log: old.log,
            state: LinkData {
                dumps: vec![],
                ranks: HashMap::new(),

            }
        }
    }
}

/*

impl From<StateMachine<StateA>> for StateMachine<StateB> {
    fn from(val: StateMachine<StateA>) -> StateMachine<StateB> {
        StateMachine {
            some_unrelated_value: val.some_unrelated_value,
            state: StateB {
                interm_value: val.state.start_value.split(" ").map(|x| x.into()).collect(),
            }
        }
    }
}


*/
