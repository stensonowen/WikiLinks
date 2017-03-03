//use std::collections::HashMap;
use super::{LinkState, RankData, HashLinks};
use super::{LinkData, new_logger};
use std::path::{Path, PathBuf};
use clap;
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

impl LinkState<HashLinks> {
    fn from_args(args: clap::ArgMatches) -> Self {
        //populate complete HashLinks from command-line args

        //first, decide whether to build links from source sql or previous backup
        let ls_dt: LinkState<LinkData> = if let (Some(p), Some(r), Some(l)) = 
            (args.value_of("page.sql"), 
             args.value_of("redirect.sql"), 
             args.value_of("pagelinks.sql")) 
        {
            LinkState::new(PathBuf::from(p), PathBuf::from(r), PathBuf::from(l))
                .into()
        } else if let Some(m) = args.value_of("manifest") {
            LinkState::<LinkData>::from_file(PathBuf::from(m), new_logger()).unwrap()
        } else {
            //clap should make this impossible
            unreachable!()
        };

        //then decide whether to build pagelinks from data or import from backup
        let ls_rd = match args.value_of("ranks") {
            Some(r) => LinkState::<RankData>::from_ranks(ls_dt, Path::new(r)),
            None => ls_dt.into(),
        };

        //convert to HashLinks
        ls_rd.into()
    }
}
