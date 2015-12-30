#include<iostream>
#include<fstream>
#include<list>
#include<tr1/functional>
#include<string>
#include<time.h>
//#include<algorithm>
//#include<iomanip>
#include<stdlib.h>
using namespace std;

class Entry{
    private:
        //Entry will be on heap, so don't need pointer member vars
        string title;
        list<unsigned int> links;
    public:
        Entry() {}
        Entry(const string &t, list<unsigned int> &l) : title(t), links(l) {}
        ~Entry() {}
};

class Table{
    private:
        tr1::hash<string> str_hash;
        Entry ** table;             //array itself
        unsigned int entries;       //number of entries
        unsigned int size;          //number of entries and blanks
        long collisions;
    public:
        Table(char *input_file);
        ~Table();
        unsigned int resolve_collisions(const string &title);
        pair<list<unsigned int>*, int> seek_links(unsigned int src, unsigned int dst);

};

Table::Table(char *input_file){
    //tr1::hash<string> str_hash;
    //open input file
    std::ifstream input(input_file);
    if(!input){
        fputs("Failed to open input.\n", stderr);
        exit(1);
    }
    #ifdef DEBUG
    clock_t t = clock();
    int progress;
    cout << "Initializing structure..." << endl;
    #endif

    //create table
    string tmp;
    getline(input, tmp);
    entries = atoi(tmp.c_str());
    
    size = 20 * entries + 1000;     //subject to optimization
    table = new Entry*[size];
    for(unsigned int i=0; i<size; i++)
        table[i] = NULL;
    cout << "done;" << endl;
    

}

Table::~Table(){}

unsigned int Table::resolve_collisions(const string &title){
    unsigned int hash = (str_hash)(title) % entries;
}


