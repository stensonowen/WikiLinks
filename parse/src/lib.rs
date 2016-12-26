#![feature(test)]

use std::io::{BufRead, BufReader};
use std::fs::File;
use std::borrow::Cow;

extern crate regex;

/* Process:
 *  0   run ./retrieve.sh to download/gunzip everything
 *  1   read through *page.sql to map every page_id to an article object
 *  2   read through *redirect.sql to mark redirects
 *  3   read through *pagelinks.sql to make note of every child link (in both directions?)
 *  4   output the entire thing into a format that `phc` likes
 */

fn parse_pagelinks() {
    let filename = "simplewiki-20161201-pagelinks.sql";
    println!("Opening `{}`...", filename);
    let f = File::open(filename).unwrap();
    let r = BufReader::new(f);
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

fn parse_pages_regex_lossy(mut r: BufReader<File>, is_simple: bool) {
    //successfully locates all 408784 entries in the simple and 40966811 in the english
    //      simple  = 408739 + 45 = 408784
    //      english = 40962071 + 4740 = 40966811 == 40966811

	// we make a few assumptions here; matches everything in the english page.sql dump
    let page_id     = r"(\d+)";     //captured, positive non-null number
    let page_nmsp   = r"(\d+)";     //captured; should never be negative (0-15 ∪ 1000-2**31)
    let page_title  = r"'([^'\\]*(?:\\.[^'\\]*)*)'"; //surrounded by `'`s, which can be escaped
    let page_restrs = r"'.*?'"; 	//not always empty, but never has escaped quotes
    let page_counter= r"\d+";       //non-captured positive number; count will be wrong 
    let page_is_redr= r"(0|1)";     // 0 or 1, to indicate binary value; capture? usefulness?
    let page_is_new = r"(?:0|1)";   // 0 or 1; info is useless to us
    let page_random = r"[\d\.]+";   //random(?) unsigned double
    let page_tchd   = r"'\d+'";     //timestamp; irrelevant; cannot contain `\'`s (?)
    let page_ln_upd = r"(?:'\d+'|NULL)"; //another irrelevant timestamp, but it can be null
    let page_latest = r"\d+";       //unsigned int: latest revision number
    let page_len    = r"\d+";       //unsigned int: page len (or weird value)
    let page_ntc    = r"(?:0|1)";   //not sure what it is. should be commented in the English wiki
    let page_cont_md= r"(?:'.*?'|NULL)"; //probably never contains escaped quotes?
    let page_lang   = r"NULL";  	//think it'll always be null?
    
    let mut re_body = vec![page_id, page_nmsp, page_title, 
        page_restrs, page_counter,  page_is_redr, 
        page_is_new, page_random,   page_tchd, 
        page_ln_upd, page_latest,   page_len, 
       /*page_ntc,*/ page_cont_md,  page_lang
    ];
    if is_simple {
        re_body.insert(12, page_ntc);
    }

    let re_query = format!(r"\({}\)", re_body.join(","));
    let re = regex::Regex::new(&re_query).unwrap();

    let mut buffer = Vec::<u8>::with_capacity(1_250_000);
	
    //keep track of all the matching links we find
    let mut count = 0u64;

    while r.read_until(b'\n', &mut buffer).unwrap() > 0 {
        {
            let s: Cow<str> = String::from_utf8_lossy(&buffer);
            let m = re.captures_iter(&s);
            for c in m {
                count += 1;
            }
        }
        buffer.clear();
    }
    println!("count: {}", count);

}
