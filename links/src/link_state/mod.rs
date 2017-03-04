// https://hoverbear.org/2016/10/12/rust-state-machine-pattern/

use fnv;
use slog_term;
use slog::{Logger, DrainExt};

// STD
use std::sync::Mutex;

// HELPERS
pub mod link_db;
pub mod link_data;
pub mod rank_data;
pub mod hash_links;

const IS_SIMPLE: bool = true;


//  ------STATE--MACHINE------


pub trait State { }
impl State for LinkDb { }
impl State for LinkData { }
impl State for RankData { }
impl State for HashLinks { }

pub struct LinkState<S: State> {
    threads: usize,     // number of threads/files to use concurrently
    size:    usize,     // number of entries
    log:     Logger,    // root logger that will be split off for components
    state:   S,         // 1 of 4 values that represent development of the data
}

fn new_logger() -> Logger {
    Logger::root(slog_term::streamer().compact().build().fuse(), o!())
}


//  ----------STATES----------


pub struct LinkDb {
    /// Store parsed collection of links from sql dumps
    db: link_db::parse::database::Database,
}

pub struct LinkData {
    /// Store parsed and converted links and addresses
    /// Link data can be quickly written to or read from disk
    dumps: Vec<Mutex<Vec<link_data::IndexedEntry>>>,
    addrs: Vec<(String,u32)>,
}

pub struct RankData {
    /// Store easily searchable link and pagerank data
    /// Pagerank data can be read from, dumped to, or exported to disk 
    links: fnv::FnvHashMap<u32,Entry>,
    ranks: fnv::FnvHashMap<u32, f64>,
}

pub struct HashLinks {
    /// Read-only, fast-lookup container for link and rank data
    /// Interact with diesel cache and interface with website
    links: fnv::FnvHashMap<u32,Entry>,
    ranks: fnv::FnvHashMap<u32, f64>,
}


// ------COMMON-OBJECTS------


#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub title: String,
    pub parents:  Vec<u32>,
    pub children: Vec<u32>,
}


//  --------------------------


/*
fn argv<'a>() -> clap::ArgMatches<'a> {
    clap::App::new(crate_name!()).about(crate_description!())
        .author(crate_authors!("\n")).version(crate_version!())
        .arg(Arg::with_name("ranks").short("r").long("ranks_dump").takes_value(true)
             .help("Supply location of rank data in csv form"))
        .arg(Arg::with_name("manifest").short("json").takes_value(true)
             .help("Supply dump of parsed link data manifest in json form"))
        .arg(Arg::with_name("page.sql").short("pages").takes_value(true)
             .conflicts_with("manifest").requires("redirect.sql").requires("pagelinks.sql"))
        .arg(Arg::with_name("redirect.sql").short("dirs").takes_value(true)
             .conflicts_with("manifest").requires("page.sql").requires("pagelinks.sql"))
        .arg(Arg::with_name("pagelinks.sql").short("links").takes_value(true)
             .conflicts_with("manifest").requires("page.sql").requires("redirect.sql"))
        .group(clap::ArgGroup::with_name("sources").required(true)
               .args(&["sql", "manifest", "page.sql", "redirects.sql", "pagelink.sql"]))
        .get_matches()
}

fn run() {
    //let pages_db = PathBuf::from("/home/owen/wikidata/simplewiki-20170201-page.sql");
    //let redir_db = PathBuf::from("/home/owen/wikidata/simplewiki-20170201-redirect.sql");
    //let links_db = PathBuf::from("/home/owen/wikidata/simplewiki-20170201-pagelinks.sql");
    //let output = PathBuf::from("/home/owen/wikidata/dumps/simple_20170201_dump2");
    //let rank_file = Path::new("/home/owen/wikidata/dumps/simple_20170201_ranks1");
    //let p_rank_file = Path::new("/home/owen/wikidata/dumps/simple_20170201_pretty_ranks1");
    //let dump = PathBuf::from("/home/owen/wikidata/dumps/simple_20170201_dump1");

    //let ld = LinkState::<LinkData>::from_file(dump, new_logger()).unwrap();
    //let rd = LinkState::<RankData>::from_ranks(ld, rank_file);
    //rd.pretty_ranks(p_rank_file).unwrap();
    
    // ====

    let hl = LinkState::<HashLinks>::from_args(argv());
    println!("Size: {}", hl.size);

    //println!("{:?}", hl.bfs(232327,460509));
    hl.print_bfs(232327,460509);

    //thread::sleep(time::Duration::from_secs(30));
}
*/


