extern crate parse;

fn main() {
    let db = parse::populate_db();
    db.print();
}
