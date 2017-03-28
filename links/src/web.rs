// helpers for web component

use fnv;
use std::str::FromStr;
use std::borrow::Cow;
use link_state::Entry;
use link_state::hash_links::{Path, PathError};
use rocket::http::uri::URI;

use super::cache::models::DbPath;
use super::cache::cache_elem::CacheElem;
//use super::cache::stack_cache::CacheElem;
use super::WIKI_URL_FMT;

#[derive(Debug, Serialize)]
pub struct Context<'a> {
    //be able to represent:
    //  Successful path: 
    //      Each node name and number (and url?)
    //  No such Source
    //      Potential Suggestions
    //  No such Destination
    //      Potential Suggestions
    //  No such Source OR Destination
    //      Potential Suggestions
    //  Unsuccessful Path
    //      Terminated after X iterations
    //      No such path
    //  Cache
    //      Sorting options
    //      Preview of elements
    pub path:       PathRes<'a>,
    pub src_search: Node<'a>,
    pub dst_search: Node<'a>,
    pub cache:      Vec<CacheElem>,
    pub cache_sort: CacheSort,
}


impl<'a> Context<'a> {
    pub fn from_cache(sort: CacheSort, cache: Vec<CacheElem>) -> Context<'a>
{
        Context {
            path:       PathRes::NotRun,
            src_search: Node::Unused,
            dst_search: Node::Unused,
            cache:      cache,
            cache_sort: sort,
        }
    }
}


#[derive(Debug, Serialize)]
pub enum Node<'a> {
    Found(u32, &'a str),
    Sugg(Vec<String>),      // when requested from diesel we get Strings anyway
    Unknown(&'a str),
    Unused
}

impl<'a> Node<'a> {
    pub fn valid(&self) -> bool {
        match *self {
            Node::Found(..) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum PathRes<'a> {
    // reference to HashLinks? seems hard
    Success(Vec<(&'a str, String)>),
    Terminated(u32),
    NoSuchPath,
    NotRun,
}

impl<'a> PathRes<'a> {
    pub fn from_db_path(db_p: DbPath, links: &fnv::FnvHashMap<u32,Entry>) -> PathRes {
        match db_p.result {
            0           => PathRes::NoSuchPath,
            i if i < 0  => PathRes::Terminated((-i) as u32),
            _ => {
                PathRes::Success(db_p.path.into_iter().map(|i| {
                    // string manip not super efficient
                    // better to use format!() ?
                    let link = links.get(&(i as u32)).unwrap();
                    let title = link.title.as_ref();
                    let mut url = String::from(WIKI_URL_FMT);
                    url.push_str(&i.to_string());
                    (title, url)
                }).collect())
            }
        }
    }
    pub fn from_path(p: Path, links: &fnv::FnvHashMap<u32,Entry>) -> PathRes {
        match p.path {
            Err(PathError::NoSuchPath)      => PathRes::NoSuchPath,
            Err(PathError::Terminated(i))   => PathRes::Terminated(i),
            Ok(path) => PathRes::Success(path.iter().map(|i| {
                    let link = links.get(i).unwrap();
                    let title = link.title.as_ref();
                    let mut url = String::from(WIKI_URL_FMT);
                    url.push_str(&i.to_string());
                    (title, url)
                }).collect())
        }
    }
}

#[derive(Debug, Serialize)]
pub enum CacheSort {
    Recent,
    Length,
    //Popular,
    //Random,
    //By combined pagerank?
}

impl FromStr for CacheSort {
    type Err = ();
    fn from_str(input: &str) -> Result<CacheSort, Self::Err> {
        match input.to_lowercase().as_str() {
            "recent" | "latest" | "new" | "newest"  => Ok(CacheSort::Recent),
            //"popular" | "top" | "best" | "hot"      => Ok(CacheSort::Popular),
            "length" | "longest" | "size"           => Ok(CacheSort::Length),
            //"random" | "rand" | "idk"               => Ok(CacheSort::Random),
            _ => Err(()),
        }
    }
}

#[derive(FromForm)]
pub struct SortParam<'a> {
    pub by: Option<&'a str>
}

#[derive(FromForm)]
pub struct SearchParams<'a> {
    pub src: &'a str,
    pub dst: &'a str,
    pub cache_sort: Option<&'a str>,
}

impl<'a> SearchParams<'a> {
    pub fn fix(&self) -> (Cow<'a, str>, Cow<'a, str>) {
        // perform preprocessing on user input 
        (preprocess(self.src), preprocess(self.dst))
    }
}

fn preprocess(input: &str) -> Cow<str> {
    // pluses become underscores
    // %20Bs become pluses
    let spaces = if input.contains('+') {
        Cow::Owned(input.replace('+', "_"))
    } else {
        Cow::Borrowed(input)
    };
    if input.contains('%') {
        Cow::Owned(
            URI::percent_decode_lossy(
                spaces.as_ref().as_bytes())
            .to_string())
    } else {
        spaces
    }
}
