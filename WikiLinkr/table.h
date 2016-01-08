#include<iostream>
#include<fstream>
//stl
#include<string>
#include<vector>
#include<list>
#include<set>
#include<map>
//
#include<tr1/functional>
#include<time.h>
#include<algorithm>
#include<iomanip>
//misc
#include<cassert>
#include<stdlib.h>
#include<memory>
//threading
#include<thread>
#include<mutex>
#include<atomic>
using namespace std;

#define MAX_DEPTH 10
#define MAX_ITERS 100
#define NUM_MUTEX 100

class Entry{
    public:
        //Entry will be on heap, so don't need pointer member vars
        string title;
        list<unsigned int> links;
        Entry() {}
        Entry(const string &t) : title(t) {}
        //Entry(string &t, list<unsigned int> &l) : title(t), links(l) {}
        ~Entry() {}
};

class Table{
    private:
        tr1::hash<string> str_hash;
        Entry ** table;             //array itself
        unsigned int entries;       //number of entries
        unsigned int size;          //number of entries and blanks
        //mutex mtx; //mutex around Entry() adds ~6% overhead (pretty sure it's type safe)
        mutex mtx[NUM_MUTEX];
        long collisions;
        long max_iters;
        void populate(vector<string> files);        //multithread master
        void read(string file);    //multithread slave
    public:
        Table(char *input_file);
        ~Table();
        unsigned int resolve_collisions(const string &title, bool create=true);
        pair<shared_ptr<list<unsigned int>>, int> seek_links(unsigned int src, unsigned int dst);
        void details();
        void printPath(string src, string dst);
};

void Table::printPath(string src, string dst){
    clock_t t = clock();
    //capitalize
    transform(src.begin(), src.end(), src.begin(), ::toupper);
    transform(dst.begin(), dst.end(), dst.begin(), ::toupper);
    unsigned int src_ = resolve_collisions(src, false);
    unsigned int dst_ = resolve_collisions(dst, false);
    pair<shared_ptr<list<unsigned int>>, int> results;
    list<unsigned int>::iterator res_itr;
    int pad_length = log10(size) + 1;
    if(!table[src_]) cout << "Cannot find article \"" << src << "\" (" << src_ << ")" << endl;
    //if(!src_ || table[src_]->title != src) cout << "Cannot find article \"" << src << "\" (" << src_ << ")" << endl;
    if(!table[dst_]) cout << "Cannot find article \"" << dst << "\" (" << dst_ << ")" << endl;
    if(!table[src_] || !table[dst_]) return;
    results = seek_links(src_, dst_);
    if(results.second == -1) cout << "No path exists from " << table[src_]->title << " (" << src_ << ") to " << table[dst_]->title << " (" << dst_ << ")" << endl;
    else if(results.second > 0){
        cout << "No path found from " << table[src_]->title << " (" << src_ << ") to " << table[dst_]->title << " (" << dst_ << ") after " << results.second << " iterations." << endl;
        t = clock() - t;
        cout << "Total time: " << (float)t / CLOCKS_PER_SEC << " seconds." << endl << endl;
    }
    else{
        int depth = 0;
        if(results.first){
            depth += results.first->size();
        }
        cout << "Found path from " << table[src_]->title << " (" << src_ << ") to " << table[dst_]->title << " (" << dst_ << ") in " << depth << " iterations" << endl;
        cout << "\t" << setw(pad_length) << src_ << "  =  " << table[src_]->title << "*\n";
        if(results.first){
            for(res_itr = results.first->begin(); res_itr != results.first->end(); res_itr++){
                cout << "\t" << setw(pad_length) << *res_itr << "  =  " << table[*res_itr]->title << endl;
            }
        }
        cout << "\t" << setw(pad_length) << dst_ << "  =  " << table[dst_]->title << "*\n";
        t = clock() - t;
        cout << "Total time: " << (float)t / CLOCKS_PER_SEC << " seconds." << endl << endl;
        //delete results;
        return;
    }
}



pair<shared_ptr<list<unsigned int>>, int> Table::seek_links(unsigned int src, unsigned int dst){
    //return pair<list<unsigned int>*, int>(NULL, 0);
    //structures store < next step's hash, previous hashes >
    //apparently stl classes weren't really meant to store pointers, as are not native ways to delete structures of dynamically-allocated structures (aside from manually), so I'm just going to practice using smart pointers.
    shared_ptr<map<unsigned int, list<unsigned int>*>> link_children(new map<unsigned int, list<unsigned int>*>);
    shared_ptr<map<unsigned int, list<unsigned int>*>> link_grandchildren(new map<unsigned int, list<unsigned int>*>);
    shared_ptr<set<unsigned int>> seen_links(new set<unsigned int>);

    map<unsigned int, list<unsigned int>*>::iterator entry_itr;
    pair<unsigned int, list<unsigned int>*> *link_entry;
    list<unsigned int>::iterator link_itr;

    list<unsigned int> node_links;
    //list<unsigned int> *parent_path = NULL;
    //list<unsigned int> *child_path = NULL;
    shared_ptr<list<unsigned int>> parent_path, child_path;

    if(src == dst){
        //finished after 0 iterations
        return pair<shared_ptr<list<unsigned int>>, int>(NULL, 0);
    }

    //to start, insert all the source's links into the structure
    //the source shouldn't be stored in all path lists redundantly
    if(table[src]->links.empty()){
        //article has no links
        return pair<shared_ptr<list<unsigned int>>, int>(NULL, -1);
    } else {
        node_links = table[src]->links;
        for(link_itr = node_links.begin(); link_itr != node_links.end(); link_itr++){
            if(*link_itr == dst){
                //finished after 1st iteration
                return pair<shared_ptr<list<unsigned int>>, int>(NULL, 0);
            } else if(*link_itr != src){
                link_entry = new pair<unsigned int, list<unsigned int>*>(*link_itr, new list<unsigned int>);
                link_children->insert(*link_entry);
            }
        }
    }
    
    //populate each iteration of links
    for(unsigned int i = 0; i < MAX_DEPTH; i++){
        //loop between items in row
        for(entry_itr = link_children->begin(); entry_itr != link_children->end(); entry_itr++){
            //parent_path.reset(entry_itr->second);
            parent_path = shared_ptr<list<unsigned int>>(entry_itr->second);
            node_links = table[entry_itr->first]->links;
            //loop between links on a page
            for(link_itr = node_links.begin(); link_itr != node_links.end(); link_itr++){
                //add iff link hasn't been seen already
                if(seen_links->find(*link_itr) == seen_links->end()){
                    child_path = shared_ptr<list<unsigned int>>(new list<unsigned int>(*parent_path));
                    //child_path.reset(new list<unsigned int>(*parent_path));
                    child_path->push_back(entry_itr->first);
                    if(*link_itr == dst){
                        //delete link_children;
                        //delete link_grandchildren;
                        //delete seen_links;
                        //delete parent_path;
                        //delete link_entry;
                        //delete child_path;
                        //cout << "done" << endl;
                        //link_children.reset();
                        //link_grandchildren.reset();
                        //seen_links.reset();
                        return pair<shared_ptr<list<unsigned int>>, int>(child_path, 0);
                    }
                    link_grandchildren->insert(pair<unsigned int, list<unsigned int>*>(*link_itr, child_path.get()));
                }
            }
            //delete parent_path;
        }
        if(link_grandchildren->empty()){
            //new list is empty (unlikely)
            return pair<shared_ptr<list<unsigned int>>, int>(NULL, -1);
        }
        //move on to next iteration
        for(entry_itr = link_children->begin(); entry_itr != link_children->end(); entry_itr++){
            seen_links->insert(entry_itr->first);
        }
        swap(link_children, link_grandchildren);
        //delete link_grandchildren;
        //link_grandchildren = new map<unsigned int, list<unsigned int>*>;
        link_grandchildren->clear();
    }
    return pair<shared_ptr<list<unsigned int>>, int>(NULL, MAX_DEPTH);
}

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

void Table::read(string file){
    //abandoned pthread because this is a member function
    //currently no mutex; infinitesimally small chance of conflict at the moment.
    ifstream f_in((char*)file.c_str());
    string line;
    unsigned int addr, articles, lines, link_addr;
    articles = lines = 0;
    while(getline(f_in, line)){
        lines++;
        if(line == "<page>"){
            getline(f_in, line);
            addr = resolve_collisions(line);
            articles++;
        } else {
            link_addr = resolve_collisions(line);
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

unsigned int Table::resolve_collisions(const string &title, bool create){
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
        if(table[hash] == NULL){
            //create if not exist
            mtx[hash % NUM_MUTEX].lock();
            if(table[hash] != NULL){
                //something changed in the last few instructions
                //free up mutex so that other thread can finish and retry this function
                mtx[hash % NUM_MUTEX].unlock();
                return resolve_collisions(title);
            } else {
                if(create){
                    table[hash] = new Entry(title);
                }
                mtx[hash % NUM_MUTEX].unlock();
                if(multiplier>max_iters){    max_iters = multiplier;  }
                collisions += multiplier;
                return hash;
            }
        } else if(table[hash]->title == title){
            if(multiplier>max_iters){    max_iters = multiplier;  }
            collisions += multiplier;
            return hash;
        } else {
            collisions++;
            offset = (offset - 1)*multiplier + 1;
            assert(offset != 1);
            hash += offset;
        }
    }
    assert(MAX_ITERS != MAX_ITERS);
}


