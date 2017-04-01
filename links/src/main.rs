#![feature(plugin)]
#![plugin(rocket_codegen)]
//#![allow(needless_pass_by_value)]

// NOTE: when scaling, remember to change bool link_db/parse/regexes.rs/IS_SIMPLE

#[macro_use]
extern crate clap;
extern crate links;
use links::cache as db;

extern crate rocket;
extern crate rocket_contrib;
use rocket_contrib::Template;
use rocket::State;

use links::link_state::{LinkState, HashLinks};

use std::str::FromStr;
use links::cache::{self, get_cache, CACHE_SIZE};
use cache::cache_elem::CacheElem;
use cache::new_cache::NewCacheOuter;
use cache::long_cache::LongCacheOuter;
use links::web::{self, Context, CacheSort, SortParam, PathRes, Node};

type SharedLinks<'a>  = State<'a, HashLinks>;
type NewCache<'a>  = State<'a, NewCacheOuter>;
type LongCache<'a> = State<'a, LongCacheOuter>;

const DEFAULT_SORT: CacheSort = CacheSort::Recent;
const CACHEWORTHY_LENGTH: usize = 5;    // only cache searches longer than this

use clap::Arg;
fn argv<'a>() -> clap::ArgMatches<'a> {
    clap::App::new(crate_name!()).about(crate_description!())
        .author(crate_authors!()).version(crate_version!())

        .arg(Arg::with_name("import")
             .long("import")
             .short("i")
             .takes_value(true)
             .help("Import link and title data from link dumps manifest"))
        .arg(Arg::with_name("export")
             .long("output")
             .short("o")
             .takes_value(true)
             .help("Export link and title data to manifest and dumps"))

        .arg(Arg::with_name("compute-ranks")
             .long("ranks")
             .takes_value(true)
             .help("After loading data, compute and save the pageranks")) 
        .arg(Arg::with_name("web-server")
             .short("w")
             .help("Run web server; program will otherwise terminate after analysis"))

        .arg(Arg::with_name("page.sql")
             .short("p")
             .takes_value(true)
             .requires("redirect.sql")
             .requires("pagelinks.sql")
             .help("Pages db from wikipedia dump"))
        .arg(Arg::with_name("redirect.sql")
             .short("r")
             .takes_value(true)
             .requires("page.sql")
             .requires("pagelinks.sql")
             .help("Internal links db from wikipedia dump"))
        .arg(Arg::with_name("pagelinks.sql")
             .short("l")
             .takes_value(true)
             .requires("page.sql")
             .requires("redirect.sql")
             .help("Internal links db from wikipedia dump"))

        .get_matches()
}


#[get("/")]
fn index(nc: NewCache, lc: LongCache) -> Template {
    let cache = match DEFAULT_SORT {
        CacheSort::Recent => nc.get(),
        CacheSort::Length => lc.get(),
    };
    let context = Context::from_cache(DEFAULT_SORT, cache);
    Template::render("bfs", &context)
}

#[get("/bfs", rank = 3)]
fn bfs_empty(nc: NewCache, lc: LongCache) -> Template {
    let cache = match DEFAULT_SORT {
        CacheSort::Recent => nc.get(),
        CacheSort::Length => lc.get(),
    };
    let context = Context::from_cache(DEFAULT_SORT, cache);
    Template::render("bfs", &context)
}

#[get("/bfs?<sort>", rank = 2)]
fn bfs_sort(sort: SortParam, nc: NewCache, lc: LongCache) -> Template {
    let sort = match sort.by {
        Some(s) => CacheSort::from_str(s).unwrap_or(DEFAULT_SORT),
        None => DEFAULT_SORT,
    };
    let cache = match sort {
        CacheSort::Recent => nc.get(),
        CacheSort::Length => lc.get(),
    };
    let context = Context::from_cache(sort, cache);
    Template::render("bfs", &context)
}

#[get("/bfs?<search>", rank = 1)]
fn bfs_search(search: web::SearchParams, conn: db::Conn, links: SharedLinks, 
              nc: NewCache, lc: LongCache) -> Template 
{
    let (src_f, dst_f) = search.fix();
    // TODO: translate empty query into random?
    let src_n = links.lookup_title(src_f.as_ref());
    let dst_n = links.lookup_title(dst_f.as_ref());
    let sort = match search.cache_sort {
        Some(s) => CacheSort::from_str(s).unwrap_or(DEFAULT_SORT),
        None => DEFAULT_SORT,
    };
    let path_res = if let (&Node::Found(s,ss), &Node::Found(d,ds)) = (&src_n, &dst_n) {
        if let Some(db_path) = cache::lookup_path(&*conn, s, d) {
            // return the path that was saved last time
            PathRes::from_db_path(db_path, links.get_links())
        } else {
            // can't find record of previous search; perform for the first time
            let path = links.bfs(s,d);
            if let Some(len) = path.size() {
                if len >= CACHEWORTHY_LENGTH {
                    // TODO: kill the clone
                    cache::insert_path(&conn, path.clone());
                }
                let ce = CacheElem::new(ss, ds, len);
                if lc.should_insert(&ce) {
                    lc.insert_elem(ce.clone());
                }
                nc.insert_elem(ce);
            }
            PathRes::from_path(path, links.get_links())
        }
    } else {
        // invalid request; search not run
        PathRes::NotRun
    };
    let cache = match sort {
        CacheSort::Recent => nc.get(),
        CacheSort::Length => lc.get(),
    };
    let context = Context {
        path:       path_res,
        src_search: src_n,
        dst_search: dst_n,
        cache:      cache,
        cache_sort: sort,
    };
    Template::render("bfs", &context)
}

/*
#[get("/foo")]
fn foo(s: State<RwLock<i32>>) -> String {
    //let t = s.read().unwrap();
    //println!("{:?}", t);
    let mut t = s.write().unwrap();
    *t = 0;
    println!("{:?}", t);
    String::new()
}
*/

fn server(hl_state: LinkState<HashLinks>) {
    // use Arc around this?
    let hl = hl_state.extract();

    let c = cache::establish_connection();
    let lc = match get_cache(&c, hl.get_links(), &CacheSort::Length, CACHE_SIZE) {
        Some(l) => LongCacheOuter::from(l),
        None => LongCacheOuter::new(),
    };
    let nc = match get_cache(&c, hl.get_links(), &CacheSort::Recent, CACHE_SIZE) {
        Some(n) => NewCacheOuter::from(n),
        None => NewCacheOuter::new(),
    };

    rocket::ignite()
        .manage(db::init_pool())
        .manage(hl)
        .manage(lc)
        .manage(nc)
        .mount("/", routes![index, bfs_empty, bfs_sort, bfs_search, /*foo*/])
        .launch();

}

fn main() {
    let hl_state = LinkState::<HashLinks>::from_args(argv());
    if let Some(hl) = hl_state {
        server(hl);
    } else {
        println!("Finished analytics; not starting a web server");
    }
}

