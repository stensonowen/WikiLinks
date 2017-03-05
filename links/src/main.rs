//#![allow(dead_code)]
//#![feature(plugin, custom_derive, custom_attribute)]
#![feature(plugin)]
#![plugin(rocket_codegen)]

// NOTE: when scaling, remember to change bool link_db/parse/regexes.rs/IS_SIMPLE

#[macro_use]
extern crate clap;
extern crate links;
//use links::web;
use links::cache as db;

extern crate rocket;
extern crate rocket_contrib;
use rocket_contrib::Template;

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
fn index(conn: db::Conn) -> String {
    String::from("howdy whorl")
}


fn main() {
    rocket::ignite()
        .manage(db::init_pool())
        .mount("/", routes![index])
        .launch();

}
