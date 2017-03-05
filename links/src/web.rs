// helpers for web component

use fnv;
use std::str::FromStr;
use link_state::Entry;
use link_state::hash_links::{Path, PathError};

const WIKI_URL_FMT: &'static str = "https://simple.wikipedia.org/?curid=";

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
    //path:       Option<Vec<(String,String)>>,
    pub path:       PathRes<'a>,
    pub src_search: Node<'a>,
    pub dst_search: Node<'a>,
    pub cache:      Option<Vec<(&'a str, i8, &'a str)>>,
    pub cache_sort: CacheSort,
}


impl<'a> Context<'a> {
    //pub fn new(path: PathRes, src: Node, dst: Node, 
    //           cache: Option<Vec<(&'a str, i8, &'a str)>>, sort: CacheSort) 
    //    -> Context<'a> 
    //{
    //    Context {
    //        path:       path,
    //        src_search: src,
    //        dst_search: dst,
    //        cache:      cache,
    //        cache_sort: sort,
    //    }
    //}
    pub fn from_cache(sort: CacheSort, cache: Option<Vec<(&'a str, i8, &'a str)>>) 
        -> Context<'a> 
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
    //Sugg(Vec<&'a str>),
    Sugg(Vec<String>),      // when requested from diesel we get Strings anyway
    Unknown(&'a str),
    Unused
}

impl<'a> Node<'a> {
    pub fn valid(&self) -> bool {
        match self {
            &Node::Found(..) => true,
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

use super::cache::models::DbPath;
impl<'a> PathRes<'a> {
    pub fn from_db_path(db_p: DbPath, links: &fnv::FnvHashMap<u32,Entry>) -> PathRes {
        match db_p.result {
            0           => PathRes::NoSuchPath,
            i if i < 0  => PathRes::Terminated((-1*i) as u32),
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
    Popular,
    Length,
    Random,
}

impl FromStr for CacheSort {
    type Err = ();
    fn from_str(input: &str) -> Result<CacheSort, Self::Err> {
        match input.to_lowercase().as_str() {
            "recent" | "latest" | "new" | "newest"  => Ok(CacheSort::Recent),
            "popular" | "top" | "best" | "hot"      => Ok(CacheSort::Popular),
            "length" | "longest" | "size"           => Ok(CacheSort::Length),
            "random" | "rand" | "idk"               => Ok(CacheSort::Random),
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
    pub sort: Option<&'a str>,
}

