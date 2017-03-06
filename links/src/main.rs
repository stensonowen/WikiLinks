#![feature(plugin)]
#![plugin(rocket_codegen)]

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
//type SharedLinks<'a> = State<'a, LinkState<HashLinks>>;
type SharedLinks<'a> = State<'a, HashLinks>;

const DEFAULT_SORT: CacheSort = CacheSort::Recent;
const DEFAULT_SIZE: u32 = 15;

use clap::Arg;
fn argv<'a>() -> clap::ArgMatches<'a> {
    clap::App::new(crate_name!()).about(crate_description!())
        .author(crate_authors!("\n")).version(crate_version!())
        .arg(Arg::with_name("ranks").short("r").long("ranks_dump").takes_value(true)
             .help("Supply location of rank data in csv form"))
        .arg(Arg::with_name("manifest").short("json").takes_value(true)
             .help("Supply dump of parsed link data manifest in json form"))
        .arg(Arg::with_name("page.sql").short("pages").takes_value(true)
             .conflicts_with("manifest").requires("redirect.sql").requires("pagelinks.sql"))
        .arg(Arg::with_name("redirect.sql").short("dirs").takes_value(true)
             .conflicts_with("manifest").requires("page.sql").requires("pagelinks.sql"))
        .arg(Arg::with_name("pagelinks.sql").short("links").takes_value(true)
             .conflicts_with("manifest").requires("page.sql").requires("redirect.sql"))
        .group(clap::ArgGroup::with_name("sources").required(true)
               .args(&["sql", "manifest", "page.sql", "redirects.sql", "pagelink.sql"]))
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
fn bfs_empty<'a>(conn: db::Conn, links: SharedLinks) -> Template {
    let cache = get_cache(&conn, links.get_links(), &DEFAULT_SORT, DEFAULT_SIZE);
    let context = Context::from_cache(DEFAULT_SORT, cache);
    Template::render("bfs", &context)
}

#[get("/bfs?<sort>", rank = 2)]
fn bfs_sort<'a>(sort: SortParam, conn: db::Conn, links: SharedLinks) -> Template {
    let sort = match sort.by {
        Some(s) => CacheSort::from_str(s).unwrap_or(DEFAULT_SORT),
        None => DEFAULT_SORT,
    };
    let cache = get_cache(&conn, links.get_links(), &sort, DEFAULT_SIZE);
    let context = Context::from_cache(sort, cache);
    Template::render("bfs", &context)
}

#[get("/bfs?<search>", rank = 1)]
fn bfs_search<'a>(search: web::SearchParams<'a>, conn: db::Conn, 
                  links: SharedLinks) -> Template 
{
    let (src_f, dst_f) = search.fix();
    let src_n = match src_f.is_empty() {
        false => lookup_addr(&*conn, src_f.as_ref()),
        true  => {
            let (id, title) = links.random_id_and_title();
            Node::Found(id, title)
        }
    };
    let dst_n = match dst_f.is_empty() {
        false => lookup_addr(&*conn, dst_f.as_ref()),
        true  => {
            let (id, title) = links.random_id_and_title();
            Node::Found(id, title)
        }
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
    println!("Context: {:?}", context);
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


