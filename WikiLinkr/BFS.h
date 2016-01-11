
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
    //vector<unsigned int> get(){ return nodes; }
    unsigned int size(){ return nodes.size(); }
    unsigned int operator[](unsigned int n){ return nodes[n]; }
};


class BFS{
  private:
    Entry ** table;
    unsigned int src, dst;
    Path *path;
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
    Path* SHP();
    void print(){
        cout << "Path from " << table[src]->title << " to " << table[dst]->title << ":\n";
        for(int i=0; i<path_.size(); i++){
            cout << "\t" << table[path_[i]]->title << "\t=  " << path_[i] << ":\n";
        }
    }
};

BFS::BFS(Entry ** t, unsigned int s, unsigned int d) : table(t), src(s), dst(d) {
    nodes = new vector<Path>;
    path = NULL;
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
                //clear();
                return 1;
            } else if(seen.find(*j) == seen.end()){
                seen.insert(*j);
                q = Path(*p, *j);
                tmp->push_back(q);
            }
        }
        //delete p;
    }
    //clear up nodes
    //delete nodes;
    nodes->clear();
    swap(nodes, tmp);
    //tmp = new vector<Path>;
    return 0;
}

Path* BFS::SHP(){
    if(src == dst){
        code = 0;
        cout << "found" << endl;
        path = new Path(dst);
        clear();
        return path;
    }
    seen.insert(src);
    nodes->push_back(Path(src));
    int status = 0;
    for(int i=0; i<10; i++){
        if(status){
            clear();
            return path;
        }
        else status = iterate();
    }
}

void BFS::clear(){
    if(nodes) delete nodes;
    if(tmp) delete tmp;
}
