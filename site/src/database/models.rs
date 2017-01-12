
//TODO: are i32's large enough for page_id's?
// any page_id's ≥ 2**31 ?
#[derive(Queryable)]
pub struct Post {
    // index of this entry
    pub id: i32,
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
    //number of times this element has been requested
    pub count: i32,
}
