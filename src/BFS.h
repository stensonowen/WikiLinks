#ifndef _bfs_h_
#define _bfs_h_

#include<iostream>
#include<vector>
#include<set>
#include<cassert>
#include "entry.h"

class Path{
    private:
        std::vector<unsigned int> nodes;
    public:
        Path(){};
        ~Path(){};
        Path(const unsigned int n){ nodes.push_back(n); }
        //Path(const vector<unsigned int> n) : nodes(n) {}
        void add(unsigned int child){ nodes.push_back(child); }
        unsigned int get_destination(){
            assert(nodes.size() > 0);
            return nodes[nodes.size()-1];
        }
        Path(const Path &other, const unsigned int child){
            nodes = other.nodes;
            nodes.push_back(child);
        }
        Path &operator=(const Path &other){
            nodes = other.nodes;
            return *this;
        }
        const unsigned int size() const{ return nodes.size(); }
        unsigned int operator[](unsigned int n) const{ return nodes[n]; }
        bool empty() const{ return nodes.empty(); }
};


class BFS{
    private:
        Entry ** table;
        unsigned int src, dst;
        Path path;
        int code;
        int max_depth;
        std::set<unsigned int> seen;
        std::vector<Path> *nodes, *tmp;
        //nodes represents all new elements to search, i.e. bottom row in tree
        //tmp is running total of nodes' children until it's filled and becomes 'nodes'
        void iterate();
        void clear();
    public:
        BFS(Entry ** t, unsigned int s, unsigned int d, int md=MAX_DEPTH);// : table(t), src(s), dst(d) {};
        std::pair<Path,int> SHP();
};

BFS::BFS(Entry ** t, unsigned int s, unsigned int d, int md) : table(t), src(s), dst(d), max_depth(md) {
    nodes = new std::vector<Path>;
    tmp = NULL;
    code = 0;
}

void BFS::iterate(){
    Path *p = NULL, q;
    unsigned int link;
    tmp = new std::vector<Path>;
    //cycle through *nodes and add all children to *tmp (if not dst), then swap
    for(unsigned long i=0; i<nodes->size(); i++){
        p = &nodes->at(i);
        for(unsigned int j=0; j<table[p->get_destination()]->links.size(); j++){
            link = table[p->get_destination()]->links[j];
            if(link == dst){
                code = 0;   //signal to make code equal to iteration number
                path = Path(*p, link);
                return;
            } else if(seen.find(link) == seen.end()){
                seen.insert(link);
                q = Path(*p, link);
                tmp->push_back(q);
            }
        }
    }
    delete nodes;
    swap(nodes, tmp);
    tmp = NULL;
    return;
}

std::pair<Path,int> BFS::SHP(){
    /*code
     * -2   provably impossible
     * -1   in progress
     *  0   success
     * n>0  terminated after n iterations
     *
     */
    code = -1;  //in progress
    if(src == dst) code = 0;
    unsigned int i;
    seen.insert(src);
    nodes->push_back(Path(src));
    for(i=1; i<=max_depth; i++){
        if(nodes->empty())  code = -2;  //no solution
        if(code != -1)      break;
        iterate();
    }
    if(code == -1) code = i;
    clear();
    //TODO
    //for(int i=0; i<path.size(); i++) std::cout << "  " << path[i];
    //std::cout << "\t\t" << code << std::endl;
    return std::pair<Path,int>(path,code);
}

void BFS::clear(){
    if(nodes) delete nodes;
    if(tmp) delete tmp;
}

#endif
