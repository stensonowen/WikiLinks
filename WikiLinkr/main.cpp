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
#define KILOBYTE 1024
#define MEGABYTE 1024*1024

using namespace std;

struct entry {
	//sizeof(entry) = 8  bytes in 32-bit
	//sizeof(entry) = 16 bytes in 64-bit
	string *url;			//holds url: (to check for collisions)
	list<int> links;		//list of hashes
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


unsigned int resolve_collisions2(const string &str, entry ** table, size_t table_entries, hash<string> &str_hash, int &collisions, bool verbose=false) {
	//Employ hash function and then use custom collision-resolving algorithm
	/* Deal with collisions by retrying with an offset of n!+1;
	Should be slightly more successful than an offset of n^2 because it generates primes very frequently (prime for 0<=n<=4, and then ~50% for n>4).
	Evades the performance hit of factorials because it only finds one product per attempt, which it stores in memory.
	Thus, rather than O(n!) additional cycles, it only requires one int and two addition operations (4 bytes, <=2 cycles)	*/
	//	This version caps the number of collision checks at {some constant}.
	size_t hash = (str_hash)(str);
	unsigned int offset = 0;
	unsigned int multiplier = 1;
	for (int i = 0; i < 100; i++) {
		offset = (offset - 1)*multiplier + 1;
		multiplier += 1;
		hash += offset;
		hash %= table_entries;
		if (verbose) {
			cout << "  Trying hash " << hash << "..." << endl;
			if (table[hash] == NULL) cout << "  No entry found at hash " << hash << ";" << endl;
			else cout << "  Entry '" << *(table[hash]->url) << "' found at hash " << hash << ";" << endl;
		}
		if (table[hash] == NULL || *(table[hash]->url) == str) { return (unsigned int)hash; }
		else {
			collisions++;
		}
	}
	//return if that value in the table is blank or a match
	if (verbose) cout << "   Didn't find any blank entries in k iterations;" << endl;
	return -1;	//should break something if 
}

void read_entry(const string &url, entry ** table, size_t table_entries, hash<string> &str_hash) {
	//Read entry info given from url
	int collisions = 0;
	size_t hash = resolve_collisions2(url, table, table_entries, str_hash, collisions);
	cout << "After " << collisions << " collisions:  ";
	if (table[hash] == NULL) {
		cout << "Entry " << &url << " is not present." << endl;
	}
	else {
		cout << "Entry " << url << " is present at 0x" << table[hash] << " and links to: " << endl;
		list<int> l = table[hash]->links;
		for (list<int>::iterator itr = l.begin(); itr != l.end(); itr++) {
			cout << "\t" << table[*itr]->url << endl;
		}
	}
}

void create_entry(size_t hash, string *url, entry ** table, list<int> *links = NULL) {
	//make a new entry from the given details; 
	table[hash] = new entry;
	table[hash]->url = url;
	if(links) table[hash]->links = *links;
}

int main() {
	clock_t t = clock();	//start timer
	//string path = string("E:\\OneDrive\\Programs\\C++_RPI\\WikiLinkr\\misc_data\\") + string("simple_parsed.txt");
	string path = string("E:\\OneDrive\\Programs\\C++_RPI\\WikiLinkr\\misc_data\\") + string("test_input.txt");
	
	std::hash<string> str_hash;	//initialize string hash function (better tailored to strings than bj or djb2 are)
	size_t hash;

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
	size_t table_entries = 1 * MEGABYTE;
	entry ** table = new entry*[table_entries];
	for (int i = 0; i < table_entries; i++) {
		//this is way faster than it should be, but still seems to work
		//thank you based compiler?
		table[i] = NULL;
	}
	size_t table_bytes = table_entries * sizeof(entry);

	int collisions = 0;
	
	//start cycling through file:
	cout << "Start reading..." << endl;
	ifstream in_file(path);
	//string line;
	//string title(""), sha1;
	string *title = NULL;
	string *sha1 = NULL;
	string line;
	int link_hash;
	list<int> *links = NULL;
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
				}
				title = new string;
				sha1 = new string;
				links = new list<int>;
				//about to show article metadata
				getline(in_file, *title);
				getline(in_file, *sha1);
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


	getchar();



	
	//allow user to test input:
	int input = -1;
	string tmp_title = "";
	size_t tmp_hash = -1;
	list<int> tmp_list;
	int tmp_count = 0;
	while (input != 0) {
		cout << "\n\nEnter one of the following: \n\t0:\t\tExit \n\t1:\t\tFind article in table \n\t2:\t\tFind hash in table \n\t3:\t\tPrint links of last article (" << tmp_title << ")" << endl;
		cin >> input;
		if (input == 1) {
			cout << "  Please enter article name: ";
			cin >> tmp_title;
			transform(tmp_title.begin(), tmp_title.end(), tmp_title.begin(), ::toupper);	//capitalize
			cout << endl;
			tmp_hash = resolve_collisions2(tmp_title, table, table_entries, str_hash, collisions, true);
			cout << "  Found ~~article~~ slot for '" << tmp_title << "' at hash " << tmp_hash << ";" << endl;
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
			for (list<int>::iterator tmp_itr = tmp_list.begin(); tmp_itr != tmp_list.end(); tmp_itr++) {
				tmp_count++;
				cout << "\t" << tmp_count << ": \t" << *tmp_itr << "= \t" << *(table[*tmp_itr]->url) << endl;
			}
		}
	}
	

	return 0;
}


/*TODO
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