#ifndef _TABLE_H_
#define _TABLE_H_

//printing/formatting
#include<iostream>
#include<fstream>
#include<algorithm>
#include<iomanip>
//stl
#include<vector>
#include<set>
//misc
#include<tr1/functional>
#include<time.h>
//threading
#include<thread>
#include<mutex>
//headers
#include "BFS.h"    //contains entry.h

class Table{
    private:
        std::tr1::hash<std::string> str_hash;
        Entry ** table;             //array itself
        std::mutex mtx[NUM_MUTEX];
        long collisions;
        long max_iters;
        void populate(std::vector<std::string> files);        //multithread master
        void read(std::string file);    //multithread slave
    public:
        unsigned int entries;       //number of entries
        unsigned int size;          //number of entries and blanks
        Table(char *input_file);
        ~Table();
        unsigned int resolve_collisions(const std::string &title, int = -1);
        void printPath(std::string src, std::string dst);
        std::string htmlPath(std::string src, std::string dst);
        void details();
        std::string htmlPath(const Path &path);
        std::string contains(std::string src,std::string dst,unsigned int &src_,unsigned int &dst_);
        Path search(unsigned int src, unsigned int dst) const;
};

Path Table::search(unsigned int src, unsigned int dst) const{
    BFS *bfs = new BFS(table, src, dst);
    std::pair<Path, int> results = bfs->SHP();
    delete bfs;
    return results.first;
}

std::string Table::contains(std::string src, std::string dst,
        unsigned int &src_, unsigned int &dst_){
    //returns string indicating results of a lookup of src and dst strings
    //if there is a problem with the lookup, then a description is returned
    //if the lookup completes successfully, src_ and dst_ are updated 
    // and an empty string is returned.
    if(src.empty() && dst.empty()) return "Invalid source and destination";
    else if(src.empty()) return "Invalid source";
    else if(dst.empty()) return "Invalid destination";

    transform(src.begin(), src.end(), src.begin(), ::toupper);
    transform(dst.begin(), dst.end(), dst.begin(), ::toupper);
    unsigned int _src = resolve_collisions(src);
    unsigned int _dst = resolve_collisions(dst);
    if(!table[_src] && !table[_dst]) return "Couldn't find either source or destination";
    else if(!table[_src]) return "Couldn't find source";
    else if(!table[_dst]) return "Couldn't find destination";
    else{
        //hash values are valid; update inputs and return nothing
        src_ = _src;
        dst_ = _dst;
        return "";
    }
}

std::string Table::htmlPath(const Path &path){
    //convert a vector of hashes into a formatted string
    //TODO: corner case: no viable path: path.empty() <=> \neg\exists path ?
    std::string directions;
    for(unsigned int i=0; i<path.size(); i++)
        directions += "&nbsp;" + table[path[i]]->title + "<br>";
    return directions;
}

void Table::details(){
    std::cout << "Table details:" << std::endl;
    std::cout << "Size: " << size << "    Entries: " << entries << std::endl;
    unsigned int l = 0;
    for(unsigned int i=0; i<size; i++)
        if(table[i])
            l += table[i]->links.size();
    std::cout << "Total number of links: " << l << std::endl;
}

std::string Table::htmlPath(std::string src, std::string dst){
    if(src.empty() && dst.empty()) return "";
    else if(src.empty()) return "Invalid source";
    else if(dst.empty()) return "Invalid destination";

    transform(src.begin(), src.end(), src.begin(), ::toupper);
    transform(dst.begin(), dst.end(), dst.begin(), ::toupper);
    unsigned int src_ = resolve_collisions(src);
    unsigned int dst_ = resolve_collisions(dst);
    if(!table[src_] && !table[dst_]) return "Couldn't find either source or destination";
    else if(!table[src_]) return "Couldn't find source";
    else if(!table[dst_]) return "Couldn't find destination";

    BFS *bfs = new BFS(table, src_, dst_);
    std::pair<Path, int> results = bfs->SHP();
    delete bfs;

    if(results.second == -1) return "No such path exists";
    else if(results.second == 0){
        return "No path found after " + std::to_string(MAX_DEPTH) + " iterations";
    } else {
        std::string directions = "\n";
        for(unsigned int i=0; i<results.first.size(); i++)
            directions += "\t\t&nbsp;" + table[results.first[i]]->title + "<br>\n";
        return directions;
    }
}

void Table::printPath(std::string src, std::string dst){
    clock_t t = clock();
    //capitalize
    transform(src.begin(), src.end(), src.begin(), ::toupper);
    transform(dst.begin(), dst.end(), dst.begin(), ::toupper);
    unsigned int src_ = resolve_collisions(src);
    unsigned int dst_ = resolve_collisions(dst);
    int pad_length = std::log10(size) + 1;
    if(!table[src_]) std::cout << "Cannot find article \"" << src << "\" (" << src_ << ")\n";
    if(!table[dst_]) std::cout << "Cannot find article \"" << dst << "\" (" << dst_ << ")\n";
    if(!table[src_] || !table[dst_]) return;

    BFS *bfs = new BFS(table, src_, dst_);
    std::pair<Path,int> results = bfs->SHP();
    delete bfs;
    
    if(results.second == -1){
        std::cout << "No path exists from " << table[src_]->title << " (" << src_ 
             << ") to " << table[dst_]->title << " (" << dst_ << ")" << std::endl;
    }
    else if(results.second == 0){
        std::cout << "No path found from " << table[src_]->title << " (" << src_ 
            << ") to " << table[dst_]->title << " (" << dst_ 
            << ") after " << MAX_DEPTH << " iterations." << std::endl;
    }
    else{
        std::cout << "Found path from " << table[src_]->title << " (" << src_ 
            << ") to " << table[dst_]->title << " (" << dst_ 
            << ") in " << results.second << " iterations" << std::endl;
        unsigned int hash;
        for(unsigned int i=0; i<results.first.size(); i++){
            hash = results.first[i];
            std::cout << "\t" << std::setw(pad_length) << hash << "  =  " << table[hash]->title << std::endl;
        }
    }
    if(results.second >= 0){
        t = clock() - t;
        std::cout << " Search time: " << (float)t / CLOCKS_PER_SEC << " seconds.\n";
    }
}

void Table::read(std::string file){
    //format should be <page> \n title \n #_links \n links...
    std::ifstream f_in((char*)file.c_str());
    std::string line, title;
    unsigned int addr, link_addr;
    while(getline(f_in, line)){
        if(line == "<page>"){
            getline(f_in, title);
            getline(f_in, line);
            addr = resolve_collisions(title, atoi(line.c_str()));
        } else {
            link_addr = resolve_collisions(line, 0);
            table[addr]->links.push_back(link_addr);
        }
    }
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
    clock_t t = clock();
    std::string tmp;
    getline(input, tmp);
    entries = atoi(tmp.c_str());
    size = 20 * entries + 1000;     //equation subject to optimization
    table = new Entry*[size];
    for(unsigned int i=0; i<size; i++)
        table[i] = NULL;
    //copy files
    std::vector<std::string> threads;
    while(getline(input, tmp)){
        threads.push_back(tmp);
    }
    populate(threads);
    t = clock() - t;
    std::cout << " Load time: " << (float)t / CLOCKS_PER_SEC << " seconds (user time).\n";
}

void Table::populate(std::vector<std::string> files){
    //create and run threads to populate table
    //remember to add mutex system
    std::vector<std::thread> threads;
    for(unsigned int i=0; i<files.size(); i++){
        threads.push_back(std::thread(&Table::read, this, files[i]));
    }
    std::cout << "Starting " << threads.size() << " threads." << std::endl;
    for(unsigned int i=0; i<threads.size(); i++){
        threads[i].join();
    }
}

Table::~Table(){
    //"total heap usage: 7,277,425 allocs, 7,277,425 frees, 247,320,661 bytes allocated"
    //No leaks
    for(unsigned int i=0; i<size; i++){
        if(table[i]){
            delete table[i];
        }
    }
    delete[] table;
}

unsigned int Table::resolve_collisions(const std::string &title, int links){
    //using factorial-based generates primes more than squares
    //perhaps this isn't actually all that great. it finds primes relatively often,
    //  but they grow extremely quickly, which destroys spatial locality
    //  A132199 (https://en.wikipedia.org/wiki/Formula_for_primes#Recurrence_relation)
    //      could store first few dozen and generate more as needed
    //      if values were stored it would be quite fast
    //      if 1's were excluded, it would generate small primes consistently
    //links:
    //  >=0 intended size of entry with specified title
    //  -1  default: do not alter entry with this title
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
                if(links >= 0){
                    table[hash] = new Entry(title, links);
                }
                mtx[hash % NUM_MUTEX].unlock();
                if(multiplier>max_iters){    max_iters = multiplier;  }
                collisions += multiplier;
                return hash;
            }
        } else if(table[hash]->title == title){
            //resize as necessary
            if(links >= 0){
                mtx[hash % NUM_MUTEX].lock();
                table[hash]->resize(links);
                mtx[hash % NUM_MUTEX].unlock();
            }
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

#endif
