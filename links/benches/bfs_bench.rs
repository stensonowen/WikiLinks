#![feature(test)]
#![allow(unknown_lints, unreadable_literal)]

/*
#![feature(alloc_system, global_allocator, allocator_api)]
extern crate alloc_system;
use alloc_system::System;
#[global_allocator] static A: System = System;
*
* valgrind results:
*   _cur    2,883,084,321
*   _alt    2,419,414,143
*/

#[macro_use] 
extern crate lazy_static;
extern crate links;
extern crate test;

use links::link_state::{LinkState, HashLinks, LinkData, new_logger};
use std::path::PathBuf;

use links::link_state::link_table::PageIndex;

const BENCH_MANIFEST_PATH: &str = "/home/owen/rust/wl/simple/dump3";

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
    use links::link_state::Path as BfsPath;

    /*  Note: tests w/ time measurements weren't all done on the same system
     *      Simple wiki tests ("*") were done on an i7 4600U
     *      English wiki tests ("**") were done on an i5 2300
     */

    /// Generic benching for breadth-first searching (either bfs or bfs2)
    fn bfs_bench_g<F>(b: &mut Bencher, bfs_fn: F, src: u32, dst: u32, len: usize)
        where F: Fn(PageIndex, PageIndex) -> BfsPath
    {
        // be sure to init the data structure before beginning the benchmark
        lazy_static::initialize(&HL);
        let src_index = HL.get_index(src.into()).unwrap();
        let dst_index = HL.get_index(dst.into()).unwrap();
        b.iter(|| {
            let p = bfs_fn(src_index, dst_index);
            assert_eq!(Some(len), p.len());
        });
    }
    // stubs to make testing a little clearer
    fn bfs1(src: PageIndex, dst: PageIndex) -> BfsPath { HL.bfs(src, dst) }
    fn bfs2(src: PageIndex, dst: PageIndex) -> BfsPath { HL.bfs2(src, dst) }



    /// Bench small searches
    fn bfs_small_g<F: Fn(PageIndex,PageIndex)->BfsPath>(b: &mut Bencher, bfs_fn: F) {
        if cfg!(feature="simple") {
            // takes ~1 μs *
            // Elkton,_Kentucky → United_States → New_York_City → 
            //  St._Patrick\'s_Cathedral_(New_York)
            bfs_bench_g(b, bfs_fn, 155171, 228159, 3)
        } else {
            // takes ~1 μs **
            // Kabuki_Rocks → List_of_video_game_musicians → Crypt_of_the_NecroDancer
            bfs_bench_g(b, bfs_fn, 31073639, 41637705, 2)
        }
    }
    #[bench] fn bfs_small_cur(b: &mut Bencher) { bfs_small_g(b, bfs1) }
    #[bench] fn bfs_small_alt(b: &mut Bencher) { bfs_small_g(b, bfs2) }



    /// Bench medium searches
    fn bfs_medium_g<F: Fn(PageIndex,PageIndex)->BfsPath>(b: &mut Bencher, bfs_fn: F) {
        if cfg!(feature="simple") {
            // takes ~300 μs *
            // Jim Jones → November 18 → April 28 → Jessica Alba → The Office
            bfs_bench_g(b, bfs_fn, 15030, 129573, 4)
        } else {
            // takes ~1 ms **
            // Palazzo_del_Capitano_del_Popolo,_Gubbio → Gothic_architecture →
            //  Oakland,_California → Oakland_firestorm_of_1991 → Detwiler_Fire
            bfs_bench_g(b, bfs_fn, 38394676, 54576532, 4)
        }
    }
    #[bench] fn bfs_medium_cur(b: &mut Bencher) { bfs_medium_g(b, bfs1) }
    #[bench] fn bfs_medium_alt(b: &mut Bencher) { bfs_medium_g(b, bfs2) }



    /// Bench large searches
    fn bfs_large_g<F: Fn(PageIndex,PageIndex)->BfsPath>(b: &mut Bencher, bfs_fn: F) {
        if cfg!(feature="simple") {
            // takes ~150 ms *
            // Macclenny,_Florida → United_States → November_11 → 2004 → 
            //  Vasaloppet → Tjejvasan → Annika_Evaldsson → 
            //  Swedish_Cross-Country_Skiing_Championships → Saltsjöbaden → 
            //  Grand_Hotel_Saltsjöbaden → Saltsjöbaden_Agreement
            bfs_bench_g(b, bfs_fn, 152629, 454989, 10)
        } else {
            // takes ~500 ms ** (any larger to `cargo bench` takes forever
            // Jack_Tatum → Ohio_State_Buckeyes_football → Indiana_Hoosiers_football
            //  → David_Starr_Jordan → Caulophryne_jordani → Caulophryne
            //  → Caulophryne_bacescui
            bfs_bench_g(b, bfs_fn, 1684129, 52186157, 6)
        }
    }
    #[bench] fn bfs_large_cur(b: &mut Bencher) { bfs_large_g(b, bfs1) }
    #[bench] fn bfs_large_alt(b: &mut Bencher) { bfs_large_g(b, bfs2) }

}

