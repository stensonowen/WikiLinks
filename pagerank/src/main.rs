extern crate wikidata;
//use wikidata::ENTRIES;

fn main() {
	println!("Length of entries: {}", wikidata::ENTRIES.len());
	println!("Length of addresses: {}", wikidata::ADDRESSES.len());

	if let Some(x) = wikidata::ADDRESSES.get("Rust") {
		println!("addrs['Rust'] = {}", x);
		if let Some(&wikidata::Page { title: t, children: c, parents: p }) 
				= wikidata::ENTRIES.get(&x) {
			println!("entries[{}]:", x);
			println!("\ttitle:\t`{}`", t);
            println!("\tchildren:");
            for i in c {
                if let Some(&wikidata::Page { title: ref y, .. }) = wikidata::ENTRIES.get(i) {
                    println!("\t\t`{}`", y);
                } else {
                    println!("\t\tERROR: NOT FOUND");
                }
            }
            println!("\tparents:");
            for i in p {
                if let Some(&wikidata::Page { title: ref y, .. }) = wikidata::ENTRIES.get(i) {
                    println!("\t\t`{}`", y);
                } else {
                    println!("\t\tERROR: NOT FOUND");
                }
            }
		} else {
			println!("Couldn't find {}", x);
		}
	} else {
		println!("Couldn't find `Rust`");
	}
}
