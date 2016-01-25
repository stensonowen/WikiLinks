#ifndef _entry_h_
#define _entry_h_

#include<iostream>
#include<vector>

#define MAX_DEPTH 10 
#define MAX_ITERS 100
#define NUM_MUTEX 50
#define UNIT 10

class Entry{
    public:
        //Entry will be on heap, so don't need pointer member vars
        std::string title;
        std::vector<unsigned int> links;
        void resize(int s){ links.reserve(s); }
        Entry() {}
        Entry(const std::string &t, int s=0) : title(t) {
            links.reserve(s);
            //links.shrink_to_fit();
        }
        ~Entry() {}
};

#endif
