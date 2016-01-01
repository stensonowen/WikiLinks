#include<iostream>
#include<fstream>
#include<vector>
#include<list>
#include<set>
#include<tr1/functional>
#include<string>
#include<time.h>
//#include<algorithm>
//#include<iomanip>
#include<cassert>
#include<stdlib.h>
//#include<pthread.h>
#include<thread>
using namespace std;

#define MAX_DEPTH 10
#define MAX_ITERS 100

class Entry{
    public:
        //Entry will be on heap, so don't need pointer member vars
        string title;
        list<unsigned int> links;
        Entry() {}
        Entry(const string &t) : title(t) {}
        Entry(string &t, list<unsigned int> &l) : title(t), links(l) {}
        ~Entry() {}
};

class Table{
    private:
        tr1::hash<string> str_hash;
        Entry ** table;             //array itself
        unsigned int entries;       //number of entries
        unsigned int size;          //number of entries and blanks
        long collisions;
        long max_iters;
        void populate(vector<string> files);        //multithread master
        void read(string file);    //multithread slave
    public:
        Table(char *input_file);
        ~Table();
        unsigned int resolve_collisions(const string &title);
        unsigned int create_if_absent(const string &title);
        pair<list<unsigned int>*, int> seek_links(unsigned int src, unsigned int dst);
        void details();
};

void Table::details(){
    cout << "Table details:" << endl;
    unsigned int blanks = 0, used = 0, links = 0;
    for(int i=0; i<size; i++){
        if(table[i] == NULL){
            blanks++;
        } else {
            used++;
            links += table[i]->links.size();
        }
    }
    cout << "Entries: " << entries << endl;
    cout << "\t" << used << " used, " << blanks << " unused out of " << size << " (" << (int)(100*used/size) << "%)" << endl;
    cout << "\tTotal links: " << links << " (average: " << links/used << ")" << endl;
}

unsigned int Table::create_if_absent(const string &title){
    unsigned int hash = resolve_collisions(title);
    if(table[hash] == NULL){
        table[hash] = new Entry(title);
    }
    return hash;
}

void Table::read(string file){
    ifstream f_in((char*)file.c_str());
    string line;
    unsigned int addr, articles, lines, link_addr;
    articles = lines = 0;
    while(getline(f_in, line)){
        lines++;
        if(line == "<page>"){
            getline(f_in, line);
            addr = create_if_absent(line);
            articles++;
        } else {
            link_addr = create_if_absent(line);
            table[addr]->links.push_back(link_addr);
        }
    }
    //cout << " " << articles << " articles; " << lines << " lines; " << endl;
}

Table::Table(char *input_file){
    collisions = 0;
    max_iters = 0;
    //open input file
    std::ifstream input(input_file);
    if(!input){
        fputs("Failed to open input.\n", stderr);
        exit(1);
    }

    //create table
    string tmp;
    getline(input, tmp);
    entries = atoi(tmp.c_str());
    size = 20 * entries + 1000;     //equation subject to optimization
    table = new Entry*[size];
    for(unsigned int i=0; i<size; i++)
        table[i] = NULL;
    //copy files
    vector<string> threads;
    while(getline(input, tmp)){
        threads.push_back(tmp);
    }
    populate(threads);
}

void Table::populate(vector<string> files){
    //create and run threads to populate table
    //remember to add mutex system
    vector<thread> threads;
    for(int i=0; i<files.size(); i++){
        threads.push_back(thread(&Table::read, this, files[i]));
        //cout << files[i] << endl;
        //read((void*)threads[i].c_str());
        //read(files[i]);
    }
    for(int i=0; i<threads.size(); i++){
        threads[i].join();
    }
    cout << "created " << threads.size() << " threads" << endl;
}

Table::~Table(){
    //"total heap usage: 7,277,425 allocs, 7,277,425 frees, 247,320,661 bytes allocated"
    //No leaks
    for(int i=0; i<size; i++){
        if(table[i]){
            delete table[i];
        }
    }
    delete[] table;
}

unsigned int Table::resolve_collisions(const string &title){
    //using factorial-based generates primes more than squares
    //perhaps this isn't actually all that great. it finds primes relatively often,
    //  but they grow extremely quickly, which destroys spatial locality
    //  A132199 (https://en.wikipedia.org/wiki/Formula_for_primes#Recurrence_relation)
    //      could store first few dozen and generate more as needed
    //      if values were stored it would be quite fast
    //      if 1's were excluded, it would generate small primes consistently
    //      
    unsigned int hash = (str_hash)(title) % size;
    int offset = 2;
    for(unsigned int multiplier = 1; multiplier < MAX_ITERS; multiplier++){
        hash %= size;
        if(table[hash] == NULL || table[hash]->title == title){
            if(multiplier>max_iters){    max_iters = multiplier;  }
            collisions += multiplier;
            return hash;
        } else {    collisions++;   }
        offset = (offset - 1)*multiplier + 1;
        assert(offset != 1);
        hash += offset;
    }
    assert(MAX_ITERS != MAX_ITERS);
}


