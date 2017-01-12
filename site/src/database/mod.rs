// Database Layout: One row composed of:
//  Cache:
//      Path:       array of 32-bit unsigned integers indicating the path, including src/dst
//      Timestamp:  the last time this entry was accessed
//      Count:      the number of times this entry was accessed
//  Addresses:
//      Title:      titles (that we can fuzzy-select?) (`tsvector`?)
//      Address:    u32 page_id
//      Rank:       f64 pagerank ?
//   search both ways?

extern crate dotenv;
extern crate chrono;

use diesel::{self, insert};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::result::Error;

use self::chrono::offset::utc::UTC;
use self::dotenv::dotenv;
use std::env;

use self::models::*;
use self::schema::paths::dsl::paths;
use self::schema::paths::dsl as row;
use self::schema::paths::table;

pub mod schema;
pub mod models;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn test() {
    let connection = establish_connection();
    //let results = paths::dsl::paths.filter(paths::dsl::src.eq(0))
    let results = paths.filter(row::src.eq(0).and(row::dst.eq(1)))
        .limit(5)
        .load::<Path>(&connection)
        .expect("Error loading paths");

    println!("Displaying {} paths", results.len());
    for p in results {
        println!("{}", p.src);
        println!("----------\n");
        println!("{}", p.count);
    }
}


pub enum Row {
    Path(Vec<u32>),
    Terminated(u32),
    NoSuchPath,
}

pub fn get_path(conn: &PgConnection, src: u32, dst: u32) -> Result<Vec<u32>,Error> {
    //adjust timestamp and count
    //Error means path does not exist 
    //  (or that it wasn't updated properly, which is no biggie)
    let target = paths.find((src as i32, dst as i32));  // ?!
    let path = diesel::update(target)
        .set(row::timestamp.eq(UTC::now()))
        .get_result::<Path>(conn)?;

    let new_count = path.count + 1;
    diesel::update(target).set(row::count.eq(new_count)).execute(conn)?;
    Ok(path.path.into_iter().map(|i| i as u32).collect())
}

pub fn insert_path(conn: &PgConnection, src: u32, dst: u32, path: Row) -> Result<usize,Error> {
    //insert a path from src to dst
    //failure is recoverable: this entry might be invalid, but 
    // it can just be calculated / inserted again next time
    let (result, path) = match path {
        //vec![] should create vector w/ 0 capacity by default
        //so needing to return an empty vec instead of None isn't too bad
        Row::Path(v) => (0i16, v.into_iter().map(|i| i as i32).collect()),
        Row::Terminated(i) => ((i as i16) * -1, vec![]),
        Row::NoSuchPath => (-1i16, vec![]),
    };
    let new_path = NewPath {
        src: src as i32,
        dst: dst as i32,
        result: result,
        path: path,
        timestamp: UTC::now()
    };
    insert(&new_path).into(table).execute(conn)
}



