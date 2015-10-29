/*
Load link structure of Wikipedia into custom hash table
Easily find shortest path between any two articles
Requires parsed Wiki dump as input (use parsr8.py)
Written by Owen Stenson, Summer/Fall 2015
*/

/*
Run in 64-bit to use >2GB of memory; 
Only tested with pagefile (windows); unknown useability with swap instead (linux)
Linux swaps process memory while Windows pages memory frames; works with Windows, unknown with Linux
*/

//IO
#include <iostream>
#include <fstream>
//Structures
#include <string>
#include <vector>
#include <list>
#include <map>
#include <set>
//Tools
#include <cassert>
#include <time.h>
#include <math.h>
#include <algorithm>
#include <iomanip>
//Bug fixes / compatibility
#include <tr1/functional>   //fixes g++ issue in which std::hash not found
#include <stdlib.h>         //atoi

#define MAX_DEPTH 10

using namespace std;

struct entry {
	//sizeof(entry) = 32 bytes in 64-bit
	string *url;			//holds url: (to check for collisions)
	list<unsigned int> links;		//list of hashes
};

unsigned int resolve_collisions(const string &str, entry ** table, unsigned int table_entries, tr1::hash<string> &str_hash, int &collisions, bool verbose=false) {
	//Employ hash function and then use custom collision-resolving algorithm
	/* Deal with collisions by retrying with an offset of n!+1;
	Should be slightly more successful than an offset of n^2 because it generates primes very frequently (prime for 0<=n<=4, and then ~50% for n>4).
	Evades the performance hit of factorials because it only finds one product per attempt, which it stores in memory.
	Thus, rather than O(n!) additional cycles, it only requires one int and two addition operations (4 bytes, <=2 cycles)	*/
	unsigned int hash = (str_hash)(str) % table_entries;
	int offset = 2;
	unsigned int multiplier = 1;
	for (int i = 0; i < 100; i++) {
		hash %= table_entries;
		if (table[hash] == NULL || *(table[hash]->url) == str) { return hash; }
		else { collisions++; }
		offset = (offset - 1)*multiplier + 1;
		if (offset == 1) {
			//offset gets stuck at 1 and never changes. Does it ever reach 1?
            cout << "\tWarning: collision offset at 1" << endl;
		}
		multiplier++;
		hash += offset;
		if (verbose) {
			std::cout << "  Trying hash " << hash << "..." << endl;
			if (table[hash] == NULL) std::cout << "  No entry found at hash " << hash << ";" << endl;
			else std::cout << "  Entry '" << *(table[hash]->url) << "' found at hash " << hash << ";" << endl;
		}		
	}
	if (verbose) std::cout << "   Didn't find any blank entries in k iterations;" << endl;
	assert(false);
	//if this assertion is hit, then something is wrong: 
    //table size should be increased (or for() loop limit should be)
	return -1;	//will break something if it's ever returned
}

void create_entry(unsigned int hash, string *url, entry ** table, list<unsigned int> *links = NULL) {
	//make a new entry from the given details; 
	table[hash] = new entry;
	table[hash]->url = url;
	if(links) table[hash]->links = *links;
}

void print_dbg_info(entry ** table, set<unsigned int> *link_tree_rest, map<unsigned int, list<unsigned int>*> *link_tree_row, map<unsigned int, list<unsigned int>*> *link_tree_new_row) {
	//debug: print contents of link_tree_rest, link_tree_row, link_tree_new_row
	std::cout << "\n\n\t" << "Top half of the tree:" << endl;
	for (set<unsigned int>::iterator tmp_itr = link_tree_rest->begin(); tmp_itr != link_tree_rest->end(); tmp_itr++) {
		std::cout << "\t\t" << *tmp_itr << "  =  " << *(table[*tmp_itr]->url) << endl;
	}
	std::cout << "\t" << "Bottom row of the tree (old):" << endl;
	for (map<unsigned int, list<unsigned int>*>::iterator tmp_itr = link_tree_row->begin(); tmp_itr != link_tree_row->end(); tmp_itr++) {
		std::cout << "\t\t" << tmp_itr->first << "  =  " << *(table[tmp_itr->first]->url) << endl;
	}
	std::cout << "\t" << "Bottom row of the tree (new):" << endl;
	if (link_tree_new_row != NULL) {
		for (map<unsigned int, list<unsigned int>*>::iterator tmp_itr = link_tree_new_row->begin(); tmp_itr != link_tree_new_row->end(); tmp_itr++) {
			std::cout << "\t\t" << tmp_itr->first << "  =  " << *(table[tmp_itr->first]->url) << endl;
		}
	}
	else {
		std::cout << "\t\tblank" << endl;
	}
	std::cout << "\n" << endl;
}

void clean_up_search_mem(set<unsigned int> *link_tree_rest, map<unsigned int, list<unsigned int>*> *link_tree_row, map<unsigned int, list<unsigned int>*> *link_tree_new_row) {
	//clean up set
	if (link_tree_rest) {
		delete[] link_tree_rest;
		delete link_tree_rest;
	}
    map<unsigned int, list<unsigned int>*>::iterator tmp_itr;
	//clean up first map
	if (link_tree_row) {
		for (tmp_itr = link_tree_row->begin(); tmp_itr != link_tree_row->end(); tmp_itr++) {
			delete[] tmp_itr->second;
		}
		delete[] link_tree_row;
		delete link_tree_row;
	}
	//clean up second map
	if (link_tree_new_row) {
		for (tmp_itr = link_tree_new_row->begin(); tmp_itr != link_tree_new_row->end(); tmp_itr++) {
			delete[] tmp_itr->second;
		}
		delete[] link_tree_new_row;
		delete link_tree_new_row;
	}
}

pair<list<unsigned int>*, int> seek_links(unsigned int source, unsigned int destination, entry ** table) {
	//from table[source], find shortest path to destination by traversing links
	//essentially a breadth-first search of tree
	//returns a list of the hashes to click in order and an error code
		//code = 0	:	success
		//code = -1	:	no way to reach destination from source
		//code > 0	:	search expired after n iterations
	
	//use map to track already checked options
		//key = hash
		//value = link structure of history of retrieval (does it have to be a pointer?)
		//should be 2 maps: bottom row in link tree and everything else (because cycling through bottom row creates new bottom row, which insert()s while iterating through
		//map is helpful because duplicates are bad and searching must be fast
	
	//map contains every item in bottom row of link tree; must be 2 because cycling through link_tree_row inserts new entries into itself
	map<unsigned int, list<unsigned int>*> *link_tree_row = new map<unsigned int, list<unsigned int>*>;
	map<unsigned int, list<unsigned int>*> *link_tree_new_row = new map<unsigned int, list<unsigned int>*>;
	//contains all other items in tree: must have record of what links have been traversed to avoid redundancy
	set<unsigned int> *link_tree_rest = new set<unsigned int>;

	map<unsigned int, list<unsigned int>*>::iterator entry_itr;	//to cycle through row
	list<unsigned int> node_links;				//store a hash's link linked list
	list<unsigned int> *parent_path = NULL;		//store link's parent's path, to branch out and add onto
	list<unsigned int> *child_path = NULL;		//tmp var for creating link paths from their parents (parent + new link = child)
	pair<unsigned int, list<unsigned int>*> *link_entry;	//to reference entry without relocating it in table
	
	if (table[source]->links.empty() == false) {
		//to start, insert all of the source's links into the structure (bottom row)
		//this prevents the source from being stored in all of the path lists, which is redundant (because it's already stored elsewhere) and expensive
		node_links = table[source]->links;
		for (list<unsigned int>::iterator link_itr = node_links.begin(); link_itr != node_links.end(); link_itr++) {
			if (*link_itr != source) {
				//insert link if it's different from the source, to prevent a few remotely possible redundancies
				if (*link_itr == destination) {
					return pair<list<unsigned int>*, int>(NULL, 0);
				}
				link_entry = new pair<unsigned int, list<unsigned int>*>(*link_itr, new list<unsigned int>);
				link_tree_row->insert(*link_entry);
			}
		}
	}
	else {
		std::cout << "table[source]->links was empty; skipping..." << endl;
	}


	//start loop between rows within tree (10 layers deep is probably enough)
	//increasing the max depth is possible, but not recommended because this thing scales horribly 
	for (unsigned int i = 0; i < MAX_DEPTH; i++){
		//start loop between items in row
		for (entry_itr = link_tree_row->begin(); entry_itr != link_tree_row->end(); entry_itr++) {
			parent_path = entry_itr->second;
			node_links = table[entry_itr->first]->links;	//copy this entry's links to a var to insert them into the map
			//start loop between links on a page
			for (list<unsigned int>::iterator link_itr = node_links.begin(); link_itr != node_links.end(); link_itr++) {
				//add this link to new row of tree iff it isn't present already
				if (link_tree_rest->find(*link_itr) == link_tree_rest->end()) {
					//to add entry to the tree, a new path must be generated by appending the parent's value to the parent's path
					child_path = new list<unsigned int>(*parent_path);
					child_path->push_back(entry_itr->first);
					//if this link is to the desired page, then return it
					if (*link_itr == destination) {
						//clean_up_search_mem(link_tree_rest, link_tree_row, link_tree_new_row);
						return pair<list<unsigned int>*, int>(child_path, 0);
					}
					link_tree_new_row->insert(pair<unsigned int, list<unsigned int>*>(*link_itr, child_path));
				}
			}
			//can clean up parent_path, because all children have copied from it
			delete parent_path;
		}
		if (link_tree_new_row->empty()) {
			std::cout << "There is no way to get to the destination from the source" << endl;
			//clean_up_search_mem(link_tree_rest, link_tree_row, link_tree_new_row);
			return pair<list<unsigned int>*, int>(NULL, -1);
		}
		//move every key from bottom row into top half (so a new bottom row can be started)
		for (entry_itr = link_tree_row->begin(); entry_itr != link_tree_row->end(); entry_itr++) {
			link_tree_rest->insert(entry_itr->first);
			//delete[] entry_itr->second;
		}
		//link_tree_row now holds contents of link_tree_new_row, and link_tree_new_row gets reset to make room
		swap(link_tree_row, link_tree_new_row);
		link_tree_new_row->clear();
	}
	std::cout << "The search exceeded its maximum depth; this can be increased, but it is expensive" << endl;
	//clean_up_search_mem(link_tree_rest, link_tree_row, link_tree_new_row);
	return pair<list<unsigned int>*, int>(NULL, MAX_DEPTH);
}

 
int main(int argc, char* argv[]) {
	//parse command line arg
	if (argc != 2) {
		cout << "Usage: \"" << "WikiLinkr.exe" << " path_to_parsed_file.txt\"" << endl;
		cout <<"(You must first run \"python parsr8.py path_to_wikipedia_dump.xml path_to_parsed_file.txt\")" << endl;
		exit(1);
	}
	ifstream in_file(argv[1]);
	
	//start timer:
    clock_t t = clock();

	
	tr1::hash<string> str_hash;	//initialize string hash function (better tailored to strings than bj or djb2 are)
	unsigned int hash;
	/*	Initialize hash table: Structure should be a contiguous array of pointers to structs
	*			structs should hold url to compare (collision checking) as well as link structure
	*			address should be a hash of the url
	*			starting address should be ~100x expected size?
	*			should use list to hold links (vectors must be contiguous?)
	*			64-bit programs mean 16-bit addresses
	*/
	
	//for analytics:
	int collisions = 0;
	unsigned int total_articles;
	string total;
	int progress;
	unsigned int article_counter = 0;

	//table info:
	unsigned int table_entries = -1;

	//open file for reading
	if (!in_file) {
		std::cout << "Failed to open input; exiting" << endl;
		exit(1);
	}

	//retrieve new article cound from beginning of input:
	getline(in_file, total);
	//total_articles = stoi(total);
    //stoi requires -std=c++11 flag
    total_articles = atoi(total.c_str());

	std::cout << "Initializing structure..." << endl;
	table_entries = 20 * total_articles + 1000;	//tentative equation
    //5,000,000 works well for simple english wiki (210,083)
    //100,000,000 works well for complete english wiki (15,819,375)

	entry ** table = new entry*[table_entries];
	for (unsigned int i = 0; i < table_entries; i++) {
		//NULL out all entries
		table[i] = NULL;
	}
	
	//start cycling through file:
	std::cout << "Started reading..." << endl;
	cout << " 0% \t[                                                  ]";
	
	//reading vars
	string *title = NULL;
	string line;
	int link_hash;
	list<unsigned int> *links = NULL;
	
	while (getline(in_file, line)) {
		//process line-by-line
		if (line == "<page>") {
			//just finished reading in links; insert data into table
			if(title != NULL){
                //source of bug: duplicates in parsed file overwrite original entry
				hash = resolve_collisions(*title, table, table_entries, str_hash, collisions);
				
                if(table[hash] == NULL){
                    create_entry(hash, title, table, links);
                } else {
                    table[hash]->links.splice(table[hash]->links.end(), *links);
                }
				
                article_counter++;
				if (article_counter % (total_articles / 100) == 0) {
					//print progress:
					progress = article_counter / (total_articles / 100);
					cout << '\r' << flush << "~" << progress << "% \t[";
					for (int i = 0; i< progress / 2; i++) cout << "=";
					for (int i = 0; i< 50 - progress / 2; i++) cout << " ";
					cout << "]";
				}
			}
			title = new string;
			links = new list<unsigned int>;
			//about to show article metadata
			getline(in_file, *title);
		}
		else {
			//line is a link: create if necessary and store it
			link_hash = resolve_collisions(line, table, table_entries, str_hash, collisions);
			if (table[link_hash] == NULL) {
				//if link didn't exist, create it 
				create_entry(link_hash, new string(line), table);
			}
			links->push_back(link_hash);
		}
	}
	//insert last article data into table
	hash = resolve_collisions(*title, table, table_entries, str_hash, collisions);
	create_entry(hash, title, table, links);
	in_file.close();
	
	cout << '\r' << flush << "100% \t[==================================================]" << endl;
	std::cout << endl << endl;
	std::cout << "Done indexing; " << collisions << " collisions" << endl;
	unsigned int entries = 0;
	unsigned int blanks = 0;
	for (unsigned int i = 0; i < table_entries; i++) {
		if (table[i] == NULL) {
			blanks++;
		}
		else {
			entries++;
		}
	}
	std::cout << "Found " << entries << " populated slots, " << blanks << " unpopulated." << endl;
	std::cout << "With " << table_entries << " slots, that is " << float(entries) / table_entries * 100 << "%" << endl;

	//delete[] table;
	t = clock() - t;
	std::cout << "Total time: " <<  ((float)t) / 1000 << " seconds." << endl << endl << endl;

	
	std::cout << "To exit, leave either field blank.\n" << endl;
	string source = "-1", dest = "-1";
	unsigned int source_hash, dest_hash;
	pair<list<unsigned int>*, int> search_results;
	int pad_length = log10(table_entries) + 1;
	
	while (true) {
		cout << " Enter source: \t\t";
		getline(cin, source);
		cout << " Enter destination: \t";
		getline(cin, dest);
		
        if (source.empty() || dest.empty()) {
            break;
        }
        //capitalize inputs:
        transform(source.begin(), source.end(), source.begin(), ::toupper);
        transform(dest.begin(), dest.end(), dest.begin(), ::toupper);


        t = clock();
        source_hash = resolve_collisions(source, table, table_entries, str_hash, collisions);
        dest_hash = resolve_collisions(dest, table, table_entries, str_hash, collisions);
        if (table[source_hash] == NULL || *table[source_hash]->url != source || table[source_hash]->links.empty()) {
            cout << "\nCouldn't find source article \"" << source << "\"; it seems it didn't exist in the Wiki dump this is using.\n" << endl;
            continue;
        }
        else if (table[dest_hash] == NULL || *table[dest_hash]->url != dest) {
            cout << "\nCouldn't find destination article \"" << dest << "\"; it seems it didn't exist in the Wiki dump this is using.\n" << endl;
            continue;
        }
        else {
            search_results = seek_links(source_hash, dest_hash, table);
            if (search_results.second == 0) {
                cout << "\n\nFound path from " << *table[source_hash]->url << " (" << source_hash << ") to " << *table[dest_hash]->url << " (" << source_hash << ")" << endl;
                cout << "\t" << setw(pad_length) << source_hash << "  =  " << *table[source_hash]->url << "*" << endl;
                if (search_results.first) {
                    for (list<unsigned int>::iterator tmp_itr = search_results.first->begin(); tmp_itr != search_results.first->end(); tmp_itr++) {
                        cout << "\t" << setw(pad_length) << *tmp_itr << "  =  " << *table[*tmp_itr]->url << endl;
                    }
                }
                cout << "\t" << setw(pad_length) << dest_hash << "  =  " << *table[dest_hash]->url << "*" << endl;
            }
            else if (search_results.second == -1) {
                cout << "Confirmed that no path exists between " << *table[source_hash]->url << " (" << source_hash << ") to " << *table[dest_hash]->url << " (" << source_hash << ")" << endl;
            }
            else {
                cout << "Search between " << *table[source_hash]->url << " (" << source_hash << ") to " << *table[dest_hash]->url << " (" << source_hash << ")" << " failed after " << search_results.second << " iterations." << endl;
            }
            t = clock() - t;
            cout << "Total time: " << ((float)t) / 1000 << " seconds." << std::endl << endl << endl;
        }
	}

    int a = 0;
	for (unsigned int i = 0; i < table_entries; i++) {
		if (table[i]) {
			delete table[i]->url;
			//delete[] table[i];	//?
			delete table[i];	//?
            a++;
		}
	}
	delete[] table;
    cout << "Deleted " << a << " urls/entries, freeing up " << a*sizeof(entry) << " bytes" << endl;
	//delete table;
	return 0;
}


/*TODO
	Clean up memory after a search
	Clean up memory at the end of the program
	Parsr sometimes includes duplicate entries: should combine rather than replace
*/
