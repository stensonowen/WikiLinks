//use diesel;

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

use diesel::prelude::*;
use diesel::pg::PgConnection;
use self::dotenv::dotenv;
use std::env;

use self::models::*;
use self::schema::paths::dsl::*;


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
    let results = paths.filter(id.eq(0))
        .limit(5)
        .load::<Post>(&connection)
        .expect("Error loading posts");

    println!("Displaying {} posts", results.len());
    for post in results {
        println!("{}", post.id);
        println!("----------\n");
        println!("{}", post.count);
    }
}



