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
//use rocket::http::uri::URI; // URI::percent_decode

mod helpers;
use helpers::*;
pub mod database;

use database::DB;

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
fn search_api(query: &str, db: DB) -> JSON<database::AddressLookup> {
    //let decoded = URI::percent_decode_lossy(query.as_bytes());
    //let fixed = bfs::preprocess(decoded.as_ref());
    let query_ = preprocess(query);
    JSON(database::lookup_addr(db.conn(), query_.as_ref()).unwrap())
}


#[get("/bfs", rank = 2)]
fn bfs_empty<'a>() -> Template {
    // any way to make this just a part of bfs_search?
    let context = Context {
        cache: None,
        path: None,
        src_err: None,
        dst_err: None,
        path_err: None,
    };
    Template::render("bfs", &context)
}

#[get("/bfs?<search>", rank = 1)]
fn bfs_search<'a>(search: Search<'a>, db: DB) -> Template {
    //let src_query = database::lookup_addr(db.conn(), preprocess(search.src));
    let mut context = Context::blank();
    let src_fix = search.prep_src();
    let src_lookup = database::lookup_addr(db.conn(), src_fix.as_ref());
    let dst_fix = search.prep_dst();
    let dst_lookup = database::lookup_addr(db.conn(), dst_fix.as_ref());
    if let (Ok(src_query), Ok(dst_query)) = (src_lookup, dst_lookup) {
        //lookups didn't fail, but might return no result
        context.src_err = src_query.to_html(src_fix.as_ref());
        context.dst_err = dst_query.to_html(dst_fix.as_ref());
        //TODO: populate Cache template
        use database::AddressLookup::Address;
        if let (Address(src_id), Address(dst_id)) = (src_query, dst_query) {
            //bfs from src_id to dst_id
            //and insert into table
            let path = {
                if let Ok(p) = database::get_path(db.conn(), src_id, dst_id) {
                    Ok(p)
                } else {
                    let path = bfs::bfs(src_id, dst_id);
                    database::insert_path(db.conn(), src_id, dst_id, &path).unwrap();
                    path
                }
            };
            //if let Ok(path) = database::get_path(db.conn(), src_id, dst_id) {
            //    match path {
            //        Ok(p) => {
            //            context.path = Some(bfs::annotate_path(p, LANGUAGE));
            //        },
            //    }
            //} else {
            //    let path = bfs::bfs(src_id, dst_id);
            //    database::insert_path(db.conn(), src_id, dst_id, &path).unwrap();
            match path {
                Ok(p) => {
                    context.path = Some(bfs::annotate_path(p, LANGUAGE));
                },
                Err(bfs::Error::Terminated(n)) => {
                    context.path_err = Some(format!("Found no path after {} iterations",n))
                },
                Err(bfs::Error::NoSuchPath) => {
                    context.path_err = Some("No such path exists".to_owned())
                },
            }
            //}
            //if let Ok(p) = path {
            //    context.path = bfs::annotate_path(p);
            //} else {
            //    context.path_err = 

            //}

            //let path_ = path.unwrap().iter().map(|n| (n.to_string(), 0)).collect();
            //let path_res = bfs::format_path(path, LANGUAGE);
            //context.path = Some(path_res);
            //context.path = Some(path_); //path.ok();
        }
    }
    Template::render("bfs", &context)
}


pub fn deploy() {
    //bfs::load_titles(); //just for testing??
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
