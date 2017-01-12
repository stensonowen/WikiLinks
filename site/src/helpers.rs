use bfs;
use wikidata;
use rocket::http::uri::URI; // URI::percent_decode
// HELPERS

#[derive(FromForm)]
pub struct Search<'a> {
    pub src: &'a str,
    pub dst: &'a str,
}

#[derive(Debug, Serialize)]
pub struct Context<'a> {
    pub cache:      Option<&'a str>,
    pub path:       Option<&'a str>,
    pub src_search: Option<String>,
    pub dst_search: Option<String>,
    //pub src_search: Option<&'a str>,
    //pub dst_search: Option<&'a str>,
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

#[derive(Serialize)]
pub enum SearchResult {
    PageId(u32),
    Recommendations(Vec<&'static str>),
    NoGuesses,
}

pub fn resolve_titles(search: &Search) -> (Option<String>, Option<String>) {
    (resolve_title(search.src, "source"), resolve_title(search.dst, "destination"))

}

fn resolve_title(query: &str, name: &str) -> Option<String> {
    //print helpful info (iff relevant)
    let decoded = URI::percent_decode_lossy(query.as_bytes());
    let fixed = bfs::preprocess(decoded.as_ref());
    if let Some(_) = wikidata::ADDRESSES.get(fixed.as_ref()) {
        None
    } else {
        let guesses = bfs::search(query);
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
