#![allow(dead_code)]
#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

// NOTE: when scaling, remember to change bool link_db/parse/regexes.rs/IS_SIMPLE

// LOGGING
#[macro_use] extern crate slog;
extern crate slog_term;
// SERIALIZING
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate csv;
// MISC
extern crate clap;
extern crate fnv;
extern crate chrono;
//extern crate test;
// DATABASE
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate dotenv;

extern crate rocket;

// COMPONENTS
pub mod link_state;
pub mod cache;
pub mod web;

//mod test;

const WIKI_URL_FMT: &'static str = "https://simple.wikipedia.org/?curid=";
const IS_SIMPLE: bool = true;

