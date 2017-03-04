extern crate chrono;

use diesel;
use std::env;
use dotenv::dotenv;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use chrono::offset::utc::UTC;

use cache::models::*;
use super::link_state::hash_links::Path;
use super::link_state::Entry;
use std::collections::HashMap;

pub mod schema;
pub mod models;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("missing DATABASE_URL");
    PgConnection::establish(&db_url)
        .expect(&format!("Failed to connect to {}", db_url))

}

/*
fn run() {
    use cache::schema::paths::dsl::*;
    let conn = establish_connection();
    let results = paths.filter(count.eq(0))
        .limit(5)
        .load::<DbPath>(&conn)
        .expect("Error posts"); ;
}
*/


//  ---------GETTERS*---------


fn lookup_path(conn: &PgConnection, src: u32, dst: u32) -> Option<DbPath> {
    // NOTE: MODIFIES INTERNAL STATE: update count and timestamp
    // NOTE: failure is recoverable
    use self::schema::paths::dsl::paths;
    use self::schema::paths::dsl as path_row;
    let target = paths.find((src as i32, dst as i32));
    diesel::update(target)
        .set(path_row::timestamp.eq(UTC::now()))
        .get_result::<DbPath>(conn) 
        .ok()
        .and_then(|path| diesel::update(target)
                  .set(path_row::count.eq(path.count+1))
                  .get_result(conn)
                  .ok())
}

fn lookup_addr(conn: &PgConnection, query: &str) -> Result<u32,Vec<String>> {
    // return address corresponding to 
    // uhh, will diesel stop this from being a potential sql injection?
    // NOTE: failure is irrecoverable (panic!s)
    use self::schema::titles::dsl::titles;
    use self::schema::titles::dsl as title_row;
    if let Ok(DbAddr { page_id: id, .. }) = titles.find(query).first(conn) {
        // first try the exact query
        Ok(id as u32)
    } else {
        // would it be super expensive to order by pagerank?
        let fuzzy_query = format!("%{}%", query);
        let guesses = titles.filter(title_row::title.like(fuzzy_query))
            .limit(10).load::<DbAddr>(conn).unwrap();
        Err(guesses.into_iter().map(|i| i.title).collect())
    }
}


//  ----------SETTERS---------


fn insert_path(conn: &PgConnection, p: Path) -> Option<DbPath> {
    // NOTE: failure is recoverable
    use self::schema::paths;
    let new_path: DbPath = p.into();
    diesel::insert(&new_path).into(paths::table).get_result(conn).ok()
}

fn insert_title(conn: &PgConnection, t: String, n: u32) -> DbAddr {
    // NOTE: won't be run in production, fine to panic!
    // TODO: probably just insert a bunch at a time
    //  the allocation is probably worth it
    use self::schema::titles;
    //let new_addr: DbAddr = (t,n).into();
    let new_addr = DbAddr::blank(t, n);
    diesel::insert(&new_addr).into(titles::table).get_result(conn).unwrap()
}


//  --------TITLES-MGMT--------


//pub fn purge_cac
fn populate_addrs(conn: &PgConnection, 
                  ranks: &HashMap<u32,f64>,
                  links: &HashMap<u32,Entry>) 
    -> Result<usize,diesel::result::Error>
{
    use self::schema::titles;
    //okay that this involves a lot of copying
    //it doesn't have to be performant
    let rows: Vec<_> = links.iter().map(|(&i,e)| DbAddr {
        title: e.title.clone(),
        page_id: i as i32,
        //pagerank: None,
        pagerank: ranks.get(&i).map(|f| *f),
    }).collect();
    diesel::insert(&rows).into(titles::table).execute(conn)
}


//  ---------LINKS-MGMT--------


fn purge_cache(conn: &PgConnection) -> Result<usize, diesel::result::Error> {
    use self::schema::paths::dsl::paths;
    use self::schema::paths::dsl as path_row;
    println!("WARNING: purging cache");
    //better way to select all?
    diesel::delete(paths.filter(path_row::src.gt(-1))).execute(conn)
}
