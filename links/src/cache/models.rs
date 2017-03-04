
use chrono::datetime::DateTime;
use chrono::offset::utc::UTC;
use super::schema::paths;
use super::schema::titles;
use super::Path;
use super::super::hash_links::PathError;

#[derive(Insertable, Queryable)]
#[table_name="paths"]
pub struct DbPath {
    // indexed by (src,dst)
    pub src: i32,
    pub dst: i32,
    //  result = 0:    definitely no path
    //  result < 0: search terminated after |result| iters
    //  result > 0: search succeeded after `result` iters
    pub result: i16,
    pub path: Vec<i32>, // option?
    pub count: i32,
    pub timestamp: DateTime<UTC>
}

impl From<Path> for DbPath {
    fn from(p: Path) -> DbPath {
        DbPath {
            src: p.src as i32,
            dst: p.dst as i32,
            result: match p.path {
                Ok(ref v) => v.len() as i16,
                Err(PathError::NoSuchPath) => 0,
                Err(PathError::Terminated(i)) => -1*i as i16,
            },
            path: match p.path {
                Ok(v) => v.into_iter().map(|i| i as i32).collect(),
                Err(_) => vec![],
            },
            count: 1,
            timestamp: UTC::now()
        }
    }
}


#[derive(Insertable, Queryable)]
#[table_name="titles"]
pub struct DbAddr {
    pub title: String,
    pub page_id: i32,
    pub pagerank: Option<f64>,
}

impl DbAddr {
    pub fn blank(t: String, n: u32) -> Self {
        DbAddr {
            title: t,
            page_id: n as i32,
            pagerank: None,
        }
    }
}

