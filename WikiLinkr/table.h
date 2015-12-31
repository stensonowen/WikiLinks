#include<iostream>
#include<fstream>
#include<vector>
#include<list>
#include<tr1/functional>
#include<string>
#include<time.h>
//#include<algorithm>
//#include<iomanip>
#include<cassert>
#include<stdlib.h>
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
    public:
        Table(char *input_file);
        ~Table();
        unsigned int resolve_collisions(const string &title);
        unsigned int create_if_absent(const string &title);
        pair<list<unsigned int>*, int> seek_links(unsigned int src, unsigned int dst);
        void populate(vector<string> files);        //multithread master
        void *read(void *f);    //multithread slave
};

unsigned int Table::create_if_absent(const string &title){
    unsigned int hash = resolve_collisions(title);
    if(table[hash] == NULL){
        table[hash] = new Entry(title);
    }
    return hash;
}

void *Table::read(void *f){
    ifstream f_in((char*)f);
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
    cout << " " << articles << " articles; " << lines << " lines; " << endl;

}

Table::Table(char *input_file){
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

void Table::populate(vector<string> threads){
    for(int i=0; i<threads.size(); i++){
        //cout << threads[i] << endl;
        read((void*)threads[i].c_str());
    }
    unsigned int blanks = 0, used = 0;
    for(int i=0; i<size; i++){
        if(table[i] == NULL){
            blanks++;
        } else {
            used++;
        }
    }
    cout << used << " used, " << blanks << " unused out of " << size << endl;
}

Table::~Table(){
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
    unsigned int multiplier = 1;
    //max_iters
    for(int i=0; i<MAX_ITERS; i++){
        hash %= size;
        if(table[hash] == NULL || table[hash]->title == title){
            if(i>max_iters){    max_iters = i;  }
            return hash;
        } else {    collisions++;   }
        offset = (offset - 1)*multiplier + 1;
        assert(offset != 1);
        multiplier++;
        hash += offset;
    }
    assert(MAX_ITERS != MAX_ITERS);
}


