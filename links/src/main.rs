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
use links::cache::get_cache;
use links::web::{self, Context, CacheSort, SortParam, PathRes, Node};
use links::cache::{self, lookup_addr};
type SharedLinks<'a> = State<'a, HashLinks>;

const DEFAULT_SORT: CacheSort = CacheSort::Recent;
const DEFAULT_SIZE: u32 = 15;

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
fn index(conn: db::Conn, links: SharedLinks) -> Template {
    let sort = DEFAULT_SORT;
    let cache = get_cache(&conn, links.get_links(), &sort, DEFAULT_SIZE);
    let context = Context::from_cache(sort, cache);
    Template::render("bfs", &context)
}

#[get("/bfs", rank = 3)]
fn bfs_empty(conn: db::Conn, links: SharedLinks) -> Template {
    let cache = get_cache(&conn, links.get_links(), &DEFAULT_SORT, DEFAULT_SIZE);
    let context = Context::from_cache(DEFAULT_SORT, cache);
    Template::render("bfs", &context)
}

#[get("/bfs?<sort>", rank = 2)]
fn bfs_sort(sort: SortParam, conn: db::Conn, links: SharedLinks) -> Template {
    let sort = match sort.by {
        Some(s) => CacheSort::from_str(s).unwrap_or(DEFAULT_SORT),
        None => DEFAULT_SORT,
    };
    let cache = get_cache(&conn, links.get_links(), &sort, DEFAULT_SIZE);
    let context = Context::from_cache(sort, cache);
    Template::render("bfs", &context)
}

#[get("/bfs?<search>", rank = 1)]
fn bfs_search(search: web::SearchParams, conn: db::Conn, 
                  links: SharedLinks) -> Template 
{
    let (src_f, dst_f) = search.fix();
    let src_n = if src_f.is_empty() {
        //TODO
        unimplemented!()
        //let (id, title) = links.random_id_and_title();
        //Node::Found(id, title)
    } else {
        lookup_addr(&*conn, src_f.as_ref())
    };
    let dst_n = if dst_f.is_empty() {
        //TODO
        unimplemented!()
        //let (id, title) = links.random_id_and_title();
        //Node::Found(id, title)
    } else {
        lookup_addr(&*conn, dst_f.as_ref())
    };
    let path_res = if let (&Node::Found(s,..), &Node::Found(d,..)) = (&src_n, &dst_n) {
        if let Some(db_path) = cache::lookup_path(&*conn, s, d) {
            // return the path that was saved last time
            PathRes::from_db_path(db_path, links.get_links())
        } else {
            // can't find record of previous search; perform for the first time
            let path = links.bfs(s,d);
            cache::insert_path(&conn, path.clone());    // TODO: kill the clone
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
    let cache = get_cache(&conn, links.get_links(), &sort, DEFAULT_SIZE);
    let context = Context {
        path:       path_res,
        src_search: src_n,
        dst_search: dst_n,
        cache:      cache,
        cache_sort: sort,
    };
    //println!("Context: {:?}", context);
    Template::render("bfs", &context)
}


fn main() {
    // get links hashmap
    // uhhhh, will .manage() do a bunch of memmoves?? sure hope not
    // use an Arc/Rc/Cell/ something?
    let hl_state = LinkState::<HashLinks>::from_args(argv());
    //let conn = cache::establish_connection();
    //cache::populate_addrs(&conn, hl_state.get_links(), hl_state.get_ranks()).unwrap();
    let hl = hl_state.extract();

    rocket::ignite()
        .manage(db::init_pool())
        .manage(hl)
        .mount("/", routes![index, bfs_empty, bfs_sort, bfs_search])
        .launch();

}


