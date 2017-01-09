extern crate pagerank;
extern crate wikidata;

fn main() {
    pagerank::wikidata_pageranks("simple_pageranks.csv");
}
