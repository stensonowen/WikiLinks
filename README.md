# WikiLinks
## Store the structure of links of the entire English Wikipedia in a hash table and finds the shortest path from one to another.

### Rust Port:

After leaving this project in a mostly finished state for a year or so I decided to pick it back up because I had some regrets. For example, I was worried that there was a data race somewhere in the table population, but was unable to find it (which didn't totally assuage my worries). I also realized in the middle of the project that Wikimedia publishes SQL dumps for data such as page links, meaning I might be able to retrieve the information I needed without needing to download a 13GB compressed dump and parsing a 50GB uncompressed dump. I also wanted to explore populating some of the table data at compile time instead of run time. 

Rust seems like an interesting candidate for some of these new priorities: it is built to prevent data races and I wouldn't have to much around with template metaprogramming. I saw an [interesting crate](https://crates.io/crates/phf) featured in the [24 Days of Rust](https://siciarz.net/) blog and wanted to try it out.

