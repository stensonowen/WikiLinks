extern crate wikidata;
extern crate bfs;
//use wikidata::ENTRIES;

fn main() {
    let a = wikidata::ADDRESSES.get("Rust").unwrap();
    let b = wikidata::ADDRESSES.get("Alan_Turing").unwrap();

    let r = bfs::bfs(*a,*b);
    bfs::print_path(r);
}
