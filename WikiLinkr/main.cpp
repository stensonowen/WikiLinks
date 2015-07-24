/*
Run in 64-bit to use >2GB of memory
Tested using >8GB (system RAM) of contiguous memory in x64 without issue
Only with pagefile (windows 8.1); useability with swap instead (ubuntu)??

Takes ~2 minutes to enter simple wiki; no profiling done yet
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
#define KILOBYTE 1024
#define MEGABYTE 1024*1024

using namespace std;

struct entry {
	//sizeof(entry) = 8  bytes in 32-bit
	//sizeof(entry) = 16 bytes in 64-bit
	string *url;				//holds url: check for collisions
	list<string>* links;	//pointer to linked list holding links
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



//size_t resolve_collisions2(const string *str, entry ** table, size_t table_entries, hash<string> *str_hash, unsigned int *collisions) {
size_t resolve_collisions2(const string &str, entry ** table, size_t table_entries, hash<string> &str_hash, unsigned int *collisions, bool verbose=false) {
	//employ hash function and then use collision-checking algorithm
	/* Deal with collisions by retrying with an offset of n!+1;
	Should be slightly more successful than an offset of n^2 because it generates primes very frequently (prime for 0<=n<=4, and then ~50% for n>4).
	Evades the performance hit of factorials because it only finds one product per attempt, which it stores in memory.
	Thus, rather than O(n!) additional cycles, it only requires one int and two addition operations (4 bytes, <=2 cycles)
	*/
	//	This version caps the number of collision checks at {some constant}.
	size_t hash = (str_hash)(str);
	unsigned int offset = 0;
	unsigned int multiplier = 1;	//multiplier=1 already checked via while() statement
									//static unsigned int collisions;
	collisions--;	//to offset incrementer
	for (int i = 0; i < 100; i++) {
		offset = (offset - 1)*multiplier + 1;
		multiplier += 1;
		hash += offset;
		hash %= table_entries;
		if (verbose) cout << "  Trying hash " << hash << "..." << endl;
		if (verbose) {
			if (table[hash] == NULL) cout << "  No entry found at hash " << hash << ";" << endl;
			else cout << "  Entry '" << *(table[hash]->url) << "' found at hash " << hash << ";" << endl;
		}
		if (table[hash] == NULL || *(table[hash]->url) == str) { return hash; }
		else {
			//Only keep track of collisions if it's necessary (if a var is passed)
			if (collisions != NULL) collisions++;
		}
	}
	//return if that value in the table is blank or a match
	if (verbose) cout << "   Didn't find any blank entries in k iterations;" << endl;
	return -1;	//should break something if 
}

void read_entry(const string &url, entry ** table, size_t table_entries, hash<string> &str_hash) {
	size_t hash = resolve_collisions2(url, table, table_entries, str_hash, NULL);
	if (table[hash] == NULL) {
		cout << "Entry " << &url << " is not present." << endl;
	}
	else {
		cout << "Entry " << url << " is present at 0x" << table[hash] << " and links to: " << endl;
		list<string> l = *(table[hash]->links);
		for (list<string>::iterator itr = l.begin(); itr != l.end(); itr++) {
			cout << "\t" << *itr << endl;
		}
	}
}

void create_entry(size_t hash, string &url, entry ** table, list<string> *links = NULL) {
	//make a new entry from the given details
	table[hash] = new entry;
	table[hash]->url = &url;
	if (!links) { table[hash]->links = new list<string>; }
	else { table[hash]->links = links; }
}

int main() {
	clock_t t = clock();	//time program
	//string path = string("E:\\Libraries\\Downloads\\WIKIPEDIA\\SIMPLE_FILES\\") + string("wiki_");	//gets rid of weird type errors (RIP strcat())
	string path = string("E:\\Libraries\\Programs\\C++_RPI\\WikiLinkr\\misc_data\\") + string("out_sample_v11.txt");
	std::hash<string> str_hash;
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
	*			64-bit programs mean 16-bit addresses; so a pointer to 
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

	unsigned int collisions = 0;	//for analytics (?)
	/*
	vector<string> examples = { "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta" };
	for (int i = 0; i < examples.size(); i++){
		hash = resolve_collisions2(examples[i], table, table_entries, str_hash, &collisions);
		read_entry(examples[i], table, table_entries, str_hash);
		create_entry(hash, examples[i], table);
	}
	for (int i = 0; i < examples.size(); i++) {
		hash = resolve_collisions2(examples[i], table, table_entries, str_hash, &collisions);
		read_entry(examples[i], table, table_entries, str_hash);
		//create_entry(hash, examples[i], table);
	} */
	
	//start cycling through file:
	cout << "Start reading..." << endl;
	ifstream in_file(path);
	string line;
	string title(""), sha1;
	list<string> links;
	unsigned int counter = 0;
	if (in_file) {
		while (getline(in_file, line)) {
			//process line-by-line
			if (line == "<page>") {
				//just finished reading in links; insert data into table
				if (title != "") {
					hash = resolve_collisions2(title, table, table_entries, str_hash, &collisions);
					create_entry(hash, title, table, &links);
				}
				//about to show article metadata
				getline(in_file, title);
				getline(in_file, sha1);
				links.clear();
				counter++;
			}
			else {
				//line is a link
				links.push_back(line);
			}
		}
		//insert last article data into table
		hash = resolve_collisions2(title, table, table_entries, str_hash, &collisions);
		create_entry(hash, title, table, &links);
		in_file.close();
	}

	cout << "Done indexing; " << collisions << " collisions \n\n" << endl;
	collisions = 0;
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
	//Should find 208,153 pages; finds ~180k (17% population rate) 
	/*
	string test1 = "APRIL";
	hash = resolve_collisions2(test1, table, table_entries, str_hash, &collisions);
	create_entry(hash, test1, table, NULL);
	test1 = "ART";
	hash = resolve_collisions2(test1, table, table_entries, str_hash, &collisions);
	create_entry(hash, test1, table, NULL);

	read_entry("APRIL", table, table_entries, str_hash);
	read_entry("ART", table, table_entries, str_hash);
	*/



	//manually add text exes

	
	/* misc use ex
	read_entry("avocados", table, table_entries, str_hash);
	hash = resolve_collisions2("avocados", table, table_entries, str_hash, &collisions);
	list<string> *x = new list<string>;
	x->push_back("apricots");
	create_entry(hash, "avocados", table, x);
	*/

	std::cout << collisions << " total collisions" << std::endl;
	//delete[] table;
	t = clock() - t;
	std::cout << "Total time: " << t << " clicks, " << ((float)t) / 1000 << " seconds." << std::endl;


	getchar();




	//allow user to test input:
	int input = -1;
	string tmp_title = "";
	size_t tmp_hash = -1;
	list<string> tmp_list;
	int tmp_count = 0;
	while (input != 0) {
		cout << "\n\nEnter one of the following: \n\t0:\t\tExit \n\t1:\t\tFind article in table \n\t2:\t\tPrint links of last article (" << tmp_title << ")" << endl;
		cin >> input;
		if (input == 1) {
			cout << "  Please enter article name: ";
			cin >> tmp_title;
			cout << endl;
			tmp_hash = resolve_collisions2(tmp_title, table, table_entries, str_hash, NULL, true);
			cout << "  Found ~~article~~ slot for '" << tmp_title << "' at hash " << tmp_hash << ";" << endl;
		}
		else if (input == 2) {
			cout << "  Links under article '" << tmp_title << "';" << endl;
			tmp_hash = resolve_collisions2(tmp_title, table, table_entries, str_hash, NULL);
			//tmp_list = *table[tmp_hash]->links;
			tmp_list = *(table[tmp_hash]->links);
			for (list<string>::iterator tmp_itr = tmp_list.begin(); tmp_itr != tmp_list.end(); tmp_itr++) {
				tmp_count++;
				cout << "\t" << tmp_count << ": \t" << *tmp_itr << endl;
			}
		}
	}

	return 0;
}


/*TODO
Implement update of links file in Python from log/newer dump
~~Create file format for storing processed post-parser data (?)~~
Find where links are getting lost
	implement debugging interface, so that the table can be populated once 
	and then queried at will (until it's changed)
Profiling to find expensive parts

*/