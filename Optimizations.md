
This is a record of the attempts I've made to speed up various parts of this project.

### Representation

#### [Overlapping children/parents sets](0e3e7a73f006c9b31b45c662dc4effb6e56c4de9) (merged)

Each `Entry` in the table stored both its `children` (articles it links to) and its `parents` (articles that link to it) as a vector of `page_id`s. 

However, for many pages, these sets have some overlap. Intuitively, if article X links to article Y, then there is a higher probability that Y will link to X. In practice, there were about 2 million nodes that were being stored in both an entry's child and parent sets.

Therefore, instead of storing two distinct vectors `P` and `C`, consolidate all possible neighbors into three sets: parents but not children (`P \ C`), parents and children (`P ∩ C`), and children but not parents (`C \ P`). Then simply concatenate `(P \ C) + (P ∩ C) + (C \ P)` to store in a neighbor set, but also store the lengths of these components. Then, when reporting an entry's parents, simply return a slice of the first 2 components; when reporting children, return a slice of the second 2 components.

This adds the cost of storing 2 integers per entry, but overall the savings are still worth it. Interestingly, `u16`s turned out to be insufficient to store these indices because of the popularity of some searches (e.g. [IMDB](https://en.wikipedia.org/wiki/IMDB) has hundreds of thousands of parent entries).

This adds a little extra time to the initial parsing step, but doesn't slow down anything else.

Overall this reduced memory usage by about 15% without reducing the total number of edges. This is very helpful because it makes swapping much less likely.


#### [Store titles as hashes](f571109ceefea339c9463bd033034bc1c909ed8c) (merged) (for now)

One pain point of the interface was searching for articles: a misspelling or a capitalization mistake would turn up no results. We can't blindly capitalize all article titles because that causes some collisions where there shouldn't be any.

Fortunately, all of this information is available to us; because we handle all the parsing, it's easy to store redirect titles (e.g. "Richard Matthew Stallman" to "Richard Stallman") or legal capitalizations (e.g. "richard stallman" to "Richard Stallman").

Unfortunately, this consumes a lot of memory. I briefly deployed a demo in this state, and just a handful of people doing searches cased the server to hit 8GB of memory (90% of which was the main entry table) and swap.

Instead of storing 25M strings, we can store 25M hashes of strings. By my math (in the commit message), there's about a .002% chance of a collision, which is very acceptable. Even though this introduces the possibility of improper data being returned, any mistake would be very obvious to the user (if their query and the page actually being used were totally different).

This freed up about 200Mb for doing searches without sacrificing the helpful searching functionality. 

In the future I plan to investigate the memory consumption of the `fst` crate, which would allow better fuzzy searching (case-insensitive searching, regex matching, or Levenshtein distance quite easily) without the risk of collisions (negligible as it is).


### Breadth-first search

#### [Bloom filters](da56735e8ad241120ffaf2285f147405a1a3f853) (reverted)

A very common operation when performing a breadth-first search is checking whether a newly found node is reachable; e.g. if we discover node *x* by traversing the children and grandchildren of the source node, we need to check that (1) we haven't already seen it in `src`'s descendents, in which case it should be ignored, or (2) it isn't reachable in the ancestors of `dst`, in which case we've found a valid path.

Most of this was done with `HashMap::contains`, which was reasonably fast. 
However, it is such a common operation that speeding things up only slightly would go a long way. 
In particular, noting that the `false` case is much more common than the `true` case makes it seem like there is some unclaimed speedup on the table.

I tried implementing this as a sort of bloom filter with only one hash function (is that still a bloom filter?): just a large bitmap indicating the presence or absense of elements. 
Like all bloom filters, this can of course report false positives in the case of collisions.
In the case where the bloom filter reports that an element is present, we need to check in the hash table.
However, this should make it fast to rule out many cases so we can avoid many of the `HashMap::contains_key` calls.

Because collisions aren't any more expensive than they were before, we can make optimizations that increase the likelihood of collisions if they speed up average access times. For example, the hash function can be simple bit-twiddling or the table can be small enough to fit on the stack.

Note that this implementation didn't include any table resizing; it had a fixed size for all searches.

This doesn't end up working very well. For large searches, the collision rate gets quite high and the impact is relatively minimal (though some long searches saw about a 10% speedup). 
For small searches, the memory use became an issue: creating a large table on the heap caused a non-trivial time penalty and even smaller stack-based tables slowed things down (presumably due to polluting the cache with a bitmap instead of the hash table itself); most medium-to-small searches got slower.

Even though speeding up the worst case is important, shorter paths are the common case. 

This "optimization" was reverted. Creating a new construct to store in memory in addition to the hashmap is bad for caching. The inability to resize the structure doomed it to being mediocre in most instances. But some of these ideas could be used in the `IHMap`.

#### [Integer Hash Map](aae33d49455b4f10297454b31b5002f925eae72a) (merged)

I had seen a major (~20%) speedup when switching the bfs module to use the `fnv` crate instead of stdlib's `HashMap` (a non-randomized hashing function that's fast on small inputs is definitely better in this case). 
But we can simplify this hash table by making assumptions about our use case.

* Entries are never deleted
* Keys are always (relatively randomly distributed) `u32`s
* We care much more about the average case than the worst case

We can probably come up with a custom hash table implementation that takes advantage of these assumptions.

* We don't need to maintain a `was_here` bit on entries
* If the table capacity is always a power of two, hashing can be a simple bitmask
* A simple open-addressing strategy with linear probing is probably fine and will help with caching

Additionally, because this is for a specific use case and doesn't have to be general, we can improve memory consumption (and therefore spatial locality) by using a reserved value for a `None` entry instead of using an `Option<T>` (this is what the `optional` crate does, but I preferred to roll my own); because `page_id`s never exceed a value that's quite a bit less than 2^32, we can use key=INT\_MAX to indicate an entry is empty. (Currently our entry size is 2*32=64 bits, so adding an `is_some` or a `was_here` bit would approximately double our memory usage on 64-bit systems because of padding.) 

After some tweaking, I settled on using constants very similar to the stdlib for the table resizing threshold (`size/capacity > .5`) and the initial capacity (2**6=64).

On its own in some random benchmarking when compared against `fnv`, the custom hashmap was a few percent faster for lookups and 10-20% faster for insertions.

The real application saw a 20% speedup for small searches and up to 60% for medium to large searches, while using ~15% less memory.



