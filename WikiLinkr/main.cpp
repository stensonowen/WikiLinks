/*
Run in 64-bit to use >2GB of memory
Tested using >8GB (system RAM) of contiguous memory in x64 without issue
Only with pagefile (windows 8.1); useability with swap instead (ubuntu)??

Takes ~2.5 minutes to enter simple wiki, using ~1 GB, 0 collisions (?), 1,000,000 entries
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
#define CLOCKS_PER_SECOND 1000

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
size_t resolve_collisions2(const string &str, entry ** table, size_t table_entries, hash<string> &str_hash, unsigned int *collisions) {
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
	for (int i = 0; i < 100; i++) {
		//Only keep track of collisions if it's necessary (if a var is passed)
		if (collisions != NULL) { collisions++; }
		offset = (offset - 1)*multiplier + 1;
		multiplier += 1;
		hash += offset;
		hash %= table_entries;
		if (table[hash] == NULL || *(table[hash]->url) == str) { return hash; }
		//return if that value in the table is blank or a match
	}
	return 0;
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

	//start cycling through file:
	ifstream in_file(path);
	string line;
	string title(""), sha1;
	list<string> links;
	if (in_file) {
		while (getline(in_file, line)) {
			//process line-by-line
			if (line == "<path>\n") {
				//just finished reading in links; insert data into table
				if (title != "") {
					hash = resolve_collisions2(title, table, table_entries, str_hash, &collisions);
					create_entry(hash, title, table, &links);
				}
				//about to show article metadata
				getline(in_file, title);
				getline(in_file, sha1);
				links.clear();
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




	/*
	//test stuff:
	string a1 = "TEST_STRING_1";
	string a2 = "TEST_LINK_2";
	read_entry(a1, table, table_entries, str_hash);
	read_entry(a2, table, table_entries, str_hash);

	//try to insert a1, a2 into a1, then read a2 from a1:
	hash = resolve_collisions2(a1, table, table_entries, str_hash, &collisions);
	list<string> *x = new list<string>;
	x->push_back(a2);
	create_entry(hash, a1, table, x);

	size_t hash2 = resolve_collisions2(a2, table, table_entries, str_hash, &collisions);
	x = new list<string>;
	x->push_back(a1);
	create_entry(hash2, a2, table, x);

	read_entry(a1, table, table_entries, str_hash);
	read_entry(a2, table, table_entries, str_hash);
	*/



	/*
	//start reading files:
	for (int i = 0; i < 1; i++){	//132
	//fix name generator
	if (i < 10){
	filename = path + "0" + to_string(i);
	} else {
	filename = path + to_string(i);
	}

	//open file
	fin.open(filename);
	if (fin.is_open()){
	std::cout << "Opened " << filename << std::endl;
	} else {
	std::cout << "Failed to open '" << filename << "'" << std::endl;
	}

	//start reading data:
	while (getline(fin, ln_buf)){
	//if (regex_match(ln_buf, regex(pattern))){	//TODO: fix: searching twice
	if ("<doc id=" == ln_buf.substr(0,8)){	//placeholder: guess at whether it'll fit the regex
	//Start of new article
	counter++;
	std::cout << "Article <" << title << "> has " << subcounter << " links.\n";
	subcounter = 0;

	vector<string> md = extract_title_metadata(ln_buf);
	id = md[0];
	url = md[1];
	title = md[2];

	//Hash article url:
	hash = resolve_collisions2(&url, table, table_entries, &str_hash, &collisions);
	table[hash] = new entry;
	table[hash]->url = &url;
	table[hash]->links = new list < string > ;

	}
	links = extract_link_urls(ln_buf);
	subcounter += links.size();		//pass ptr to first char (string -> char*)
	for (itr = links.begin(); itr != links.end(); itr++){
	//cast iterator's value to an unsigned char ptr, and hash it
	//bj_hash((unsigned char*)(*itr).c_str());
	table[hash]->links->push_back(*itr);
	}
	}
	std::cout << "Article <" << url << "> has " << subcounter << " links.\n";


	fin.close();
	}
	std::cout << "\n\nTotal Articles: " << counter << std::endl;

	string u1 = "Capitalization";
	read_entry(&u1, table, table_entries, &str_hash);
*/
	std::cout << collisions << " total collisions" << std::endl;
	delete[] table;
	t = clock() - t;
	std::cout << "Total time: " << t << " clicks, " << ((float)t) / CLOCKS_PER_SEC << " seconds." << std::endl;
	getchar();

	return 0;
}

/*
Change filenames: could use a 26-base to determine extra number
cd E:\Libraries\Downloads\WIKIPEDIA\ex_smp_2\AB
for filename in os.listdir("."):
os.rename(filename, filename[:5] + "1" + filename[-2:])

Article Name Regex:		<doc id="[\d]+" url=.+" title=".+">
Article Link Regex:		<a href="[^ ]+"></a>
*/

/*TODO
Fix wonky passes by ref (late at night)
Fix parsing issues (python-side)
No tested version of WikiExtractor (2.32, 2.34) handles links properly.
A hackey fix of 2.32 does, but only 2.34 utilizes multithreading properly.
Need to amend code anyway; need to use urllib to generate page url from title
Write my own???
Create file format for storing processed post-parser data (?)
Once program fills out hash table once, it should save data in easily recreatable way
Should be easy; most expensive part by far is regexes, which don't need to be used every time.
Write automated updater???
Faster way to update info than by re-downloading wiki dump, re-parsing, and re-re-ing?



*/