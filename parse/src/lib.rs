use std::io::{BufRead, BufReader};
use std::fs::File;
use std::borrow::Cow;

extern crate regex;
#[macro_use]
extern crate slog;

mod regexes;
mod database;
use database::*;

// Parsing Note:
//  Most of the [u8] -> &str conversions involve potential errors in which the source
//  might not be valid u8. This is not common but the case must be handled.
//  I'm not sure how utf-16 works with rust regexes, and it might mean lots more memory.
//  We handle this with String::from_utf8_lossy(), which replaces bad utf8 with '�'
//  It also returns a Cow, which suits our purposes well.
//

// Used to be: addresses = entries = 172,350    / 408,784

// bytes in the buffer for reading one line at a time
// problems may arise if this buffer fills up all the way: some data will not be read
// to be safe, it is ~20% larger than the longest line in a dump ( 1,025,987 - 1,039,069 )
const BUFFER_SIZE: usize = 1_250_000;


pub fn populate_db(page_sql:   &'static str,    //will this be problematic?
                   redirs_sql: &'static str, 
                   links_sql:  &'static str, 
                   log: &slog::Logger) -> Database {

    let db_log = log.new(o!("page.sql"      => page_sql,
                            "redirect.sql"  => redirs_sql,
                            "pagelinks.sql" => links_sql));

    let mut db = Database::new(db_log);
    let pages = parse_generic(&page_sql,
                              &regexes::pages_regex(),
                              &mut db,
                              |db: &mut Database, data: regex::Captures| {
                                  db.add_page(&data)
                              });
    info!(log, "Number of page entries: {} / {}", pages.0, pages.1);
    let redirs = parse_generic(&redirs_sql,
                              &regexes::redirect_regex(),
                              &mut db,
                              |db: &mut Database, data: regex::Captures| {
                                  db.add_redirect(&data)
                              });
    info!(log, "Number of redirects: {} / {}", redirs.0, redirs.1);
    let links = parse_generic(&links_sql,
                              &regexes::pagelinks_regex(),
                              &mut db,
                              |db: &mut Database, data: regex::Captures| {
                                  db.add_pagelink(&data)
                              });
    info!(log, "Number of pagelinks: {} / {}", links.0, links.1);
    db.finalize();
    db
}

pub fn parse_generic<F>(filename: &str, re: &str, db: &mut Database, action: F) -> (u64,u64)
    where F: Fn(&mut Database, regex::Captures) -> bool 
{
    // parse a mysql dump from a custom regex
    // use a closure to define how the database uses the results

    let f = File::open(filename).unwrap();
    let mut reader = BufReader::new(f);
    let re = regex::Regex::new(re).unwrap();
    let mut buffer = Vec::<u8>::with_capacity(BUFFER_SIZE);
    let mut success = 0u64;
    let mut attempts = 0u64;

    loop {
        let len = reader.read_until(b'\n', &mut buffer).unwrap();
        if len == 0 {
            // done (empty line has a length of 1)
            break;
        } else if len == BUFFER_SIZE {
            // crash/complain w/ error message
            panic!(format!("Input db line was longer than your buffer size; increase \
                            BUFFER_SIZE to at least {}.",
                           len));
        } else {
            let s: Cow<str> = String::from_utf8_lossy(&buffer);
            for entry in re.captures_iter(&s) {
                if action(db, entry) {
                    success += 1;
                }
                attempts += 1;
            }
        }
        buffer.clear();
    }
    (success,attempts)
}
