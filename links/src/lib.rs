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
// DATABASE
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate dotenv;

extern crate rocket;    //move this lower?
extern crate rocket_contrib;

// COMPONENTS
pub mod link_state;
pub mod cache;
pub mod web;


/*
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;

pub struct SharedState {
    conn: r2d2::PooledConnection<ConnectionManager<PgConnection>>,
    // should this /just/ be the 'links' hashmap? or the whole state?
    hash_links: link_state::LinkState<link_state::HashLinks>,
}
*/
