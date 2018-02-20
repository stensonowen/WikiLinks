/*
//#![feature(getpid)]
#![feature(alloc_system, global_allocator, allocator_api)]

extern crate alloc_system;
use alloc_system::System;
#[global_allocator] static A: System = System;
*/

// NOTE: when running on simple wiki, use `--features=simple` flag

#[macro_use]
extern crate clap;
extern crate links;
extern crate rand;

use links::link_state::{self, LinkState};

use clap::Arg;
fn argv<'a>() -> clap::ArgMatches<'a> {
    clap::App::new(crate_name!()).about(crate_description!())
        .author(crate_authors!()).version(crate_version!())

        .arg(Arg::with_name("import")
             .long("import")
             .short("i")
             .takes_value(true)
             .help("Import link and title data from link dumps manifest"))
        .arg(Arg::with_name("export")
             .long("output")
             .short("o")
             .takes_value(true)
             .help("Export link and title data to manifest and dumps"))

        .arg(Arg::with_name("compute_ranks")
             .long("compute-ranks")
             .takes_value(true)
             .conflicts_with("web-server")
             .help("After loading data, compute and save the pageranks")) 
        .arg(Arg::with_name("farthest_ancestor")
             .long("farthest-ancestor")
             .takes_value(true)
             .conflicts_with("web_server")
             .help("Find the maximum links required to get from any link to the given one"))
        .arg(Arg::with_name("cli-bfs")
             .long("cli-bfs")
             .help("Command-line bfs"))
        //.arg(Arg::with_name("web-server")
        //     .short("w")
        //     .help("Run web server; program will otherwise terminate after analysis"))

        .arg(Arg::with_name("page.sql")
             .short("p")
             .takes_value(true)
             .requires("redirect.sql")
             .requires("pagelinks.sql")
             .help("Pages db from wikipedia dump"))
        .arg(Arg::with_name("redirect.sql")
             .short("r")
             .takes_value(true)
             .requires("page.sql")
             .requires("pagelinks.sql")
             .help("Internal links db from wikipedia dump"))
        .arg(Arg::with_name("pagelinks.sql")
             .short("l")
             .takes_value(true)
             .requires("page.sql")
             .requires("redirect.sql")
             .help("Internal links db from wikipedia dump"))

        .get_matches()
}

/*
#[get("/bfs?<search>", rank = 1)]
fn bfs_search(search: web::SearchParams, conn: db::Conn, links: SharedLinks, 
              nc: NewCache, lc: LongCache) -> Template 
{
    let (src_f, dst_f) = search.fix();
    // TODO: translate empty query into random?
    let src_n = links.lookup_title(src_f.as_ref());
    let dst_n = links.lookup_title(dst_f.as_ref());
    let sort = match search.cache_sort {
        Some(s) => CacheSort::from_str(s).unwrap_or(DEFAULT_SORT),
        None => DEFAULT_SORT,
    };
    let path_res = if let (&Node::Found(s,ss), &Node::Found(d,ds)) = (&src_n, &dst_n) {
        if let Some(db_path) = cache::lookup_path(&*conn, s, d) {
            // return the path that was saved last time
            PathRes::from_db_path(db_path, links.get_links())
        } else {
            // can't find record of previous search; perform for the first time
            let path = links.bfs(s,d);
            if let Some(len) = path.size() {
                if len >= CACHEWORTHY_LENGTH {
                    // TODO: kill the clone
                    cache::insert_path(&conn, path.clone());
                }
                let ce = CacheElem::new(ss, ds, len);
                if lc.should_insert(&ce) {
                    lc.insert_elem(ce.clone());
                }
                nc.insert_elem(ce);
            }
            PathRes::from_path(path, links.get_links())
        }
    } else {
        // invalid request; search not run
        PathRes::NotRun
    };
    let cache = match sort {
        CacheSort::Recent => nc.get(),
        CacheSort::Length => lc.get(),
    };
    let context = Context {
        path:       path_res,
        src_search: src_n,
        dst_search: dst_n,
        cache:      cache,
        cache_sort: sort,
    };
    Template::render("bfs", &context)
}
*/

//use links::link_state::bfs::BFS;
//fn loop_bfs(pd: &

extern crate chrono;
use chrono::Local;
use links::link_state::Path;

fn _time_search(ls: &LinkState<link_state::ProcData>, src: u32, dst: u32) -> (i64,Path) {
    let start = Local::now();
    let p = ls.bfs(src, dst);
    let dur = Local::now().signed_duration_since(start);
    (dur.num_nanoseconds().unwrap(), p)
}

fn _random_elem(ls: &LinkState<link_state::ProcData>) -> u32 {
    let mut guess: u32;
    loop {
        guess = rand::random();
        if ls.contains(guess) {
            return guess;
        }
    }
}

fn main() {
    let argv = argv();
    /*
    if argv.is_present("cli-bfs") {
        let ls: LinkState<link_state::HashLinks> = LinkState::from_args(&argv);
        ls.cli_bfs().expect("io error");
    }
    */

    let ls: LinkState<link_state::HashLinks> = LinkState::from_args(&argv);
    let (src,dst) = if cfg!(feature="simple") { 
        (152_629, 454_989) 
    } else { 
        (1_684_129, 52_186_157) 
    };

    // 172504K
    // 172500K

    // none         293,810,837 bytes
    // bfs          301,149,929 bytes
    // bfs2 (.50)   300,101,737 bytes

    let path = ls.bfs(src, dst);
    println!("{:?}", path);

    //println!("\n\n\nMEMORY USED:\n");
    //::std::process::Command::new("/usr/bin/pmap")
    //    .arg(format!("{}", ::std::process::id()))
    //    .spawn().unwrap();


    /*
    let ls: LinkState<link_state::ProcData> = LinkState::from_args(&argv);
    //let ls: LinkState<link_state::HashLinks> = LinkState::from_args(&argv);
    //ls.longest_path(309528); // "1961–62_AHL_season"
    // Yobibyte: 401967  →  1961-62_AHL_season: 309528

    let mut random_walk = (0..10_000).map(|_| {
        let (src, dst) = (random_elem(&ls), random_elem(&ls));
        time_search(&ls, src, dst)
    }).collect::<Vec<(i64,Path)>>();
    random_walk.sort_by(|i,j| i.0.cmp(&j.0));

    for (i,p) in random_walk {
        if let Ok(ref v) = p.path {
            let titles: Vec<&String> = v.iter().map(|&n| ls.get(n)).collect();
            println!("{:08}:\t{:?}\t{:?}", i, p, titles);
        }
        //println!("{:08}:\t{:?}", i, p);
    }
    */

    /*
    let mut guess: u32;
    let mut count = 0;
    loop {
        guess = rand::random();
        if ls.contains(guess) {
            count += 1;
            ls.longest_path(guess);
            if count > 25 {
                break;
            }
            println!("\n\n\n");
        }
    }
    */

}

