#![feature(test)]

use std::io::{BufRead, BufReader};
use std::fs::File;

extern crate test;
extern crate regex;
#[macro_use] extern crate nom;

/* Process:
 *  0   run ./retrieve.sh to download/gunzip everything
 *  1   read through *page.sql to map every page_id to an article object
 *  2   read through *redirect.sql to mark redirects
 *  3   read through *pagelinks.sql to make note of every child link (in both directions?)
 *  4   output the entire thing into a format that `phc` likes
 */

#[cfg(test)]
mod tests {
}

fn parse_pagelinks() {
    let filename = "simplewiki-20161201-pagelinks.sql";
    println!("Opening `{}`...", filename);
    let f = File::open(filename).unwrap();
    let r = BufReader::new(f);
}

/*

fn parse_pagelinks_regex(r: BufReader<File>) {
    //regex: match mysql entry of the form (int,int,'string (don\'t forget escapes)',int)
    //  only first int (src page_id) and str (dst article title) are important to us
    //      the other integers are namespaces
    //  parses mysql dump directly; only preparation is downloading / gunzipping data
    // It seems this finds all entries that parse_pagelinks_mysql does. 
    //  However, this remains to be proven.
    // This finds every result that parse_pagelinks_mysql does on the simple wiki.
    //  However, that can't be tested on the full english wiki: the mysql parser
    //   requires days to load everything into memory (and requires tons of it),
    //   and this ran for ~60 hours before returning: ```thread 'main' panicked at 'called
    //      `Result::unwrap()` on an `Err` value: Error { repr: Custom(Custom { kind: 
    //      InvalidData, error: StringError("stream did not contain valid UTF-8") }) }',
    //      ../src/libcore/result.rs:788`, and returning error code 101
    //  Supposedly this is because of the subgroup matching. A python parser can go through
    //   the larger full English dump in ~30 minutes in 1 thread, so this shouldn't take so
    //   long. Maybe the long lines are also major caching problems? Anyway, the next step
    //   is probably to write a new Regex with Cursors or something. And maybe multithread it.
    extern crate regex;
    use regex::Regex;
    let re = Regex::new(r"\((\d)+,-?\d+,'([^'\\]*(?:\\.[^'\\]*)*)',-?\d+\)").unwrap();
    let mut count = 0;

    for line in r.lines() {
        let l = line.unwrap();
        let m = re.captures_iter(&l);
        for c in m {
            let dst = c.at(2).unwrap();
            let src: u32 = c.at(1).unwrap().parse().unwrap();
            println!("Found match: `{}`", c.at(0).unwrap());
            println!("	src page_id: \t`{}`", src);
            println!("  dst title: \t`{}`", dst);
            count += 1;
        }
    }
    println!("Count: {}", count);
}

fn parse_pagelinks_mysql(r: BufReader<File>) {
    //mysql: iterate through entries of mysql dump 
    //  (iterating is slightly faster this was than with regex but requires more setup)
    // must be preceded by:
    //      mysql> CREATE DATABASE en_pagelinks;
    //      $ mysql -p smp_pagelinks < simplewiki-20161201-pagelinks.sql
    //  which takes a long time for large-ish dumps
    //  I terminated the enwiki-*-pagelinks.sql loading after ~48 hours
    //      (it was maxing out IO not always using much CPU or memory)
    //      ((it was in a spruced up VM, but maybe the hard drive should have been 'fixed'?

    #[macro_use]
    extern crate mysql;
    use mysql::OptsBuilder;
    let mut builder = OptsBuilder::new();
    builder.user(Some("root")).pass(Some("yoursql")).db_name(Some("smp_pagelinks"));
    let pool = mysql::Pool::new(builder).unwrap();
    let mut count = 0;
    for i in pool.prep_exec(r"SELECT * FROM pagelinks", ()).unwrap() {
        let row = i.unwrap();
        let (from, _, title, _): (u32,u32,String,u32) = mysql::from_row(row);
        println!("`{}` \t->\t`{}`", from, title);
        count += 1;
    }
    println!("Count: {}", count);
}
*/

use std::borrow::Cow;

#[derive(Debug)]
struct Link<'a> {
    src: u32,
    dst: Cow<'a, str>,
}

pub fn str_to_u32(b: &[u8]) -> u32 {
    //only accepts [0-9] (no commas, no negatives)
    use std::str::from_utf8;
    let s = from_utf8(b).unwrap();
    u32::from_str_radix(s, 10).unwrap()
}

fn parse_pagelinks_nom(r: BufReader<File>) {
    use nom::{IResult, digit};

    named!(title, escaped!(is_not!("\\'"), '\\', is_a!("\"n\\")));
    named!(pos_num <&[u8], u32>, map!(recognize!(digit), str_to_u32));
    named!(link(&[u8]) -> Link, do_parse!(
            tag!("(")   >>
            recognize!(digit) >>
            tag!(",")   >>
            x: pos_num  >>
            tag!(",")   >>
            y: title    >>
            tag!(",")  >>
            recognize!(digit) >>
            tag!(")")   >>
            ( Link{ src:x, dst: String::from_utf8_lossy(y) } )
            ));

    for line in r.lines() {
        let l = line.unwrap();
    }
}


fn parse_pagelinks_regex_lossy(mut r: BufReader<File>) {
    //like parse_pagelinks_regex but can tolerate occasional utf-16 characters
    // which are extremely uncommon but not non-existant in the pagelinks dump
    //Also I think this should be faster
    //When built in release mode we find 1022340437 links from 38GB in 90 
    
    //our buffer size is ~20% greater than the longest line we've found
    let mut buffer = Vec::<u8>::with_capacity(1_250_000);
	
    //keep track of all the matching links we find
    let mut count = 0u64;
    let re = regex::Regex::new(r"\((\d)+,-?\d+,'([^'\\]*(?:\\.[^'\\]*)*)',-?\d+\)").unwrap();

    while r.read_until(b'\n', &mut buffer).unwrap() > 0 {
        {
            let s: Cow<str> = String::from_utf8_lossy(&buffer);
            let m = re.captures_iter(&s);
            for c in m {
                let dst: &str = c.at(2).unwrap();
                let src: u32  = c.at(1).unwrap().parse().unwrap();
                count += 1;
            }
        }
        buffer.clear();
    }
    println!("count: {}", count);
}

    
