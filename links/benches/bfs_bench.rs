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
    use links::link_state::Path as BfsPath;

    /// Generic benching for breadth-first searching (either bfs or bfs2)
    fn bfs_bench_g<F>(b: &mut Bencher, bfs_fn: F, src: u32, dst: u32, len: usize)
        where F: Fn(u32, u32) -> BfsPath
    {
        // be sure to init the data structure before beginning the benchmark
        lazy_static::initialize(&HL);
        b.iter(|| {
            let p = bfs_fn(src, dst);
            assert_eq!(Some(len), p.len());
        });
    }
    // stubs to make testing a little clearer
    fn bfs1(src: u32, dst: u32) -> BfsPath { HL.bfs(src, dst) }
    fn bfs2(src: u32, dst: u32) -> BfsPath { HL.bfs2(src, dst) }



    /// Bench small searches
    fn bfs_small_g<F: Fn(u32,u32)->BfsPath>(b: &mut Bencher, bfs_fn: F) {
        if cfg!(feature="simple") {
            // John McAfee → YouTube → United States
            bfs_bench_g(b, bfs_fn, 527998, 219587, 2)
        } else {
            // Beelzebub → Pharisees → History_of_the_Jews_in_Japan → Steven_Seagal
            bfs_bench_g(b, bfs_fn, 19010124, 67404, 3);
        }
    }
    #[bench] fn bfs_small_1(b: &mut Bencher) { bfs_small_g(b, bfs1) }
    #[bench] fn bfs_small_2(b: &mut Bencher) { bfs_small_g(b, bfs2) }



    /// Bench medium searches
    fn bfs_medium_g<F: Fn(u32,u32)->BfsPath>(b: &mut Bencher, bfs_fn: F) {
        if cfg!(feature="simple") {
            // Jim Jones → November 18 → April 28 → Jessica Alba → The Office
            bfs_bench_g(b, bfs_fn, 15030, 129573, 4)
        } else {
            // Blind_(The_Sundays_album) → Guitar → Spruce → Pseudotsuga →
            //  List_of_Tortricidae_genera → Auratonota
            bfs_bench_g(b, bfs_fn, 3214166, 25765644, 5)
        }
    }
    #[bench] fn bfs_medium_1(b: &mut Bencher) { bfs_medium_g(b, bfs1) }
    #[bench] fn bfs_medium_2(b: &mut Bencher) { bfs_medium_g(b, bfs2) }



    /// Bench large searches
    fn bfs_large_g<F: Fn(u32,u32)->BfsPath>(b: &mut Bencher, bfs_fn: F) {
        if cfg!(feature="simple") {
            // Yobibyte → Zebibyte → Exbibyte → Exabyte → 
            //  Unit of measurement → Day → June 14 → Vancouver Canucks → 
            //  NHL All-Star Team → Danny Lewicki → 1961-62 AHL season
            bfs_bench_g(b, bfs_fn, 401967, 309528, 10)
        } else {
            // Tanhc_function → Sinhc_function → Trigonometric_integral → Spiral →
            //  Museum_of_Fine_Arts,_Boston → Ukiyo-e → Wildlife_of_Japan →
            //  List_of_moths_of_Japan → List_of_moths_of_Japan_(Pyraloidea-Drepanoidea) 
            //  → Paracymoriza_okinawanus
            bfs_bench_g(b, bfs_fn, 45691392, 43386956, 9)
        }
    }
    #[bench] fn bfs_large_1(b: &mut Bencher) { bfs_large_g(b, bfs1) }
    #[bench] fn bfs_large_2(b: &mut Bencher) { bfs_large_g(b, bfs2) }

}

