extern crate pagerank;
extern crate wikidata;

fn main() {
    let mut web = pagerank::Web::new();
    println!("Initial Sum:  {}", web.sum());
    println!(" i\t\tMax Diff\t\tSum");
    for i in 0..20 {
        let diff = web.iterate3();
        println!("{:03}:    {}, \t{}", i, diff, web.sum());
    }
}
