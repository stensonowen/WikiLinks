//use bfs;
//use wikidata;
use rocket::http::uri::URI; // URI::percent_decode
use std::borrow::Cow;
use std::str::FromStr;

pub const DEFAULT_CACHE_SORT: SortOptions = SortOptions::Recent;

// BASIC REQUEST-RELATED TYPES:

#[derive(Serialize)]
pub enum BfsApiResult {
    //Result of a bfs Api call
    Success {
        ids: Vec<u32>,
        titles: Vec<&'static str>,
    },
    TerminatedAfter(usize),
    InvalidSrc,
    InvalidDst,
    InvalidSrcAndDst,
    NoSuchPath,
}


#[derive(Debug, Serialize, Clone, Copy)]
pub enum SortOptions {
    //Different ways the cache can be sorted
    Recent,
    Popular,
    Length,
    //Random,   //how to do efficiently?
}

impl SortOptions {
    pub fn convert(input: &str) -> Option<SortOptions> {
        SortOptions::from_str(input).ok()
    }
}

impl FromStr for SortOptions {
    type Err = ();
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        //TODO: do this w/ patterns intead of to_lowercase() ?
        let lower_in = input.to_lowercase();
        match lower_in.as_str() {
            "recent" | "latest" | "new" | "newest" => Ok(SortOptions::Recent),
            "popular" | "top" | "best" | "hot" => Ok(SortOptions::Popular),
            "length" | "longest" | "size" => Ok(SortOptions::Length),
            _ => Err(()),
        }
    }
}


#[derive(Debug, Serialize)]
pub struct Context<'a> {
    // All data that can be passed to tera template
    //TODO: replace some Strings w/ &'a strs 
    pub cache:      Option<Vec<(&'a str, &'a str, i16)>>,
    //pub cache_sort: Option<&'a str>,
    pub cache_sort: SortOptions,
    pub src_t:      Option<String>, //todo
    pub dst_t:      Option<String>,
    pub bad_src:    bool,
    pub bad_dst:    bool,
    pub path:       Option<Vec<(&'static str, String)>>,
    pub src_alts:   Option<Vec<String>>,
    pub dst_alts:   Option<Vec<String>>,
    pub path_err:   Option<String>,
}

impl<'a> Context<'a> {
    pub fn blank() -> Context<'a> {
        Context {
            cache: None,
            cache_sort: DEFAULT_CACHE_SORT,
            src_t: None,
            dst_t: None,
            bad_src: true,
            bad_dst: true,
            path: None,
            src_alts: None,
            dst_alts: None,
            path_err: None,
        }
    }
}


// QUERY PARAMETER TYPES

#[derive(FromForm, Debug)]
pub struct CacheSortParam<'a> {
    pub cache_sort: Option<&'a str>,
}


#[derive(FromForm, Debug)]
pub struct BfsApiParams<'a> {
    pub src_title:  Option<&'a str>,
    pub dst_title:  Option<&'a str>,
    pub src_id:     Option<u32>,
    pub dst_id:     Option<u32>,
}


#[derive(FromForm, Debug)]
pub struct SearchParams<'a> {
    pub src: &'a str,
    pub dst: &'a str,
    pub cache_sort: Option<&'a str>,
}

impl<'a> SearchParams<'a> {
    fn prep_src(&'a self) -> Cow<'a, str> {
        preprocess(self.src)
    }
    fn prep_dst(&'a self) -> Cow<'a, str> {
        preprocess(self.dst)
    }
    pub fn prep(&'a self) -> (Cow<'a, str>, Cow<'a, str>) {
        (self.prep_src(), self.prep_dst())
    }
}


// HELPER FUNCTIONS

pub fn preprocess<'a>(input: &'a str) -> Cow<'a, str> {
    let decoded = URI::percent_decode_lossy(input.as_bytes());
    //preprocess a string before it can become a valid title
    //first, replace any spaces with underscores (iff necessary)
    //replace spaces w/ underscores (how they are in the wiki dump)
    //replace pluses also, which are an artifact of html forms
    let escaped = |c| c == ' ';
    let decoded = {
        if decoded.contains(&escaped) {
            //urls/forms turn spaces into 
            Cow::Owned(decoded.replace(&escaped, &"_"))
        } else {
            decoded
        }
    };
    let decoded = {
        if decoded.contains("%2B") {
            Cow::Owned(decoded.replace("%2B", &"+"))
        } else {
            decoded
        }
    };
    decoded
}

