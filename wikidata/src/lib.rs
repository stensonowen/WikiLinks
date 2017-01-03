#![allow(dead_code)]

extern crate phf;
include!("codegen_links.rs");

//pub mod codegen_links;
//use codegen_links::*;
//pub mod codegen_entries;
//use codegen_entries::*;
//pub mod codegen_addresses;
//use codegen::addresses::*;

//include!("codegen_links.rs");

#[derive(Clone)]
struct Page {
    title:      &'static str,
    children:   &'static [u32],
    parents:    &'static [u32],
}

include!("codegen_entries.rs");
include!("codegen_addresses.rs");

//fn parse_keyword(keyword: u32) -> Option<Page> {
//    KEYWORDS.get(&keyword).cloned()
//}


fn main() {
    //println!("hello world");
    println!("Length of entries: {}", ENTRIES.len());
    println!("Length of addresses: {}", ADDRESSES.len());

    if let Some(x) = ADDRESSES.get("Rust") {
        println!("addrs['Rust'] = {}", x);
        if let Some(&Page { title: ref t, children: ref c, parents: ref p }) = ENTRIES.get(&x) {
            println!("entries[{}]:", x);
            println!("\ttitle:\t`{}`", t);
            println!("\tchildren:\t`{:?}`", c);
            println!("\tparents:\t`{:?}`", p);
        } else {
            println!("Couldn't find {}", x);
        }
    } else {
        println!("Couldn't find `Rust`");
    }

}

