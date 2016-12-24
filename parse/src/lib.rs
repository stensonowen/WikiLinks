#![feature(test)]

use std::io::{BufRead, BufReader};
use std::fs::File;
extern crate test;
extern crate regex;

#[macro_use] extern crate nom;

#[cfg(test)]
mod tests {
    use regex::Regex;
    use std::io::{BufRead, BufReader, Read};
    use std::fs::File;
    use test::Bencher;

    #[test]
    fn it_works() {
        // `wc -L`: 1,036,792 bytes after 8 minutes
        let filepath = "/home/qj/wikidata/enwiki-20161201-pagelinks.sql";
        let f = File::open(filepath).unwrap();
        let r = BufReader::new(f);
        for s in r.split(b',').take(5) {
            println!("\t`{}`", String::from_utf8(s.unwrap()).unwrap());
        }
    }

    #[bench]
    fn line_regex(b: &mut Bencher) {
        let filepath = "/home/owen/shared/code/rust/wikilinks/one_line";
        let mut f = File::open(filepath).unwrap();
        let re = Regex::new(r"\((\d)+,-?\d+,'([^'\\]*(?:\\.[^'\\]*)*)',-?\d+\)").unwrap();
        b.iter(|| {
            let mut s = String::new();
            f.read_to_string(&mut s);
            let mut count = 0;
            let m = re.captures_iter(&s);
            for c in m {
                let _dst = c.at(2).unwrap();
                let _src: u32 = c.at(1).unwrap().parse().unwrap();
                count += 1;
            }
            println!("Count: {}", count);
            //29465
        })
    }

    #[test]
    fn read_and_do_nothing() {
        // English  Wiki Pagelinks: 37k lines, 38G bytes, qj-deb-serv
        //line by line BR w/ printing every 1k: 82.5 minutes, 37468 lines
        //byte by byte BR w/ no printing: 182 minutes, 38,132,820,826 bytes
        //
        // Simple Wiki Pagelinks: 314 lines, 264M bytes, stenso-deb-laptop
        //
        //byte by byte BR: 70 seconds
        //
        //let filepath = "/home/qj/wikidata/enwiki-20161201-pagelinks.sql";
        let filepath = "/home/owen/shared/code/rust/wikilinks/simplewiki-20161201-pagelinks.sql";
        let f = File::open(filepath).unwrap();
        let r = BufReader::new(f);
        //let mut v = Vec<u8>::with_capacity(10_000);
        //for (i,_) in r.lines().enumerate() {
            //if i%1000 == 0 {
            //    println!("\t{}", i);
            //}
        let mut bytes = 0u64;
        for b in r.bytes() {
            bytes += 1;
        }
        println!("bytes: {}", bytes);
    }
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


