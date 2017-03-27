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
use links::cache::{self, get_cache};
use links::cache::stack_cache::StackCache;
use links::web::{self, Context, CacheSort, SortParam, PathRes, Node};

use std::sync::{Arc, RwLock, Mutex};

//use links::cache::{self, lookup_addr};
type SharedLinks<'a> = State<'a, HashLinks>;
//type SharedCache<'a, 'b> = State<'a, StackCache<'b>>;
type SharedCache<'a> = State<'a, StackCache>;

const DEFAULT_SORT: CacheSort = CacheSort::Recent;
const DEFAULT_SIZE: u32 = 15;
const CACHEWORTHY_LENGTH: usize = 5;    // only cache searches longer than this

use clap::Arg;
fn argv<'a>() -> clap::ArgMatches<'a> {
    clap::App::new(crate_name!()).about(crate_description!())
        .author(crate_authors!()).version(crate_version!())

        .arg(Arg::with_name("import_links")
             .long("import_links")
             .takes_value(true)
             .help("Import link data from link dumps manifest"))
        .arg(Arg::with_name("export_links")
             .long("export_links")
             .takes_value(true)
             .help("Export link data to manifest and dumps"))
        .arg(Arg::with_name("import_md")
             .long("import_md")
             .takes_value(true)
             .help("Import rank/title data from metadata manifest"))
        .arg(Arg::with_name("export_md")
             .long("export_md")
             .takes_value(true)
             .help("Export rank/title data to manifest and dumps"))

        .arg(Arg::with_name("compute_ranks")
             .long("compute_ranks")
             .help("If not provided by a manifest, pageranks will be computed"))

        .arg(Arg::with_name("page.sql")
             .long("page.sql")
             .takes_value(true)
             .conflicts_with("import_links")
             .requires("redirect.sql")
             .requires("pagelinks.sql"))
        .arg(Arg::with_name("redirect.sql")
             .long("redirect.sql")
             .takes_value(true)
             .conflicts_with("import_links")
             .requires("page.sql")
             .requires("pagelinks.sql"))
        .arg(Arg::with_name("pagelinks.sql")
             .long("pagelinks.sql")
             .takes_value(true)
             .conflicts_with("import_links")
             .requires("page.sql")
             .requires("redirect.sql"))

        .group(clap::ArgGroup::with_name("sources")
               .required(true)
               .args(&["import_links", "page.sql"]))
        .get_matches()
}


#[get("/")]
fn index(conn: db::Conn, links: SharedLinks, cache: SharedCache) -> Template {
    let sort = DEFAULT_SORT;
    //let cache = get_cache(&conn, links.get_links(), &sort, DEFAULT_SIZE);
    //let history = cache.get(&sort);
    //let context = Context::from_cache(sort, Some(history));
    let context = Context::from_cache(sort, None);
    Template::render("bfs", &context)
}

#[get("/bfs", rank = 3)]
fn bfs_empty(conn: db::Conn, links: SharedLinks) -> Template {
    let cache = get_cache(&conn, links.get_links(), &DEFAULT_SORT, DEFAULT_SIZE);
    //let context = Context::from_cache(DEFAULT_SORT, cache);
    let context = Context::from_cache(DEFAULT_SORT, None);
    Template::render("bfs", &context)
}

#[get("/bfs?<sort>", rank = 2)]
fn bfs_sort(sort: SortParam, conn: db::Conn, links: SharedLinks) -> Template {
    let sort = match sort.by {
        Some(s) => CacheSort::from_str(s).unwrap_or(DEFAULT_SORT),
        None => DEFAULT_SORT,
    };
    let cache = get_cache(&conn, links.get_links(), &sort, DEFAULT_SIZE);
    //let context = Context::from_cache(sort, cache);
    let context = Context::from_cache(sort, None);
    Template::render("bfs", &context)
}

#[get("/bfs?<search>", rank = 1)]
fn bfs_search(search: web::SearchParams, conn: db::Conn, 
                  links: SharedLinks) -> Template 
{
    let (src_f, dst_f) = search.fix();
    // TODO: translate empty query into random?
    let src_n = if src_f.is_empty() {
        Node::Unused
    } else {
        links.lookup_title(src_f.as_ref())
    };
    let dst_n = if dst_f.is_empty() {
        Node::Unused
    } else {
        links.lookup_title(dst_f.as_ref())
    };
    let path_res = if let (&Node::Found(s,..), &Node::Found(d,..)) = (&src_n, &dst_n) {
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
            }
            PathRes::from_path(path, links.get_links())
        }
    } else {
        // invalid request; search not run
        PathRes::NotRun
    };
    let sort = match search.cache_sort {
        Some(s) => CacheSort::from_str(s).unwrap_or(DEFAULT_SORT),
        None => DEFAULT_SORT,
    };
    //let cache = get_cache(&conn, links.get_links(), &sort, DEFAULT_SIZE);
    let context = Context {
        path:       path_res,
        src_search: src_n,
        dst_search: dst_n,
        //cache:      cache,
        cache:      None,
        cache_sort: sort,
    };
    //println!("Context: {:?}", context);
    Template::render("bfs", &context)
}

#[get("/foo")]
fn foo(s: State<RwLock<i32>>) -> String {
    //let t = s.read().unwrap();
    //println!("{:?}", t);
    let mut t = s.write().unwrap();
    *t = 0;
    println!("{:?}", t);
    String::new()
}

fn main() {
    // get links hashmap
    // uhhhh, will .manage() do a bunch of memmoves?? sure hope not
    // use an Arc/Rc/Cell/ something?
    let hl_state = LinkState::<HashLinks>::from_args(argv());
    //let conn = cache::establish_connection();
    //cache::populate_addrs(&conn, hl_state.get_links(), hl_state.get_ranks()).unwrap();
    let hl = hl_state.extract();

    let cache = StackCache::blank();
    let x = RwLock::new(42);

    rocket::ignite()
        .manage(db::init_pool())
        .manage(hl)
        .manage(cache)
        .manage(x)
        .mount("/", routes![index, bfs_empty, bfs_sort, bfs_search, foo])
        .launch();

}


