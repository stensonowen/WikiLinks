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
use super::phf;

use self::models::*;
use self::schema::paths::dsl::paths;
use self::schema::paths::dsl as paths_row;
use self::schema::paths::table as paths_table;
use self::schema::titles::dsl::titles;
use self::schema::titles::dsl as titles_row;
use self::schema::titles::table as titles_table;

use r2d2::{Pool, PooledConnection, GetTimeout, Config};
use r2d2_diesel::ConnectionManager;

use super::rocket::request::{Outcome, FromRequest};
use rocket::Outcome::{Success, Failure};
use super::rocket::Request;
use super::rocket::http::Status;

pub mod schema;
pub mod models;


pub struct DB(PooledConnection<ConnectionManager<PgConnection>>);

lazy_static! {
    pub static ref DB_POOL: Pool<ConnectionManager<PgConnection>> = create_db_pool();
}

impl DB {
    pub fn conn(&self) -> &PgConnection {
        &*self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for DB {
    type Error = GetTimeout;
    fn from_request(_: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match DB_POOL.get() {
            Ok(conn) => Success(DB(conn)),
            Err(e) => Failure((Status::InternalServerError, e)),
        }
    }
}

pub fn create_db_pool() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let config = Config::default();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::new(config, manager).expect("Failed to create pool.")
}


pub fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}


pub enum PathOption {
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
        .set(paths_row::timestamp.eq(UTC::now()))
        .get_result::<Path>(conn)?;

    let new_count = path.count + 1;
    diesel::update(target).set(paths_row::count.eq(new_count)).execute(conn)?;
    Ok(path.path.into_iter().map(|i| i as u32).collect())
}

use super::bfs;

pub fn insert_path(conn: &PgConnection, 
                   src: u32, 
                   dst: u32, 
                   //path: PathOption) 
                   path: &Result<Vec<u32>,bfs::Error>)
        -> Result<usize,Error> {
    //insert a path from src to dst
    //failure is recoverable: this entry might be invalid, but 
    // it can just be calculated / inserted again next time
    let (result, path) = match path {
        //vec![] should create vector w/ 0 capacity by default
        //so needing to return an empty vec instead of None isn't too bad
        //PathOption::Path(v) => (0i16, v.into_iter().map(|i| i as i32).collect()),
        //PathOption::Terminated(i) => ((i as i16) * -1, vec![]),
        //PathOption::NoSuchPath => (-1i16, vec![]),
        &Ok(ref v) => (0i16, v.into_iter().map(|&i| i as i32).collect()),
        &Err(bfs::Error::Terminated(i)) => ((i as i16) * -1, vec![]),
        &Err(bfs::Error::NoSuchPath) => (-1i16, vec![]),
    };
    let new_path = NewPath {
        src: src as i32,
        dst: dst as i32,
        result: result,
        path: path,
        timestamp: UTC::now()
    };
    insert(&new_path).into(paths_table).execute(conn)
}


pub fn populate_addrs(conn: &PgConnection, 
                      addrs: &'static phf::Map<&'static str, u32>) 
        -> Result<usize,Error> {
    let mut i = 0;
    for (title, &addr) in addrs.into_iter() {
        let new_addr = NewAddress {
            title: title,
            page_id: addr as i32,
        };
        insert(&new_addr).into(titles_table).execute(conn)?;
        i += 1;
    }
    Ok(i)
}

#[derive(Serialize)]
pub enum AddressLookup {
    Address(u32),
    Suggestions(Vec<String>),
}

pub fn lookup_addr(conn: &PgConnection, query: &str) -> Result<AddressLookup,Error> {
    //let lookup = titles.find(titles_row::title.eq(query));//.first(conn);
    //let foo: QueryResult<Address> = titles.find(query).first(conn);
    
    if let Ok(Address{ page_id: id, .. }) = titles.find(query).first(conn) {
        Ok(AddressLookup::Address(id as u32))
    } else {
        //order by pagerank?
        //or would that be super expensive?
        let fuzzy_query = format!("%{}%", query);
        let guesses = titles.filter(titles_row::title.like(fuzzy_query))
            .limit(10)
            .load::<Address>(conn)?;
        let g = guesses.into_iter().map(|i| i.title.clone()).collect();
        Ok(AddressLookup::Suggestions(g))
    }
}

pub fn purge_cache(conn: &PgConnection) -> Result<usize,Error> {
    println!("Warning: purging cache");
    diesel::delete(paths.filter(paths_row::src.gt(i32::min_value()))).execute(conn)
}
