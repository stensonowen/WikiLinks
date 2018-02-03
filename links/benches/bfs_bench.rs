#![feature(test)]

#[macro_use] 
extern crate lazy_static;
extern crate links;
extern crate test;

use links::link_state::{LinkState, HashLinks, LinkData, new_logger};
use std::path::PathBuf;

const BENCH_MANIFEST_PATH: &'static str = "/home/owen/rust/wl/simple/dump1";

lazy_static! {
    static ref HL: LinkState<HashLinks> = {
        let m = PathBuf::from(BENCH_MANIFEST_PATH);
        let ls_dt = LinkState::<LinkData>::import(m, new_logger()).unwrap();
        ls_dt.into()
    };
}

 
#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bfs_small(b: &mut Bencher) {
        let src = 527998; // John McAfee
        let dst = 159076; // United States
        // len=3: John McAfee → YouTube → United States
        b.iter(|| {
            HL.bfs(src, dst);
        });
    }

    #[bench]
    fn bfs_medium(b: &mut Bencher) {
        let src = 15030;  // Jim Jones
        let dst = 129573; // The Office (U.S. TV series)
        // len=5: Jim Jones → November 18 → April 28 → Jessica Alba → The Office
        b.iter(|| {
            HL.bfs(src, dst);
        });
    }

    // Yobibyte: 401967  →  1961-62_AHL_season: 309528
    #[bench]
    fn bfs_long(b: &mut Bencher) {
        let src = 401967; // Yobibyte
        let dst = 309528; // 1961-62 AHL season
        // len=11: Yobibyte → Zebibyte → Exbibyte → Exabyte 
        //          → Unit of measurement → Day → June 14 → Vancouver Canucks 
        //          → NHL All-Star Team → Danny Lewicki → 1961-62 AHL season
        b.iter(|| {
            HL.bfs(src, dst);
        });
    }

    #[bench]
    fn bfs_small_2(b: &mut Bencher) {
        let src = 527998; // John McAfee
        let dst = 159076; // United States
        // len=3: John McAfee → YouTube → United States
        b.iter(|| {
            HL.bfs2(src, dst);
        });
    }

    #[bench]
    fn bfs_medium_2(b: &mut Bencher) {
        let src = 15030;  // Jim Jones
        let dst = 129573; // The Office (U.S. TV series)
        // len=5: Jim Jones → November 18 → April 28 → Jessica Alba → The Office
        b.iter(|| {
            HL.bfs2(src, dst);
        });
    }

    // Yobibyte: 401967  →  1961-62_AHL_season: 309528
    #[bench]
    fn bfs_long_2(b: &mut Bencher) {
        let src = 401967; // Yobibyte
        let dst = 309528; // 1961-62 AHL season
        // len=11: Yobibyte → Zebibyte → Exbibyte → Exabyte 
        //          → Unit of measurement → Day → June 14 → Vancouver Canucks 
        //          → NHL All-Star Team → Danny Lewicki → 1961-62 AHL season
        b.iter(|| {
            HL.bfs2(src, dst);
        });
    }

}

