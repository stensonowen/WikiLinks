#![feature(plugin, custom_derive, custom_attribute)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;

extern crate rocket;
extern crate rocket_contrib;
extern crate r2d2;
extern crate r2d2_diesel;

extern crate wikidata;
extern crate bfs;
extern crate phf;

use rocket_contrib::Template;
use rocket_contrib::JSON;
use rocket::response::Redirect;

mod helpers;
use helpers::*;
pub mod database;

use database::DB;

const LANGUAGE: &'static str = "simple";
const CACHE_SIZE: i64 = 15; //ew, must be signed
const DEFAULT_CACHE_SORT: database::SortOptions = database::SortOptions::Recent;

// Intented site behavior
//
//  API
//      ( use text or page_id? )
//      wikilinks.xyz/bfs/src/dst           data is ????, returns JSON
//      wikilinks.xyz/bfs?src=src&dst=dst   data is text, renders a page
//      searchability? api to fuzzy search article titles?
//      cache, once that works
//
//  Todo
//      make a favicon.ico
//      figure out a domain? renew wikilinks.xyz?
//      design pages? learn some js (puke)?
//
//

#[get("/")]
fn index() -> Redirect {
    Redirect::to("/bfs")
}

/*
#[get("/bfs/api/<src>/<dst>", format = "application/json")]
fn bfs_api(src: u32, dst: u32) -> JSON<PathResult> {
    let r = match (wikidata::ENTRIES.get(&src), wikidata::ENTRIES.get(&dst)) {
        (None, None) => 
            PathResult::Error(String::from("Both src and dst page_ids were invalid")),
        (None, _) => 
            PathResult::Error(String::from("Invalid source page_id")),
        (_, None) => 
            PathResult::Error(String::from("Invalid destination page_id")),
        (Some(s), Some(d)) => { 
            match bfs::bfs(src,dst) {
                Ok(v) => PathResult::Path {
                    lang:   LANGUAGE,
                    src:    s.title,
                    dst:    d.title,
                    len:    v.len()-1,
                    nodes:  v
                },
                Err(bfs::Error::Terminated(x)) => 
                    PathResult::Error(format!("Failed after {} rounds", x)),
                Err(bfs::Error::NoSuchPath) => 
                    PathResult::Error(String::from("No Such Path")),
            }
        }
    };
    JSON(r)
}

#[get("/bfs/api/search?<query>")]
fn search_api(query: &str, db: DB) -> JSON<database::AddressLookup> {
    //let decoded = URI::percent_decode_lossy(query.as_bytes());
    //let fixed = bfs::preprocess(decoded.as_ref());
    let query_ = preprocess(query);
    JSON(database::lookup_addr(db.conn(), query_.as_ref()).unwrap())
}
*/

#[derive(Serialize)]
enum BfsApiRet {
    Success {
        ids: Vec<u32>,
        titles: Vec<&'static str>,
    },
    InvalidSrc,
    InvalidDst,
    InvalidSrcAndDst,
    TerminatedAfter(usize),
    NoSuchPath,
}

#[get("/bfs_api?<api>")]
fn api_bfs(db: DB, api: BfsApi) -> JSON<BfsApiRet> {
    let src = lookup_id_or_title(&db, api.src_id, api.src_title);
    let dst = lookup_id_or_title(&db, api.dst_id, api.dst_title);
    let ret = match (src, dst) {
        (Err(_), Err(_)) => BfsApiRet::InvalidSrcAndDst,
        (Err(_), _) => BfsApiRet::InvalidSrc,
        (_, Err(_)) => BfsApiRet::InvalidDst,
        (Ok(src_id), Ok(dst_id)) => {
            let path = match database::get_path(db.conn(), src_id, dst_id) {
                //TODO: make these the same type?
                Ok(database::PathOption::Path(p)) => Ok(p),
                Ok(database::PathOption::Terminated(i)) => Err(bfs::Error::Terminated(i)),
                Ok(database::PathOption::NoSuchPath) => Err(bfs::Error::NoSuchPath),
                _ => {
                    //perform the search if we never have, and save it in the db
                    let path = bfs::bfs(src_id, dst_id);
                    database::insert_path(db.conn(), src_id, dst_id, &path).unwrap();
                    path
                }
            };
            match path {
                Ok(v) => BfsApiRet::Success {
                    titles: v.iter()
                        .map(|id| wikidata::ENTRIES.get(id).unwrap().title)
                        .collect(),
                    ids: v,
                },
                Err(bfs::Error::Terminated(i)) => BfsApiRet::TerminatedAfter(i),
                Err(bfs::Error::NoSuchPath) => BfsApiRet::NoSuchPath,
            }
        }
    };
    JSON(ret)
}

fn lookup_id_or_title(db: &DB, id: Option<u32>, title: Option<&str>) -> Result<u32,()> {
    //returns the address if it's valid, or an error otherwise
    //should this just be an Option??
    match (id, title) {
        (Some(_), Some(_)) => Err(()),
        (None, None) => Err(()),
        (Some(id), None) => {
            //make sure this address is valid? is this necessarey?
            match wikidata::ENTRIES.contains_key(&id) {
                true  => Ok(id),
                false => Err(()),
            }
        },
        (None, Some(t)) => {
            let fix = preprocess(t);
            match database::lookup_addr(db.conn(), fix.as_ref()) {
                Ok(database::AddressLookup::Address(id)) => Ok(id),
                _ => Err(()),
            }
        }
    }
}

#[get("/bfs", rank = 2)]
fn bfs_empty<'a>() -> Template {
    // any way to make this just a part of bfs_search?
    // using Option-al fields in Search still requires a `?`
    // this also catches when only one of src/dst is specified
    let mut context = Context::blank();
    context.bad_src = false;
    context.bad_dst = false;
    Template::render("bfs", &context)
}

#[get("/bfs?<search>", rank = 1)]
fn bfs_search<'a>(search: Search<'a>, db: DB) -> Template {
    //let src_query = database::lookup_addr(db.conn(), preprocess(search.src));
    let mut context = Context::blank();
    //pre-process, check title validity
    let (src_fix, dst_fix) = search.prep();
    let src_lookup = database::lookup_addr(db.conn(), src_fix.as_ref());
    let dst_lookup = database::lookup_addr(db.conn(), dst_fix.as_ref());
    if let (Ok(src_query), Ok(dst_query)) = (src_lookup, dst_lookup) {
        //lookups didn't fail, but might return no result
        //set src|dst titles even if they're bad/guesses
        println!("\tSRC = `{:?}`;\t\tDST = `{:?}`", src_fix, dst_fix);
        context.src_t = Some(src_fix.into_owned());
        context.dst_t = Some(dst_fix.into_owned());
        use database::AddressLookup::Address;
        if let (&Address(src_id), &Address(dst_id)) = (&src_query, &dst_query) {
            //well-formed request
            context.bad_src = false;
            context.bad_dst = false;
            //try to get this from the database
            let path = match database::get_path(db.conn(), src_id, dst_id) {
                Ok(database::PathOption::Path(p)) => Ok(p),
                Ok(database::PathOption::Terminated(i)) => Err(bfs::Error::Terminated(i)),
                Ok(database::PathOption::NoSuchPath) => Err(bfs::Error::NoSuchPath),
                _ => {
                    //perform the search if we never have, and save it in the db
                    let path = bfs::bfs(src_id, dst_id);
                    database::insert_path(db.conn(), src_id, dst_id, &path).unwrap();
                    path
                }
            };
            match path {
                Ok(p) => {
                    context.path = Some(bfs::annotate_path(p, LANGUAGE)); },
                Err(bfs::Error::Terminated(n)) => {
                    context.path_err = Some(format!("Found no path after {} iterations",n)) },
                Err(bfs::Error::NoSuchPath) => {
                    context.path_err = Some("No such path exists".to_owned()) },
            }
        } else {
            // src or dst title was invalid; update context with suggestions/indicator
            if let database::AddressLookup::Suggestions(v) = src_query {
                context.src_alts = Some(v);
            } else {
                context.bad_src = false;
            }
            if let database::AddressLookup::Suggestions(v) = dst_query {
                context.dst_alts = Some(v);
            } else {
                context.bad_dst = false;
            }
        }
        //get cache after action, so it can reflect our search
        context.cache = database::get_cache(db.conn(), 
                                            //database::SortOptions::Recent, 
                                            //database::SortOptions::Popular, 
                                            //database::SortOptions::Length, 
                                            search.sort_option().unwrap_or(DEFAULT_CACHE_SORT),
                                            CACHE_SIZE).ok();
    }
    Template::render("bfs", &context)
}


//#[derive(FromForm, Queryable, Debug)]
//struct Test {
//    a: String,
//    b: Option<String>,
//}

//#[get("/foo?<t>")]
//fn test(t: Test) -> String {
//    format!("`{:?}`", t)
//}

pub fn deploy() {
    //bfs::load_titles(); //just for testing??
    rocket::ignite()
        .mount("/",
               routes![index,
                           bfs_search,
                           bfs_empty,
                           //bfs_api,
                           //search_api,
                           //test
                           api_bfs,
    ])
        .launch();
}
