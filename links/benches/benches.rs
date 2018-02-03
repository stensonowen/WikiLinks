/*
 * NOTE: this is mostly testing web/db stuff
 * should be re-added when the web stuff is
#![feature(test)]

#[macro_use] 
extern crate lazy_static;
extern crate fnv;
extern crate links;
extern crate test;

use links::link_state::{LinkState, LinkData, ProcData, HashLinks, new_logger};
use links::cache;

use std::path::{Path, PathBuf};

//const BENCH_MANIFEST_PATH: &'static str = "/home/owen/wikidata/dumps/simple_20170201_dump2";
//const BENCH_RANKS_PATH: &'static str = "/home/owen/wikidata/dumps/simple_20170201_ranks2";
const BENCH_MANIFEST_PATH: &'static str = "/home/owen/wikidata/dumps3/links.json";
const BENCH_RANKS_PATH: &'static str = "/home/owen/wikidata/dumps3/metadata.json";
const IS_SIMPLE: bool = true;

lazy_static! {
    static ref HL: HashLinks = {
        let m = PathBuf::from(BENCH_MANIFEST_PATH);
        let r = Path::new(BENCH_RANKS_PATH);

        //let ls_dt = LinkState::<LinkData>::from_file(m, new_logger()).unwrap();
        let ls_dt = LinkState::<LinkData>::import(m, new_logger()).unwrap();
        //let ls_rd = LinkState::<ProcData>::from_ranks(ls_dt, r);
        let mut ls_rd: LinkState<ProcData> = ls_dt.into();
        ls_rd.import(r);
        //let ls_rd = LinkState::<ProcData>::import(ls_dt, r);
        
        let ls_hl: LinkState<HashLinks> = ls_rd.into();
        ls_hl.extract()
    };
    //static ref DB: Mutex<PgConnection> = Mutex::new(cache::establish_connection());
}

 
pub fn add_two(a: i32) -> i32 {
    a + 2
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn lookup_string_db(b: &mut Bencher) {
        //select random article title (should be negligible)
        let db = cache::establish_connection();
        //let (_, title) = HL.random_id_and_title();
        // different title every time: 57,918 (-145)
        // same title every time:  55,501 
        //let title = "Rensselaer Polytechnic Institute";
        let title = "United States of America";
        b.iter(|| {
            //let (_, title) = HL.random_id_and_title();
            cache::lookup_addr(&db, title);
        })
    }

    #[bench]
    fn lookup_titles(b: &mut Bencher) {
        let title = "United States of America";
        b.iter(|| {
            HL.lookup_title(title);
        })
    }
    /*
    #[bench]
    fn lookup_string_db_acceptable_overhead(b: &mut Bencher) {
        // the setup time for `lookup-_string_db`
        b.iter(|| {
            let (_, _title) = HL.random_id_and_title();
        })
    }
    */

    // Times to request history in different orders
    // Results:     Sort,   Microseconds
    //              Length  204
    //              Popular 226
    //              Recent  256
    //  tl;dr: it's expensive, and should probably be replaced
    #[bench]
    fn get_cache_recent(b: &mut Bencher) {
        let sort = links::web::CacheSort::Recent;
        let db = cache::establish_connection();
        b.iter(|| {
            cache::get_cache(&db, HL.get_links(), &sort, 15);
        })
    }
    #[bench]
    fn get_cache_popular(b: &mut Bencher) {
        let sort = links::web::CacheSort::Popular;
        let db = cache::establish_connection();
        b.iter(|| {
            cache::get_cache(&db, HL.get_links(), &sort, 15);
        })
    }
    #[bench]
    fn get_cache_length(b: &mut Bencher) {
        let sort = links::web::CacheSort::Length;
        let db = cache::establish_connection();
        b.iter(|| {
            cache::get_cache(&db, HL.get_links(), &sort, 15);
        })
    }

    #[bench]
    fn path_cache_lookup(b: &mut Bencher) {
        // select a medium-length path
        // path length: 4
        let (src, dst) = if IS_SIMPLE {
            // "Pope_Anastasius_IV" to "South_West_England": 4 iterations
            (359684, 54327)
        } else {
            // "Dartmouth" to "John_Weber_(darts_player)": 4 iterations
            (19701041, 44781296)
        };
        let db = cache::establish_connection();

        b.iter(|| {
            cache::lookup_path(&db, src, dst);
        })
    }
    #[bench]
    fn path_cache_bfs_med(b: &mut Bencher) {
        let (src, dst) = if IS_SIMPLE {
            // "Pope_Anastasius_IV" to "South_West_England": 4 iterations
            (359684, 54327)
        } else {
            // "Dartmouth" to "John_Weber_(darts_player)": 4 iterations
            (19701041, 44781296)
        };
        assert!(HL.get_links().get(&src).is_some());
        assert!(HL.get_links().get(&dst).is_some());
        assert_eq!(HL.bfs(src, dst).path.unwrap().len(), 5);

        b.iter(|| {
            HL.bfs(src, dst);
        })
    }
    #[bench]
    fn path_cache_bfs_short(b: &mut Bencher) {
        let (src, dst) = if IS_SIMPLE {
            // "Canada" to "Organization": 2 iterations
            (219589, 32644)
        } else {
            // "Richard_Stallman" to "Jesus": 2 iterations
            (3434143, 1095706)
        };
        assert!(HL.get_links().get(&src).is_some());
        assert!(HL.get_links().get(&dst).is_some());
        assert_eq!(HL.bfs(src, dst).path.unwrap().len(), 3);

        b.iter(|| {
            HL.bfs(src, dst);
        })
    }
    #[bench]
    fn path_cache_bfs_long(b: &mut Bencher) {
        let (src, dst) = if IS_SIMPLE {
            // "Glow_in_the_dark" to "Yolande_Fox": 7 iterations
            (359127, 533446)
        } else {
            // "Joys_Sebastian" to "Philorene_texturata": 
            (21689617, 36955430)
        };
        assert!(HL.get_links().get(&src).is_some());
        assert!(HL.get_links().get(&dst).is_some());
        assert_eq!(HL.bfs(src, dst).path.unwrap().len(), 8);

        b.iter(|| {
            HL.bfs(src, dst);
        })
    }

}
*/
