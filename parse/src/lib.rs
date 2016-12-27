use std::io::{BufRead, BufReader};
use std::fs::File;
use std::borrow::Cow;

extern crate regex;
mod regexes;
mod database;
use database::*;

/* Parsing Note:
 *  Most of the [u8] -> &str conversions involve potential errors in which the source
 *  might not be valid u8. This is not common but the case must be handled.
 *  I'm not sure how utf-16 works with rust regexes, and it might mean lots more memory.
 *  We handle this with String::from_utf8_lossy(), which replaces bad utf8 with '�'
 *  It also returns a Cow, which suits our purposes well.
 */

/* Process:
 *  0   run ./retrieve.sh to download/gunzip everything
 *  1   read through *page.sql to map every page_id to an article object (and back)
 *  2   read through *redirect.sql to mark redirects
 *  3   read through *pagelinks.sql to make note of every child link (in both directions?)
 *  4   output the entire thing into a format that `phc` likes
 */

//bytes in the buffer for reading one line at a time
//problems may arise if this buffer fills up all the way: some data will not be read
//to be safe, it is ~20% larger than the longest line in a dump ( 1,025,987 - 1,039,069 )
const BUFFER_SIZE: usize = 1_250_000;


pub fn parse() -> Database {
    let dir = String::from("/home/owen/shared/code/rust/wikilinks/data/");
    //let redir_f = dir.clone() + "simplewiki-20161201-redirect.sql";
    let pages_f = dir.clone() + "simplewiki-20161201-page.sql";
    //let links_f = dir + "simplewiki-20161201-pagelinks.sql";

    let mut db = Database::new();
    //let redirects = parse_generic(&redir_f, &regexes::redirect_regex(), &mut db, 
    //                              |db: &mut Database, data: regex::Captures| { 
    //                                  db.add_redirect(&data); 
    //                              });
    //println!("Number of redirects: {}", redirects);
    let pages = parse_generic(&pages_f, &regexes::pages_regex(), &mut db, 
                              |db: &mut Database, data: regex::Captures| { 
                                  db.add_page(&data); 
                              });
    println!("Number of page entries: {}", pages);
    //let links = parse_generic(&links_f, &regexes::pagelinks_regex(), &mut db, 
    //                          |db: &mut Database, data: regex::Captures| { 
    //                              db.add_pagelink(&data); 
    //                          });
    //println!("Number of pagelinks: {}", links);
    println!("Number of elements: {}", db.len());
    db
}

pub fn parse_generic<F>(filename: &str, re: &str, db: &mut Database, action: F) -> u64
    where F: Fn(&mut Database, regex::Captures) -> ()
{
    //parse a mysql dump from a custom regex
    //use a closure to define how the database uses the results
    
    let f = File::open(filename).unwrap();
    let mut reader = BufReader::new(f);
    let re = regex::Regex::new(re).unwrap();
    let mut buffer = Vec::<u8>::with_capacity(BUFFER_SIZE);
    let mut count = 0u64;

    loop {
        let len = reader.read_until(b'\n', &mut buffer).unwrap(); 
        if len == 0 {
            //done (empty line has a length of 1)
            break;
        } else if len == BUFFER_SIZE {
            //crash/complain w/ error message
            panic!(format!("Input db line was longer than your buffer size; increase BUFFER_SIZE to at least {}.", len));
        } else {
            let s: Cow<str> = String::from_utf8_lossy(&buffer);
            for entry in re.captures_iter(&s) {
                action(db, entry);
                count += 1;
            }
        }
        buffer.clear();
    }
    count
}


