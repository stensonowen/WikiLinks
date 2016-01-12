
#include<iostream>
#include<vector>
#include<set>
#include<cassert>
#include<memory>
#include "entry.h"
using namespace std;

class Path{
  private:
    vector<unsigned int> nodes;
  public:
    Path(){};
    ~Path(){};
    Path(const unsigned int n){ nodes.push_back(n); }
    //Path(const vector<unsigned int> n) : nodes(n) {}
    //void add(unsigned int child){ nodes.push_back(child); }
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
    unsigned int size(){ return nodes.size(); }
    unsigned int operator[](unsigned int n){ return nodes[n]; }
};


class BFS{
  private:
    Entry ** table;
    unsigned int src, dst;
    Path path;
    int code;
    set<unsigned int> seen;
    vector<Path> *nodes, *tmp;
    //nodes represents all new elements to search, i.e. bottom row in tree
    //tmp is running total of nodes' children until it's filled and becomes 'nodes'
    void iterate();
    void clear();
   public:
    BFS(Entry ** t, unsigned int s, unsigned int d);// : table(t), src(s), dst(d) {};
    pair<Path,int> SHP();
};

BFS::BFS(Entry ** t, unsigned int s, unsigned int d) : table(t), src(s), dst(d) {
    nodes = new vector<Path>;
    tmp = NULL;
    code = 0;
}

void BFS::iterate(){
    Path *p = NULL, q;
    //list<unsigned int> *tmp_links;
    unsigned int *tmp_links, link;
    tmp = new vector<Path>;
    //cycle through *nodes and add all children to *tmp (if not dst), then swap
    //cout << (nodes->size() == false) << endl;
    for(unsigned long i=0; i<nodes->size(); i++){
        p = &nodes->at(i);
        //tmp_links = &(table[p->get_destination()]->links);
        //for(list<unsigned int>::iterator j = tmp_links->begin(); j != tmp_links->end(); j++){
        for(unsigned int j=0; j<table[p->get_destination()]->links.size(); j++){
            //link = tmp_links[j];
            link = table[p->get_destination()]->links[j];
            if(link == dst){
                code = 1;   //1 correlates to the step before iterate() is run
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

pair<Path,int> BFS::SHP(){
    if(src == dst) code = 1;
    seen.insert(src);
    nodes->push_back(Path(src));
    for(int i=0; i<MAX_DEPTH; i++){
        if(nodes->empty()) code = -1;
        if(code > 0){
            code = i;
            break;
        }
        iterate();
    }
    clear();
    return pair<Path,int>(path,code);
}

void BFS::clear(){
    if(nodes) delete nodes;
    if(tmp) delete tmp;
}
