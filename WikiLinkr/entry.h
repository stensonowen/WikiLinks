#include<iostream>
#include<stdlib.h>
#include<list>
#include<string>

#define MAX_DEPTH 10
#define MAX_ITERS 100
#define NUM_MUTEX 100

class Entry{
    public:
        //Entry will be on heap, so don't need pointer member vars
        std::string title;
        std::list<unsigned int> links;
        Entry() {}
        Entry(const std::string &t) : title(t) {}
        //Entry(string &t, list<unsigned int> &l) : title(t), links(l) {}
        ~Entry() {}
};
