
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
    Path path_;
    int code;
    set<unsigned int> seen;
    vector<Path> *nodes, *tmp;
    //nodes represents all new elements to search, i.e. bottom row in tree
    //tmp is running total of nodes' children until it's filled and becomes 'nodes'
    int iterate();
    void clear();
   public:
    BFS(Entry ** t, unsigned int s, unsigned int d);// : table(t), src(s), dst(d) {};
    Path SHP();
    void print(){
        cout << "Path from " << table[src]->title << " to " << table[dst]->title << ":\n";
        for(int i=0; i<path_.size(); i++){
            cout << "\t" << table[path_[i]]->title << "\t=  " << path_[i] << ":\n";
        }
    }
};

BFS::BFS(Entry ** t, unsigned int s, unsigned int d) : table(t), src(s), dst(d) {
    nodes = new vector<Path>;
    tmp = NULL;
    cout << "New BFS: \n\tsrc : " << table[src]->title << "\n\tdst : " << table[dst]->title << endl;
}

int BFS::iterate(){
    Path *p = NULL, q;
    list<unsigned int> *tmp_links;
    tmp = new vector<Path>;
    //cycle through *nodes and add all children to *tmp (if not dst), then swap
    for(long i=0; i<nodes->size(); i++){
        p = &nodes->at(i);
        tmp_links = &(table[p->get_destination()]->links);
        for(list<unsigned int>::iterator j = tmp_links->begin(); j != tmp_links->end(); j++){
            if(*j == dst){
                path_ = Path(*p, *j);
                return 1;
            } else if(seen.find(*j) == seen.end()){
                seen.insert(*j);
                q = Path(*p, *j);
                tmp->push_back(q);
            }
        }
    }
    delete nodes;
    swap(nodes, tmp);
    return 0;
}

Path BFS::SHP(){
    if(src == dst){
        code = 0;
        cout << "found" << endl;
        clear();
        return path_;
    }
    seen.insert(src);
    nodes->push_back(Path(src));
    int status = 0;
    for(int i=0; i<5; i++){
        status = iterate();
        if(status){
            clear();
            return path_;
        }
    }
    if(nodes) delete nodes;
    cout << "nope" << endl;
    return Path();
}

void BFS::clear(){
    if(nodes) delete nodes;
    if(tmp) delete tmp;
}
