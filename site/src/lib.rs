#![feature(plugin, custom_derive, custom_attribute)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate serde_derive;
extern crate rocket;
extern crate rocket_contrib;

extern crate wikidata;
extern crate bfs;
extern crate phf;

use rocket_contrib::Template;
use rocket_contrib::JSON;
use rocket::response::Redirect;
use rocket::http::uri::URI; // URI::percent_decode

mod helpers;
use helpers::*;
pub mod database;

const LANGUAGE: &'static str = "simple";

// Intented site behavior
//
//  API
//      ( use text or page_id? )
//      wikilinks.xyz/bfs/src/dst           data is ????, returns JSON
//      wikilinks.xyz/bfs?src=src&dst=dst   data is text, renders a page
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
fn search_api(query: &str) -> JSON<SearchResult> {
    let decoded = URI::percent_decode_lossy(query.as_bytes());
    let fixed = bfs::preprocess(decoded.as_ref());
    if let Some(&id) = wikidata::ADDRESSES.get(fixed.as_ref()) {
        JSON(SearchResult::PageId(id))
    } else {
        let r = bfs::search(fixed.as_ref());
        if r.is_empty() {
            JSON(SearchResult::NoGuesses)
        } else {
            JSON(SearchResult::Recommendations(r))
        }
    }
}


#[get("/bfs", rank = 2)]
fn bfs_empty<'a>() -> Template {
    // any way to make this just a part of bfs_search?
    let context = Context {
        cache: None,
        path: None,
        src_search: None,
        dst_search: None,
    };
    Template::render("bfs", &context)
}

#[get("/bfs?<search>", rank = 1)]
fn bfs_search<'a>(search: Search<'a>) -> Template {
    let (src_search, dst_search) = resolve_titles(&search);
    let context = Context {
        cache: Some(search.src),
        path: Some("just plain `bfs`"),
        src_search: src_search,
        dst_search: dst_search,
    };
    Template::render("bfs", &context)
}


pub fn deploy() {
    bfs::load_titles(); //just for testing??
    rocket::ignite()
        .mount("/",
               routes![index,
                           bfs_search,
                           bfs_empty,
                           bfs_api,
                           search_api,
    ])
        .launch();
}
