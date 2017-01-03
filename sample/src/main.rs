extern crate wikidata;
//use wikidata::ENTRIES;

fn main() {
	println!("Length of entries: {}", wikidata::ENTRIES.len());
	println!("Length of addresses: {}", wikidata::ADDRESSES.len());

	if let Some(x) = wikidata::ADDRESSES.get("Rust") {
		println!("addrs['Rust'] = {}", x);
		if let Some(&wikidata::Page { title: ref t, children: ref c, parents: ref p }) 
				= wikidata::ENTRIES.get(&x) {
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
