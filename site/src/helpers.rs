//use bfs;
//use wikidata;
use rocket::http::uri::URI; // URI::percent_decode
use std::borrow::Cow;
use super::database::SortOptions;


// HELPERS

#[derive(FromForm, Debug)]
pub struct Search<'a> {
    pub src: &'a str,
    pub dst: &'a str,
    //pub cache: Option<Vec<(&'a str, &'a str, i16)>>,
    pub cache_sort: Option<&'a str>,
}

impl<'a> Search<'a> {
    fn prep_src(&'a self) -> Cow<'a, str> {
        preprocess(self.src)
    }
    fn prep_dst(&'a self) -> Cow<'a, str> {
        preprocess(self.dst)
    }
    pub fn prep(&'a self) -> (Cow<'a, str>, Cow<'a, str>) {
        (self.prep_src(), self.prep_dst())
    }
    pub fn sort_option(&self) -> Option<SortOptions> {
        match self.cache_sort {
            Some(x) if x.to_lowercase() == "recent"  => Some(SortOptions::Recent),
            Some(x) if x.to_lowercase() == "popular" => Some(SortOptions::Popular),
            Some(x) if x.to_lowercase() == "length"  => Some(SortOptions::Length),
            _ => None,
        }
    }
}

#[derive(FromForm, Debug)]
pub struct BfsApi<'a> {
    pub src_title:  Option<&'a str>,
    pub dst_title:  Option<&'a str>,
    pub src_id:     Option<u32>,
    pub dst_id:     Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct Context<'a> {
    //pub cache:      Option<&'a str>,
    //pub cache:      Option<&'a str>,
    pub cache:      Option<Vec<(&'a str, &'a str, i16)>>,
    pub src_t:      Option<String>, //todo
    pub dst_t:      Option<String>,
    pub bad_src:    bool,
    pub bad_dst:    bool,
    //pub path:       Option<&'a str>,
    //pub path:       Option<String>,
    //pub path:       Option<Vec<String>>,
    pub path:       Option<Vec<(&'static str, String)>>,
    pub src_alts:   Option<Vec<String>>,    //todo: &'a str ?
    pub dst_alts:   Option<Vec<String>>,
    //pub dst_err:    Option<String>,
    pub path_err:   Option<String>,
    //pub src_err: Option<&'a str>,
    //pub dst_err: Option<&'a str>,
}

impl<'a> Context<'a> {
    pub fn blank() -> Context<'a> {
        Context {
            cache: None,
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


#[derive(Serialize)]
pub enum PathResult {
    Path {
        lang:   &'static str,
        src:    &'static str,
        dst:    &'static str,
        len:    usize,
        nodes:  Vec<u32>,
    }, 
    Error(String),
}

pub fn preprocess<'a>(input: &'a str) -> Cow<'a, str> {
    let decoded = URI::percent_decode_lossy(input.as_bytes());
    //preprocess a string before it can become a valid title
    //first, replace any spaces with underscores (iff necessary)
    //replace spaces w/ underscores (how they are in the wiki dump)
    //replace pluses also, which are an artifact of html forms
    let escaped = |c| c == ' ' || c == '+';
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

