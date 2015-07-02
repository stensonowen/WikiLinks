# WikiLinkr
Trying to store the structure of links of the entire English Wikipedia in a hash table.

I intended to upload this because MSVS 2015 just came out a week or two ago and I wanted to try out the Community edition, for which there is a GitHub extension. I've also been meaning to start using GitHub more regularly, rather than just as a form of backup and to show off my projects. 

The project is somewhat inspired by [the wikipedia game](https://en.wikipedia.org/wiki/Wikipedia:Wiki_Game), the goal of which is to get from one article to another only by clicking links. I've also been trying to find interesting ways to implement cool data structures and stuff learned in class. This seemed like a good candidate for a hash table and a good excuse to play around with pagefiles. 

Ultimately, this will read in the result of a Wikipedia parser (I've been looking at [this one](https://github.com/attardi/wikiextractor), but it's been pretty buggy), use regular expressions (probably) to identify which other articles each article links to, and store that data in a hash map. Eventually it might use a graph-like structure to find the shortest path from one article to another, and maybe generate cool data about which articles link to what things, but that's pretty far away at this point.
