//#![feature(getpid)]
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

// COMPONENTS
pub mod link_state;

const IS_SIMPLE: bool = true;

