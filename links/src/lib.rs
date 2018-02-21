//#![feature(getpid)]
#![allow(unknown_lints, bool_comparison)]

// NOTE: when running on the simple wiki, parsing is slightly different
// run with `cargo build --features=simple`

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
extern crate fst;
extern crate chrono;

// COMPONENTS
pub mod link_state;


