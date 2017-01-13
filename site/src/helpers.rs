use bfs;
use wikidata;
use rocket::http::uri::URI; // URI::percent_decode
use std::borrow::Cow;


// HELPERS

#[derive(FromForm, Debug)]
pub struct Search<'a> {
    pub src: &'a str,
    pub dst: &'a str,
    //pub submit: Option<&'a str>,
    //pub submit: &'a str,
}

impl<'a> Search<'a> {
    pub fn prep_src(&'a self) -> Cow<'a, str> {
        preprocess(self.src)
    }
    pub fn prep_dst(&'a self) -> Cow<'a, str> {
        preprocess(self.dst)
    }
    //pub fn prep_dst(&'a self) -> &'a str {
    //    preprocess(self.dst).as_ref()
    //}
}

#[derive(Debug, Serialize)]
pub struct Context<'a> {
    //pub cache:      Option<&'a str>,
    pub cache:      Option<&'a str>,
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

pub fn resolve_titles(search: &Search) -> (Option<String>, Option<String>) {
    (resolve_title(search.src, "source"), resolve_title(search.dst, "destination"))

}

pub fn preprocess<'a>(input: &'a str) -> Cow<'a, str> {
    let decoded = URI::percent_decode_lossy(input.as_bytes());
    //preprocess a string before it can become a valid title
    //first, replace any spaces with underscores (iff necessary)
    if decoded.contains(' ') {
        Cow::Owned(decoded.replace(' ', &"_"))
    } else {
        decoded
    }
}


fn resolve_title(query: &str, name: &str) -> Option<String> {
    //print helpful info (iff relevant)
    let query_ = preprocess(query);
    //let decoded = URI::percent_decode_lossy(query.as_bytes());
    //let fixed = bfs::preprocess(decoded.as_ref());
    if let Some(_) = wikidata::ADDRESSES.get(query_.as_ref()) {
        None
    } else {
        let guesses = bfs::search(query_.as_ref());
        if guesses.is_empty() {
            Some(format!("<p style=\"color:#FF0000;\">No ideas found for `{}` for the {}</p>", 
                         query, name))
        } else {
            let mut s = String::new();
            s.push_str("No articles found with that name for the ");
            s.push_str(name);
            s.push_str(". Maybe you meant: <ul>");
            for g in guesses {
                s.push_str("<li>");
                s.push_str(g);
                s.push_str("</li>");
            }
            s.push_str("</ul>");
            Some(s)
        }
    }

}

