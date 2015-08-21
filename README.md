# WikiLinkr
## Stores the structure of links of the entire English Wikipedia in a hash table and finds the shortest path from one to another.

### Background / Overview
The project is somewhat inspired by [the wikipedia game](https://en.wikipedia.org/wiki/Wikipedia:Wiki_Game), the goal of which is to get from one article to another only by clicking links. I've also been trying to find interesting ways to implement cool data structures and stuff learned in class. This seemed like a good candidate for a hash table and a good excuse to play around with pagefiles. 

Parsr8.py reads in a Wikipedia dump and outputs a file containing only the article names and their links. main.cpp reads through that file and creates a hash map containing a linked list of hashes of articles its entry links to. (For example, the article title "DNA" might hash to 123456, so an entry containing its name would be created at table[123456]. The article for DNA might link to the article RNA, which hashes to 987654, so table[123456]->links should include 987654, and so on.) It is helpful to use a hash table because it is important that this structure scale well, as it holds tens of millions of unique articles (in the worst case). 

The seek_links() function is essentially a breadth-first tree search algorithm with lots of nodes. Article hashes and their lists of links are stored (as the keys and values respectively) of a map to avoid duplicates. In each of a limited number of iterations, the articles in this map are moved to a set (to keep track of them so that duplicates are not searched again later), and each of their link hashes are compared to the sought article hash and then added to a new map, which is used in the next iteration. This scales pretty terribly (iteration *n* has to search and store something like 300^n links), but I'm pretty confident it can't be done any faster (at least without eating up tons of memory).

I was going to use a third-party parser to sort through the irrelevant noise from a Wikipedia dump (the only thing that matters is the link structure, which is about 10% of the dump),  but couldn't find any that both did what I needed and didn't crash all the time. I wrote my own parser in Python because populating the hash table from a parsed file is much less expensive than parsing a dump every time the program is run; this way a dump only needs to be parsed once. I wrote the parser in Python because it was initially going to be able to update dumps from the internet natively (and parsing also just lends itself to being written in Python), but that never happened and I never rewrote it in C++. I also wrote a pretty sloppy multithreaded version, but the new overhead was greater than the speedup, and it didn't matter much because parsing takes much less time than populating the table (~20 minutes for English Wiki, ~15 seconds for Simple Wiki).

Trying to run this using the complete English collection of Wikipedia articles is interesting, but not very practical. It requires about 85GB of memory, which at the moment must be accomplished by resizing the pagefile (unless you have an absurd amount of RAM). This is a significant bottleneck, though, so it takes about 12 hours to fully load the table from the parsed file (onto an SSD, no less). This only adds a small benefit too, because looking for articles through several layers of links is prohibitively expensive (therefore it's only practical to search from/for bigger, more popular articles). 

Instead, it is much more interesting to use the collection of Simple Wikipedia articles, which is significantly (about 100x) smaller. This dump takes about 15 seconds to parse, about 2 minutes to load into a table, and uses about 1GB of RAM, making it much more reasonable. Additionally, the Simple Wiki collection includes most common articles, which are going to be used most frequently in finding shorter paths, so the results are usually identical but much more accessible. This does mean it's impossible to search for very obscure articles, but that was impractical anyway because of how poorly the search algorithm scales.

### Quick Setup: 
The quickest (and probably smartest) way to get this up and running is to use the Simple Wiki xml dump. From the 'simplewiki' link [here](https://dumps.wikimedia.org/backup-index.html), download simplewiki-XXXXXXXX-pages-articles.xml.bz2, unzip it, and parse it with "python parsr8.py simplewiki-XXXXXXXX-pages-articles.xml simple_parsed.txt" (or use [an old, pre-parsed version](https://github.com/stensonowen/WikiLinkr/blob/master/misc_data/simple_parsed.txt)). Then pass that to the main.cpp project and give it a few minutes.


### General Benchmarks
For a general idea of how long this might take you (before you try the complete dump). 
System Specs: 
* CPU: i5 2400 @ 3.1GHz
* RAM: 8GB DDR3 @ 666MHz
* Primary Storage (pagefile): 224GB Crucial M500 SSD
* Secondary Storage (parsed dump file): Generic 1TB HDD (7200 RPM)

|              | Dump Size | Parsed Size | Parse Time | Populate Time | Populate RAM |
|--------------|-----------|-------------|------------|---------------|--------------|
|Complete Wiki | 52.7 GB   | 6.36 GB     | 24 min     | 12.4 hours    | 85 GB        |
|Simple Wiki   | .497 GB   | 67.5 MB     | 14 s       | 2.4 minutes   | 1.4 GB       |

The SSD was the bottleneck when populating the hash table for the complete wiki (because it was pretty much perpetually swapping), but the CPU bottlenecked everything else (neither parsr8.py nor main.cpp are multithreaded). The Simple Wiki version should be pretty accessible to most computers (I'll also upload the results of some examples for those who can't / don't care enough to set it up). 