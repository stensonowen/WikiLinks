## Current Status

Currently the fastest form of the project is a pure Rust rewrite of parsing, storing, searching, computing pageranks, and serving web requests. 

The link data is parsed from sql dumps published monthly, adapting for redirects and dead pages. It is saved to disk and loaded into a hash table where the [pagerank](https://en.wikipedia.org/wiki/Pagerank) of each page can be computed and saved. A bidirectional breadth-first search can then be run on the link table, searching the children and grandchildren of the source and the parents and grandparents of the destination until the sets intersect, a process which completes relatively quickly in the worst case even with an ugly order notation. Searches are cached using [Postgres](https://www.postgresql.org/) with [Diesel](https://diesel.rs), and [Rocket](https://rocket.rs) processes requests to serve up saved searches or perform new ones.

Overall this makes for a pretty fast website. Storing the table in memory only fills about 6GB. The longest breadth-first search I've been able to run finished in a few seconds. Parsing the sql dumps and computing pageranks take about one and twelve hours respectively, but both can be improved and only need to be run once (or zero times) per month. Table population completes in one minute, and hasn't yet been multithreaded. 

Searches are fast and the process seems to scale well, but it can always be better.

### Usage

Downloading the relevant databases can be automated by running `retrieve.sh` (which might require tweaking some variable values), or by downloading manifests/data that I might remember to host.

Pretty much all steps of the process, from parsing the databases to hosting the site, can be accomplished with the primary `links` crate. Various steps can be skipped by importing a manifest that you previously exported or downloaded, but for a recent data dump you might have to parse your own. Parsing the data, computing pageranks, and exporting everything to be imported later can be done with:
```
cargo run --release -- \
--page.sql ~/wikidata/simplewiki-20170201-page.sql \
--redirect.sql ~/wikidata/simplewiki-20170201-redirect.sql \
--pagelinks.sql ~/wikidata/simplewiki-20170201-pagelinks.sql \ 
--export_links ~/wikidata/links.json \
--compute_ranks \
--export_md ~/wikidata/metadata.json
```
Starting up the server is much faster after this.
```
ROCKET_ENV=production cargo run --release -- \
--import_links ~/wikidata/links.json \
--import_md ~/wikidata/metadata.json
```

## Original Approach: Python and C++

Early on, the process of loading links and seeking paths was quite straightforward. It was divided into three real components. 

1. A python script would parse the wikipedia dump's text file, looking for the pattern that would indicate an internal link, (e.g. `... page content with a [[link article title|link friendly name]]...`). The script would produce a text file associating each article title with the titles of all of its children. This was useful because parsing was somewhat expensive; I/O is a bottleneck, so cutting out most of the text that would be ignored anyway is helpful. This process isn't ideally efficient, taking about half an hour on one core for the full English dump, but it only needs to be run once per dump. 

2. The next step was table generation. To make lookups fast, everything is stored in a pretty minimalistic hash table, mapping an article title to a struct containing its title and children. So for each parent/child pair of titles in the python parse data, each string was hashed and looked up (and inserted if necessary) to get its associated index. Then the collection of children at `table[parent_index]->children` was updated to include the child index. This way each string only needs to be stored once, and all of the links can be represented as 32-bit integers, which are small and cheap to operate on.

3. Performing a breadth-first search from one index to another is just a matter of expanding the pool of reachable indices until it includes the destination index. If `dst_index` is a child of `src_index`, then the search is over; otherwise each of `src_index`'s children is added to a pool of seen indices. 
For each element in this pool (i.e. for each child of `src_index`), if `dst_index` is among its children, then the search is over: there might be other equally long searches, but there can't be any shorter ones; otherwise all of its children (which are `src_id`'s grandchildren) that have not been seen before are added to a new pool of descendants to be searched, repeating until the `dst_index` is found or a max iteration is reached. This effectively populates sets of indices that are reachable in no more than `n` iterations for increasing values of `n`. It is as efficient as can be expected, only storing what it needs (a set of the `n`th generation of descendants and a set of all other reachable indices) and not entering into any redundant cycles. This doesn't scale particularly well: articles can have several hundred children (the average is probably around ~80), so that means storing 80<sup>i</sup>; a search of depth 10 could require searching through <i>billions</i> of integers. Fortunately, most searches are much shorter than this, but it meant that searches could have wildly unpredictable run times, from milliseconds to minutes.

### Speed/Memory optimizations

There were a handful of ways I sped this process up over the course of a few months. It initially had almost comically bad resource consumption, requiring an ~80GB pagefile and ~13 hours to load into memory (from the end of the python parsing step to the beginning of the first search), but this improved drastically after heavy refactoring and profiling. 

* Refactoring was the first major step; it didn't speed things up per se, but it turned the codebase from a mess of delicate unreadable garbage into something much easier to hack on.
* Switching from using a linked list to store children indices to a vector. My initial thought process was that the hash table took up lots of contiguous memory so some non-contiguous memory would lead to better usage of RAM. In reality, it just turned every 32-bit integer into a list node that also contained two 64-bit pointers, and it killed any semblance of cache locality populating/searching could have used. I was fresh out of data structures; can you blame me?
* Tweaked python parser to also output the number of children for each article. This allowed the table population step to perfectly allocate storage for every vector of children, leading to fewer reallocations and less unnecessary overhead.
* Reduced a layer or two of indirection in the entry struct and hash table. I hadn't given this much consideration while initially writing the program, but this helped to reduce the number of cache misses and unecessary pointer storage.
* Switched from running inside MS Visual Studio to compiling with gcc and running on linux. I didn't do this before because I didn't have a machine running linux with enough memory (and I thought swap files wouldn't handle the load as well as a pagefile had), but when I finally fixed it up to compile in gcc there was a large performance improvement. I figured running in Visual Studio probably hooked memory allocations (or maybe the windows allocator is less suited to this than libc's `malloc`?), but population times went from minutes to seconds.
* I parallelized the table population process. Because data loaded by the table was the output of my own python script, it was easy enough to split the data into multiple chunks that could all be read concurrently (more on this below).

Overall, this worked quite well. Some of these changes had drastic effects on memory consumption, which eliminated the need for a pagefile and increased the effects of caching, in turn drastically improving the population time: at the end of the day this step took about five minutes and under 16GB of RAM.

### Thread Safety

For a while, populating the hash table by reading in the output of the python parser was excruciatingly slow. This was mitigated somewhat using many of the memory optimizations above, and I was always planning to sacrifice population time for search time, but I wanted to improve this anyway.

However, parallelizing the table population step seemed unavoidably expensive: assigning a mutex to every element seemed like the safest option but would add an unacceptable amount of overhead, and playing around with atomics hadn't really gone anywhere. Using a single mutex to lock the table would work very poorly because of the number of lookups that were constantly being done: resolving collisions during a table lookup would frequently require several reads (to compare the title stored at an entry to the title being hashed), each of which would require a mutex lock.

Instead of locking the whole table with one mutex or storing a mutex for every slot in the table, there is a middle ground. Instead, there is an array of about `m` (about 100) mutexes that are tasked with protecting a table of 15 million elements; whenever an index `i` needs to be read or written, mutex `i % m` is locked. Because there are so many more entries than mutexes collisions are far from impossible, but the time delay is negligible: at most it's the delay of writing a value to an entry. However, because only 4 threads (or however many cores are being used) will be running at a time, there is a high probability that they will be locking 4 different mutexes, meaning all threads can operate effectively in parallel, with only the overhead of semi-frequent locks/unlocks. 

### Shortcomings

The biggest issue with this iteration of the project was data integrity. I had noticed some inconsistencies and what I suspected were longer than necessary paths, from which worries about thread safety arose. Someone recommended that as long as I had this giant table of internal wiki links I should compute the pageranks, but this was difficult or impossible because of redirects. 

At the time a search could be choked by using an anomalous page for the destination. For example, the wikipedia page [USA](https://en.wikipedia.org/w/index.php?title=USA&redirect=no) redirects to the page for the [United States](https://en.wikipedia.org/wiki/United_States), but that information was not present in the full text dump being given to the python script, so the page table had no way to realize or represent this. A search for "USA" in the destination would search for an article that incorrectly linked to "USA" and relied on the site to handle the redirect. Obviously this was both inefficient and unfaithful to the actual data, but it was a niche case (and kind of a funny one).

The project was functional enough to host, but only for a small group of people. It used [crow](https://github.com/ipkn/crow) as a web framework, which was fun to play around with, but wasn't well documented and made it difficult to effectively parallelize the server.

## Rust rewrite

### Why Rust

I was worried some of the anomalies in searches were the result of a data race, which is what initially pushed me to leave C++ for Rust. I had also seen some interesting crates I wanted to try out (such as [phf](https://crates.io/crates/phf), which I unfortunately had to ditch). But the biggest reason is that Rust is cool, and I like writing it more than C++ (or perhaps more accurately I dislike debugging Rust less than C++). 

### Addressing Shortcomings

Because it was a ground-up rewrite I was free to fix some problems in the C++ version that initially would have required ugly hacks.

One of the biggest differences is how input is handled. Instead of a python script to strip Parent-Child title pairs from the text dump, the Rust version parses article metadata from a series of sql dumps published every month: `page.sql`, `redirects.sql`, and `pagelinks.sql`. Now, instead of mapping article titles to hashes that change every time the program is run, articles are associated with their true `page_id`, which is simpler and easier to debug (though it means a bit of unnecessary overhead in the form of a table mapping article titles to their `page_id`). This led to quite a meaningful performance improvement, as it allowed consolidating true links with their redirects and didn't have the shortcoming of including pages linked from the english wiki to another language wiki.

The other major difference is the breadth-first search itself. Rather than only storing child indices and searching until it finds a parent of the destination index, the Rust version stores both children and parent collections; when searching, it populated trees of what unseen nodes are reachable from the source's children/grandchildren/etc. and what unseen nodes are reachable from the destination's parents/grandparents/etc. until the sets intersect. This has the same exponential order notation, but the search depth is effectively halved for a linear cost, which goes a long way.

### State Machine Pattern

One of the biggest purposes this project serves for me is serving as a problem I'm very familiar with that I can use to test out new technologies; it started as an excuse to play around with Visual Studio Community Edition when it came out, as well as hash tables and pagefiles, which I had just learned about, and then evolved to experiments concurrency, then with Rust, then [phf](https://crates.io/crates/phf), and so on.

One of the more recent ideas I wanted to play around with was a [Rust state machine pattern](https://hoverbear.org/2016/10/12/rust-state-machine-pattern/); such state machines utilize Rust's trait system to verify at compile time that they can't be operated upon illegally. Even though the `LinkState` struct only goes through four sequential states, it's useful that this tool can verify everything is done in the right order. Even though it might be overkill in this situation, it provides yet another data integrity guarantee at compile time and provides a nice structure for organizing the codebase. 

### Pageranks

After creating a lookup table of articles and their children that took redirects into account, I couldn't not compute the pageranks of the articles. The [PageRank algorithm](https://en.wikipedia.org/wiki/Pagerank) was initially used by Google as a metric of centrality of web pages based what pages they link to and the rank of pages that link to them. Each value is between 0 and 1, and can be thought of as the probability that a user who clicks on on links randomly with an 85% chance of clicking again will land on that page. As far as I can tell, the way pagerank is usually computed on very large data sets is with some funky linear algebra that I don't understand, so instead my implementation takes a linear approach. It isn't as fast as it could be, but I was prioritizing makeing the code as straightforward as possible to reduce the possibility of errors. Currently it takes about twelve hours to run on the full English wikipedia (~5 million entries, ~80 links each) with a maximum error of 0.00000001, but that figure was probably exacerbated by the fact that the process was using required swap space (and of course the process only has to be run once per monthly dump).

Currently the pagerank data doesn't serve much of a purpose other than being interesting. There is a `rank` field in the postgres cache database which is currently used to sort suggestions when a search query does not match a page title, but that's pretty unnoticable. I'll probably do some data visualization project with the data or something.

For now a preview of the highest ranks of the simple wikipedia can be found [here](https://gist.github.com/stensonowen/25df4124c1509a7033c5e1553c404a47) and of the english wikipedia can be found [here](https://gist.github.com/stensonowen/d72342cbee65893e03faf46eb77b2adb). 

### Security

I didn't spend much time implementing security measures or testing the website, but I think it works out fine anyway. Potential issues I've thought about are:

* XSS with article searches: I think this is theoretically possible, considering the page renders html based on user-defined input. I'm not sure if Tera/Rocket make that harder/impossible. The reason this isn't an issue is that user html is only rendered if it matches a valid wikipedia page; therefore if you wanted to do any vandalism/cookie stealing, you'd first have to edit wikipedia to contain an article whose title is your payload. 
* SQL injections used to be simiparly possible: the C++ version used to use a small wrapper that I wrote to construct SQL queries, so it was definitely vulnerable. However, the database would only be queried <i>after</i> the source and destination titles were valid, so again, any injection payload would have to be a valid wikipedia article title. If someone can find/make some such vulnerability then they deserve to vandalize the database. However, this is no longer an issue, as the Rust version uses the diesel ORM, which should make sql injections very difficult or impossible.
* Hash table Dos: Rust hashmaps by default use a randomized hashing algorithm to make it harder for an adversary to maliciously craft queries to kill performance; the biggest hash table I employ uses [fnv](https://crates.io/crates/fnv) instead. Fnv is notably faster (at least for small inputs), and the only way users can maliciously craft page entries is, again, by modifying the dumps to collide several popular pages or something.

### Future Work

Optimizations:

* Reduce cache misses by inlining parents/children in ?Sized structs
* Use faster hashing via bitwise operations
* Refactor parsing module
* Rewrite perfect hashing crate
* Parallelize rank computation
* Parallelize link importing/exporting
* Performance profiling
* Experiment with Bloom filters in bfs

Features:

* Proper fuzzy searching 
* Incorporate pagerank somehow


