/*
Load link structure of Wikipedia into custom hash table
Easily find shortest path between any two articles
Requires parsed Wiki dump as input (parsr8.py)
Written by Owen Stenson, Summer 2015
*/

/*
Run in 64-bit to use >2GB of memory; Tested using ~10GB (w/ 8GB system RAM) of contiguous memory in x64 without issue
Only tested with pagefile (windows); unknown useability with swap instead (linux)

W/ link structure of hashes instead of strings, still takes ~2.5 minutes
	Uses more memory: ~1.3GB, which is 967420 entries, or ~92 capacity%, and 2M collisions
	Switch to vectors/arrays? Could use Parsr to pre-count elements

Benchmarks:
	Complete Wiki (52GB originally, 6GB parsed): table populates in ~13 hours
		Requires ~77GB additional swap space to store ~15.5 million new articles (excluding unmatched links)
		Finds articles seemingly instantly, populates correctly (as far as I can tell)
	Sample Wiki (74MB parsed): table populates in 2-2.5 minutes, requires ~1GB RAM
		Requires ~1GB RAM to store ~200k new articles (excluding unmatched links)		
*/

#include <iostream>
#include <fstream>
#include <string>
#include <vector>
#include <cassert>
#include <list>
#include <time.h>
#include <math.h>
#include <string>
#include <algorithm>	//capitalize
#include <map>
#include <set>
#define KILOBYTE 1024
#define MEGABYTE 1024*1024
#define GIGABYTE 1024*1024*1024

using namespace std;

struct entry {
	//sizeof(entry) = 8  bytes in 32-bit
	//sizeof(entry) = 16 bytes in 64-bit
	string *url;			//holds url: (to check for collisions)
	list<unsigned int> links;		//list of hashes
};

unsigned long djb2_hash(unsigned char *str) {
	//http://www.cse.yorku.ca/~oz/hash.html
	unsigned long hash = 5381;
	int c;
	while (c = *str++)
		hash = ((hash << 5) + hash) + c; /* hash * 33 + c */
	return hash;
}

int bj_hash(unsigned char *str)
{
	int h = 0;
	while (*str)
		h = h << 1 ^ *str++;
	return (h > 0 ? h : -h) % 33;
}


unsigned int resolve_collisions2(const string &str, entry ** table, unsigned int table_entries, hash<string> &str_hash, int &collisions, bool verbose=false) {
	//Employ hash function and then use custom collision-resolving algorithm
	/* Deal with collisions by retrying with an offset of n!+1;
	Should be slightly more successful than an offset of n^2 because it generates primes very frequently (prime for 0<=n<=4, and then ~50% for n>4).
	Evades the performance hit of factorials because it only finds one product per attempt, which it stores in memory.
	Thus, rather than O(n!) additional cycles, it only requires one int and two addition operations (4 bytes, <=2 cycles)	*/
	unsigned int hash = (str_hash)(str) % table_entries;	//size_t instead?
	int offset = 2;
	unsigned int multiplier = 1;
	for (int i = 0; i < 100; i++) {
		hash %= table_entries;
		if (table[hash] == NULL || *(table[hash]->url) == str) { return hash; }
		else { collisions++; }
		offset = (offset - 1)*multiplier + 1;
		//if (offset == 1) {
		//	cout << "!" << endl;
		//}
		multiplier++;
		hash += offset;
		if (verbose) {
			cout << "  Trying hash " << hash << "..." << endl;
			if (table[hash] == NULL) cout << "  No entry found at hash " << hash << ";" << endl;
			else cout << "  Entry '" << *(table[hash]->url) << "' found at hash " << hash << ";" << endl;
		}		
	}
	if (verbose) cout << "   Didn't find any blank entries in k iterations;" << endl;
	assert(false);
	//if this assertion is hit, then something is wrong: table size should be increased (or for() loop limit)
	return -1;	//should break something if 
}

void read_entry(const string &url, entry ** table, unsigned int table_entries, hash<string> &str_hash) {
	//Read entry info given from url
	int collisions = 0;
	unsigned int hash = resolve_collisions2(url, table, table_entries, str_hash, collisions);
	cout << "After " << collisions << " collisions:  ";
	if (table[hash] == NULL) {
		cout << "Entry " << &url << " is not present." << endl;
	}
	else {
		cout << "Entry " << url << " is present at 0x" << table[hash] << " and links to: " << endl;
		list<unsigned int> l = table[hash]->links;
		for (list<unsigned int>::iterator itr = l.begin(); itr != l.end(); itr++) {
			cout << "\t" << table[*itr]->url << endl;
		}
	}
}

void create_entry(unsigned int hash, string *url, entry ** table, list<unsigned int> *links = NULL) {
	//make a new entry from the given details; 
	table[hash] = new entry;
	table[hash]->url = url;
	if(links) table[hash]->links = *links;
}

void print_dbg_info(entry ** table, set<unsigned int> *link_tree_rest, map<unsigned int, list<unsigned int>*> *link_tree_row, map<unsigned int, list<unsigned int>*> *link_tree_new_row) {
	//debug: print contents of link_tree_rest, link_tree_row, link_tree_new_row
	cout << "\n\n\t" << "Top half of the tree:" << endl;
	for (set<unsigned int>::iterator tmp_itr = link_tree_rest->begin(); tmp_itr != link_tree_rest->end(); tmp_itr++) {
		cout << "\t\t" << *tmp_itr << "  =  " << *(table[*tmp_itr]->url) << endl;
	}
	cout << "\t" << "Bottom row of the tree (old):" << endl;
	for (map<unsigned int, list<unsigned int>*>::iterator tmp_itr = link_tree_row->begin(); tmp_itr != link_tree_row->end(); tmp_itr++) {
		cout << "\t\t" << tmp_itr->first << "  =  " << *(table[tmp_itr->first]->url) << endl;
	}
	cout << "\t" << "Bottom row of the tree (new):" << endl;
	if (link_tree_new_row != NULL) {
		for (map<unsigned int, list<unsigned int>*>::iterator tmp_itr = link_tree_new_row->begin(); tmp_itr != link_tree_new_row->end(); tmp_itr++) {
			cout << "\t\t" << tmp_itr->first << "  =  " << *(table[tmp_itr->first]->url) << endl;
		}
	}
	else {
		cout << "\t\tblank" << endl;
	}
	cout << "\n" << endl;
}

void clean_up_search_mem(set<unsigned int> *link_tree_rest, map<unsigned int, list<unsigned int>*> *link_tree_row, map<unsigned int, list<unsigned int>*> *link_tree_new_row) {
	//clean up set
	if (link_tree_rest) {
		delete[] link_tree_rest;
		delete link_tree_rest;
	}
	//clean up first map
	if (link_tree_row) {
		for (map<unsigned int, list<unsigned int>*>::iterator tmp_itr = link_tree_row->begin(); tmp_itr != link_tree_row->end(); tmp_itr++) {
			delete[] tmp_itr->second;
		}
		delete[] link_tree_row;
		delete link_tree_row;
	}
	//clean up second map
	if (link_tree_new_row) {
		for (map<unsigned int, list<unsigned int>*>::iterator tmp_itr = link_tree_new_row->begin(); tmp_itr != link_tree_new_row->end(); tmp_itr++) {
			delete[] tmp_itr->second;
		}
		delete[] link_tree_new_row;
		delete link_tree_new_row;
	}
}

list<unsigned int> *seek_links(unsigned int source, unsigned int destination, entry ** table) {
	//from table[source], find shortest path to destination by traversing links
	//essentially a breadth-first search of tree
	//returns a list of the hashes to click in order
	
	//use map to track already checked options
		//key = hash
		//value = link structure of history of retrieval (does it have to be a pointer?)
		//should be 2 maps: bottom row in link tree and everything else (because cycling through bottom row creates new bottom row, which insert()s while iterating through
		//map is helpful because duplicates are bad and searching must be fast
	
	//map contains every item in bottom row of link tree; must be 2 because cycling through link_tree_row inserts new entries into itself
	map<unsigned int, list<unsigned int>*> *link_tree_row = new map<unsigned int, list<unsigned int>*>;
	map<unsigned int, list<unsigned int>*> *link_tree_new_row = new map<unsigned int, list<unsigned int>*>;
	//contains every other item in tree: must have record of what links have been traversed to avoid redundancy
	set<unsigned int> *link_tree_rest = new set<unsigned int>;

	map<unsigned int, list<unsigned int>*>::iterator entry_itr;	//to cycle through row
	list<unsigned int> node_links;				//store a hash's link linked list
	list<unsigned int> *parent_path = NULL;		//store link's parent's path, to branch out and add onto
	list<unsigned int> *child_path = NULL;		//tmp var for creating link paths from their parents (parent + new link = child)
	pair<unsigned int, list<unsigned int>*> *link_entry;	//to reference entry without relocating it in table
	
	//to start, insert all of the source's links into the structure (bottom row)
	//this prevents the source from being stored in all of the path lists, which is redundant (because it's stored elsewhere) and expensive
	node_links = table[source]->links;
	for (list<unsigned int>::iterator link_itr = node_links.begin(); link_itr != node_links.end(); link_itr++) {
		if (*link_itr != source) {
			//insert link if it's different from the source, to prevent a few remotely possible redundancies
			link_entry = new pair<unsigned int, list<unsigned int>*>(*link_itr, new list<unsigned int>);
			link_tree_row->insert(*link_entry);
		}
	}	
		
	//start loop between rows within tree (10 layers deep is probably enough)
	//increasing the max depth is possible, but not recommended because this thing scales horribly 
	for (int i = 0; i < 10; i++){
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
						return child_path;
					}
					link_tree_new_row->insert(pair<unsigned int, list<unsigned int>*>(*link_itr, child_path));
				}
			}
			//can clean up parent_path, because all children have copied from it
			delete parent_path;
		}
		if (link_tree_new_row->empty()) {
			cout << "There is no way to get to the destination from the source" << endl;
			clean_up_search_mem(link_tree_rest, link_tree_row, link_tree_new_row);
			return NULL;
		}
		//move every key from bottom row into top half (so a new bottom row can be started)		//performance versus iterating through?
		//set_intersection(link_tree_rest->begin(), link_tree_rest->end(), link_tree_row->begin(), link_tree_row->end(), link_tree_rest);
		for (entry_itr = link_tree_row->begin(); entry_itr != link_tree_row->end(); entry_itr++) {
			link_tree_rest->insert(entry_itr->first);
			//delete[] entry_itr->second;
		}
		//link_tree_row now holds contents of link_tree_new_row, and link_tree_new_row gets reset to make room
		swap(link_tree_row, link_tree_new_row);
		link_tree_new_row->clear();
	}
	cout << "The search exceeded its maximum depth; this can be increased, but it is expensive" << endl;
	clean_up_search_mem(link_tree_rest, link_tree_row, link_tree_new_row);
	return NULL;
}


int main() {
	clock_t t = clock();	//start timer
	string path = string("E:\\OneDrive\\Programs\\C++_RPI\\WikiLinkr\\misc_data\\") + string("simple_parsed.txt");
	//string path = string("E:\\OneDrive\\Programs\\C++_RPI\\WikiLinkr\\misc_data\\") + string("english_wiki.txt");
	//string path = string("E:\\OneDrive\\Programs\\C++_RPI\\WikiLinkr\\misc_data\\") + string("test_input3.txt");
	
	std::hash<string> str_hash;	//initialize string hash function (better tailored to strings than bj or djb2 are)
	unsigned int hash;

	/*	Initialize hash table:
	*		Simple wiki: ~130,000 entries
	*		Whole wiki: ~5,000,000 entries
	*
	*		Structure should be a contiguous array of pointers to structs
	*			structs should hold url to compare (collision checking) as well as link structure
	*			address should be a hash of the url
	*			starting address should be ~100x expected size?
	*			should use list to hold links (vectors must be contiguous?)
	*			64-bit programs mean 16-bit addresses
	*/
	std::cout << sizeof(entry) << " bytes per entry" << std::endl;
	cout << "Initializing structure..." << endl;
	unsigned int table_entries = 5 * MEGABYTE;	//good size for sample english wiki (>200k new articles, >2 minutes)
	//unsigned int table_entries = 100 * MEGABYTE;	//good size for complete english wikipedia (>15M new articles, >12 hours)

	entry ** table = new entry*[table_entries];
	for (unsigned int i = 0; i < table_entries; i++) {
		//this is way faster than it should be, but still seems to work
		//thank you based compiler?
		table[i] = NULL;
	}
	unsigned int table_bytes = table_entries * sizeof(entry);

	int collisions = 0;
	
	unsigned int article_counter = 0;
	//start cycling through file:
	cout << "Start reading..." << endl;
	ifstream in_file(path);
	string *title = NULL;
	//string *sha1 = NULL;
	string sha1;
	string line;
	int link_hash;
	list<unsigned int> *links = NULL;
	unsigned int counter = 0;
	if (in_file) {
		while (getline(in_file, line)) {
			//process line-by-line
			if (line == "<page>") {
				//just finished reading in links; insert data into table
				//if (title != "") {
				if(title != NULL){
					//title should start as NULL; not sure why it isn't
					hash = resolve_collisions2(*title, table, table_entries, str_hash, collisions);
					create_entry(hash, title, table, links);
					article_counter++;
					if (article_counter % 100000 == 0) {
						//cout << "\t" << article_counter / 1000000 << "M" << endl;
						if (article_counter % 1000000 == 0) {
							cout << "\t" << article_counter / 1000000 << "M" << endl;
						}
						else {
							cout << "\t" << article_counter / 100000 << " * 100K" << endl;
						}
					}
				}
				title = new string;
				//sha1 = new string;
				links = new list<unsigned int>;
				//about to show article metadata
				getline(in_file, *title);
				//getline(in_file, *sha1);
				getline(in_file, sha1);
				counter++;
			}
			else {
				//line is a link: get the hash, create if necessary, and store it
				link_hash = resolve_collisions2(line, table, table_entries, str_hash, collisions);
				if (table[link_hash] == NULL) {
					//if link didn't exist, create it 
					create_entry(link_hash, new string(line), table);
				}
				links->push_back(link_hash);
			}
		}
		//insert last article data into table
		hash = resolve_collisions2(*title, table, table_entries, str_hash, collisions);
		create_entry(hash, title, table, links);
		in_file.close();
	}

	cout << "Done indexing; " << collisions << " collisions \n\n" << endl;
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
	cout << "Found " << entries << " populated slots, " << blanks << " unpopulated." << endl;
	cout << "With " << table_entries << " slots, that is " << float(entries) / table_entries * 100 << "%\n" << endl;
	//Logs 203,435 pages; parser finds 208,153
	//	Table not missing any entries, but the dump contains duplicates; might have to revisit the parser


	std::cout << collisions << " total collisions" << std::endl;
	//delete[] table;
	t = clock() - t;
	std::cout << "Total time: " << t << " clicks, " << ((float)t) / 1000 << " seconds." << std::endl << endl << endl;

	/* // Write article titles to file to test against parsed dump file:
	cout << "Retrieving output: " << endl;
	ofstream f_out;
	f_out.open("article_names.txt");
	string tmp_name;
	int num_articles = 0;
	for (int i = 0; i < table_entries; i++) {
		if (table[i] != NULL) {
			f_out << *(table[i]->url) << endl;
		}
	}
	f_out.close();
	*/


	//getchar();

	bool debug = true;

	if(debug) {
		//allow user to test input:
		int input = -1;
		string tmp_title = "";
		unsigned int tmp_hash = -1;
		list<unsigned int> tmp_list;
		int tmp_count = 0;
		while (input != 0) {
			cout << "\n\nEnter one of the following: \n\t0:\t\tExit \n\t1:\t\tFind article in table \n\t2:\t\tFind hash in table \n\t3:\t\tPrint links of last article (" << tmp_title << ")" << endl;
			cout << "\t4:\t\tFind link path between articles" << endl;
			cin >> input;
			if (input == 1) {
				cout << "  Please enter article name: ";
				cin >> tmp_title;
				transform(tmp_title.begin(), tmp_title.end(), tmp_title.begin(), ::toupper);	//capitalize
				cout << endl;
				tmp_hash = resolve_collisions2(tmp_title, table, table_entries, str_hash, collisions, true);
				cout << "  Found article slot for '" << tmp_title << "' at hash " << tmp_hash << ";" << endl;
			}
			else if (input == 2) {
				cout << " Please enter hash: ";
				cin >> tmp_hash;
				cout << endl;
				if (table[tmp_hash] == NULL) {
					cout << " hash " << tmp_hash << " not found" << endl;
				}
				else {
					cout << " table[" << tmp_hash << "] = " << *(table[tmp_hash]->url) << endl;
				}
			}
			else if (input == 3) {
				cout << "  Links under article '" << tmp_title << "';" << endl;
				tmp_hash = resolve_collisions2(tmp_title, table, table_entries, str_hash, collisions);
				//tmp_list = *table[tmp_hash]->links;
				tmp_list = table[tmp_hash]->links;
				for (list<unsigned int>::iterator tmp_itr = tmp_list.begin(); tmp_itr != tmp_list.end(); tmp_itr++) {
					tmp_count++;
					cout << "\t" << tmp_count << ": \t" << *tmp_itr << " = \t" << *(table[*tmp_itr]->url) << endl;
				}
			}
			else if (input == 4) {
				string source;
				string dest;
				cout << " Enter source: ";
				cin >> source;
				transform(source.begin(), source.end(), source.begin(), ::toupper);	//capitalize
				cout << " Enter destination: ";
				cin >> dest;
				transform(dest.begin(), dest.end(), dest.begin(), ::toupper);	//capitalize
				t = clock();	//start timer

				unsigned int source_hash = resolve_collisions2(source, table, table_entries, str_hash, collisions);
				unsigned int dest_hash = resolve_collisions2(dest, table, table_entries, str_hash, collisions);
				if (table[source_hash] == NULL) {
					cout << "Error: " << source << " wasn't found in the table (it would be at " << source_hash << ");" << endl;
					continue;
				}
				else if (table[dest_hash] == NULL) {
					cout << "Error: " << dest << " wasn't found in the table (it would be at " << dest_hash << ");" << endl;
					continue;
				}
				else {
					list<unsigned int> *link_path = seek_links(source_hash, dest_hash, table);
					if (link_path) {
						cout << "\n\nFound path from " << *table[source_hash]->url << " (" << source_hash << ") to " << *table[dest_hash]->url << " (" << source_hash << ")" << endl;
						cout << "\t" << source_hash << "  =  " << *table[source_hash]->url << "*" << endl;
						for (list<unsigned int>::iterator tmp_itr = link_path->begin(); tmp_itr != link_path->end(); tmp_itr++) {
							cout << "\t" << *tmp_itr << "  =  " << *table[*tmp_itr]->url << endl;
						}
						cout << "\t" << dest_hash << "  =  " << *table[dest_hash]->url << "*" << endl;
						t = clock() - t;
						cout << "\t\t(" << t / 1000.0 << " seconds)" << endl;
					}
					else {
						cout << "\n\nCouldn't find path from " << *table[source_hash]->url << " (" << source_hash << ") to " << *table[dest_hash]->url << " (" << source_hash << ")" << endl;
					}
				}
			}
		}
	}
	else{
		t = clock();	//start timer

		string source = "ABACUS";
		string dest = "PHILOSOPHY";

		unsigned int source_hash = resolve_collisions2(source, table, table_entries, str_hash, collisions);
		unsigned int dest_hash = resolve_collisions2(dest, table, table_entries, str_hash, collisions);
		list<unsigned int> *link_path = seek_links(source_hash, dest_hash, table);

		if (link_path) {
			cout << "\n\nFound path from " << *table[source_hash]->url << " (" << source_hash << ") to " << *table[dest_hash]->url << " (" << source_hash << ")" << endl;
			cout << "\t" << source_hash << "  =  " << *table[source_hash]->url << "*" << endl;
			for (list<unsigned int>::iterator tmp_itr = link_path->begin(); tmp_itr != link_path->end(); tmp_itr++) {
				cout << "\t" << *tmp_itr << "  =  " << *table[*tmp_itr]->url << endl;
			}
			cout << "\t" << dest_hash << "  =  " << *table[dest_hash]->url << "*" << endl;
		}
		else {
			cout << "\n\nCouldn't find path from " << *table[source_hash]->url << " (" << source_hash << ") to " << *table[dest_hash]->url << " (" << source_hash << ")" << endl;
		}
		t = clock() - t;
		std::cout << "Total time: " << t << " clicks, " << ((float)t) / 1000 << " seconds." << std::endl << endl << endl;
	}
	
	getchar();

	//clean up (most) memory
	/*
	for (unsigned int i = 0; i < table_entries; i++) {
		if (table[i]) {
			delete[] table[i]->url;
			delete[] table[i];	//?
			delete table[i];	//?
		}
	}
	delete[] table;
	delete table;
	*/
	return 0;
}


/*TODO
	Clean up memory after a search
	Clean up memory at the end of the program
	Fix bug where space bars aren't interpreted correctly in searches
	Parsr sometimes includes duplicate entries: should combine rather than replace
	Re-device UI

	Implement update of links file in Python from log/newer dump
		Find where links are getting lost (2%, down from 15%)
	Profiling to find expensive parts
	sizeof(string) > sizeof(char[]) ???
	Remove duplicate entries (on parser side?) (dumps contain multiple entries w/ different links?)
	Clean up memory (first each entry, then table)
	Verify data integrity following links on links

STATUS
	Occupying ~20% of the table requires 1GB for simple wiki (~105GB for total)
	Links are not populating the table (shouldn't anyway)
		Should an entry's link storage include strings (yes) AND hashes? 
			Would sacrifice memory for speed

*/