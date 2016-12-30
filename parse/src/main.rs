extern crate parse;

fn main() {
    let mut db = parse::populate_db();
    db.verify();
    db.print();
    println!();
    db.clean_up();
    println!();
    db.print();
}
