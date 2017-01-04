#![allow(dead_code)]

extern crate phf;

include!("codegen_links.rs");
include!("codegen_entries.rs");
include!("codegen_addresses.rs");

pub struct Page {
    pub title:      &'static str,
    pub children:   &'static [u32],
    pub parents:    &'static [u32],
}

