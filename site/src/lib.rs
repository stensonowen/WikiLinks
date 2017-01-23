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
use helpers::DEFAULT_CACHE_SORT;

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


#[get("/bfs_api?<api>")]
fn api_bfs(db: DB, api: BfsApiParams) -> JSON<BfsApiResult> {
    let src = lookup_id_or_title(&db, api.src_id, api.src_title);
    let dst = lookup_id_or_title(&db, api.dst_id, api.dst_title);
    let ret = match (src, dst) {
        (Err(_), Err(_)) => BfsApiResult::InvalidSrcAndDst,
        (Err(_), _) => BfsApiResult::InvalidSrc,
        (_, Err(_)) => BfsApiResult::InvalidDst,
        (Ok(src_id), Ok(dst_id)) => {
            let path = match database::get_path(db.conn(), src_id, dst_id) {
                //TODO: make these the same type?
                Ok(database::PathLookup::Path(p)) => Ok(p),
                Ok(database::PathLookup::Terminated(i)) => Err(bfs::Error::Terminated(i)),
                Ok(database::PathLookup::NoSuchPath) => Err(bfs::Error::NoSuchPath),
                _ => {
                    //perform the search if we never have, and save it in the db
                    let path = bfs::bfs(src_id, dst_id);
                    database::insert_path(db.conn(), src_id, dst_id, &path).unwrap();
                    path
                }
            };
            match path {
                Ok(v) => BfsApiResult::Success {
                    titles: v.iter()
                        .map(|id| wikidata::ENTRIES.get(id).unwrap().title)
                        .collect(),
                    ids: v,
                },
                Err(bfs::Error::Terminated(i)) => BfsApiResult::TerminatedAfter(i),
                Err(bfs::Error::NoSuchPath) => BfsApiResult::NoSuchPath,
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

#[get("/bfs", rank = 3)]
fn bfs_empty<'a>(db: DB) -> Template {
    // any way to make this just a part of bfs_search?
    // using Option-al fields in Search still requires a `?`
    // this also catches when only one of src/dst is specified
    let mut context = Context::blank();
    context.bad_src = false;
    context.bad_dst = false;
    context.cache = database::get_cache(db.conn(), 
                                        DEFAULT_CACHE_SORT,
                                        CACHE_SIZE).ok();
    context.cache_sort = DEFAULT_CACHE_SORT;
    Template::render("bfs", &context)
}

#[get("/bfs?<sort>", rank = 2)]
fn bfs_sort<'a>(db: DB, sort: CacheSortParam) -> Template {
    //request certain cache sort without making a search
    let mut context = Context::blank();
    context.bad_src = false;
    context.bad_dst = false;
    let sort_method: SortOptions = sort.cache_sort
        .and_then(SortOptions::convert)
        .unwrap_or(DEFAULT_CACHE_SORT);
    context.cache_sort = sort_method;
    context.cache = database::get_cache(db.conn(), 
                                        sort_method,
                                        CACHE_SIZE).ok();
    context.cache_sort = sort_method;
    Template::render("bfs", &context)

}

#[get("/bfs?<search>", rank = 1)]
fn bfs_search<'a>(search: SearchParams<'a>, db: DB) -> Template {
    //let src_query = database::lookup_addr(db.conn(), preprocess(search.src));
    let mut context = Context::blank();
    //pre-process, check title validity
    let (src_fix, dst_fix) = search.prep();
    let src_lookup = database::lookup_addr(db.conn(), src_fix.as_ref());
    let dst_lookup = database::lookup_addr(db.conn(), dst_fix.as_ref());
    if let (Ok(src_query), Ok(dst_query)) = (src_lookup, dst_lookup) {
        //lookups didn't fail, but might return no result
        //set src|dst titles even if they're bad/guesses
        context.src_t = Some(src_fix.into_owned());
        context.dst_t = Some(dst_fix.into_owned());
        use database::AddressLookup::Address;
        if let (&Address(src_id), &Address(dst_id)) = (&src_query, &dst_query) {
            //well-formed request
            context.bad_src = false;
            context.bad_dst = false;
            //try to get this from the database
            let path = match database::get_path(db.conn(), src_id, dst_id) {
                Ok(database::PathLookup::Path(p)) => Ok(p),
                Ok(database::PathLookup::Terminated(i)) => Err(bfs::Error::Terminated(i)),
                Ok(database::PathLookup::NoSuchPath) => Err(bfs::Error::NoSuchPath),
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
        //use std::str::FromStr;
        //get cache after action, so it can reflect our search
        let sort_method: SortOptions = search.cache_sort
            .and_then(SortOptions::convert)
            .unwrap_or(DEFAULT_CACHE_SORT);
        context.cache = database::get_cache(db.conn(), 
                                            //SortOptions::Recent, 
                                            //SortOptions::Popular, 
                                            //SortOptions::Length, 
                                            sort_method,
                                            CACHE_SIZE).ok();
        context.cache_sort = sort_method;
    }
    Template::render("bfs", &context)
}


pub fn deploy() {
    rocket::ignite()
        .mount("/",
               routes![index,
                           bfs_search,
                           bfs_empty,
                           //bfs_api,
                           //search_api,
                           //test
                           api_bfs,
                           bfs_sort,
    ])
        .launch();
}
