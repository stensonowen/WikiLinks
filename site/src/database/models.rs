
use super::chrono::naive::datetime::NaiveDateTime;
use super::chrono::datetime::DateTime;
use super::chrono::offset::utc::UTC;
//use super::super::diesel::types::Nullable;
//use super::super::diesel::expression::nullable::Nullable;

//TODO: are i32's large enough for page_id's?
// any page_id's ≥ 2**31 ?
#[derive(Queryable)]
pub struct Path {
    // index of this entry
    //pub id: i32,
    //src page_id
    pub src: i32,
    //dst page_id
    pub dst: i32,
    //Result of search: can be
    //  0: success
    // -1: definitely no path
    // ≥0: search cancelled after n iterations
    pub result: i16,

    //page_ids of the path
    pub path: Vec<i32>,    //not Option<Vec<i32>> ?
    //pub path: Option<Vec<i32>>,
    //pub path: Nullable<Vec<i32>>,

    //number of times this element has been requested
    pub count: i32,
    //the last time this element was requested
    pub timestamp: NaiveDateTime,
}

use super::schema::paths;

#[derive(Insertable)]
#[table_name="paths"]
pub struct NewPath {
    //pub id: i32,
    pub src: i32,
    pub dst: i32,
    pub result: i16,
    pub path: Vec<i32>,
    pub timestamp: DateTime<UTC>,
}

