extern crate wikidata;
extern crate bfs;
extern crate clap;
use std::io::{self, Write};
use wikidata::ADDRESSES;
use bfs::{bfs, search, print_path, load_titles};
use clap::{App, Arg};

// Notes about using:
//  - Performing the same search multiple times might yield different paths
//      (though always with the same length)
//      An empty value for src/dst article titles means to use the prior value
//      So after performing a search, hit Enter twice to do it again
//  - Article searches are case sensitive
//  - Search recommendations are based on substrings, not string similarity.
//      So for a page to be recommended, your query must be a part of its article title
//      Which is still case sensitive. (I know that's a little harsh)
//  - Redirects are accounted for. "Ronald Reagan" and "Reagan" are treated as identical
//  - Control-C to quit
//
//

fn main() {
    let matches = App::new("simple.wikilinks")
        .version("1.0")
        .author("owen <stensonowen@gmail.com>")
        .about("Find link paths between simple.wikipedia.org articles (data from Dec 2016)")
        .arg(Arg::with_name("links")
            .short("l")
            .long("links")
            .help("Print followable links in paths"))
        .get_matches();
    load_titles();  //get this out of the way (probably doesn't have to be lazy)
    let print_links = match matches.is_present("links") {
        true  => Some("simple"),
        false => None,
    };
    println!("{:?}", print_links);

    let mut src_id = None;
    let mut dst_id = None;
    loop {
        println!("Search from one Simple Wikipedia article to another by links");
        print!("Enter the title of the SOURCE article:       ");
        src_id = Some(get_article(&src_id));
        print!("Enter the title of the DESTINATION article:  ");
        dst_id = Some(get_article(&dst_id));
        print_path(bfs(src_id.unwrap(), dst_id.unwrap()), print_links);
    }
}

fn get_article(prev: &Option<u32>) -> u32 {
    //request an article from the user and return the page_id
    //if it's invalid, try to give them some recommendations
    //if they enter nothing, maybe default back to the previous value (`prev`)
    let mut buffer = String::new();
    let mut page_id: Option<u32> = None;
    while page_id.is_none() {
        buffer.clear();
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut buffer).expect("Failed to read input");
        let fix = buffer.trim().replace(' ', "_");
        if fix.is_empty() {
            if let &Some(p) = prev {
                return p;
            } else {
                println!("No prior article was found");
                print!("Please enter another article title to try:   ");
                continue;
            }
        }
        page_id = match ADDRESSES.get(fix.as_str()) {
            Some(id) => Some(*id),
            None => {
                let guesses = search(&fix);
                if guesses.len() == 0 {
                    println!("No such page or similar articles were found");
                    print!("Please enter another article title to try:   ");
                    continue;
                }
                println!("No page title found with that name. Perhaps you meant:");
                for (i, g) in guesses.iter().enumerate() {
                    println!("\t{}:\t`{}`", i, g);
                }
                print!("Enter the number corresponding to your selection, or 'x' to retry: ");
                buffer.clear();
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut buffer).expect("Failed to read input");
                match buffer.trim().parse::<usize>() {
                    Ok(i @ 0...10) if i < guesses.len() => {
                        Some(*ADDRESSES.get(guesses.get(i).unwrap()).unwrap())
                    }
                    _ => {
                        print!("Please enter another article title to try:   ");
                        None
                    }
                }
            }
        };
    }
    page_id.unwrap()
}
