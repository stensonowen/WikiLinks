
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
    Path(const vector<unsigned int> n) : nodes(n) {}
    void add(unsigned int child){ nodes.push_back(child); }
    unsigned int get_destination(){
        assert(nodes.size() > 0);
        return nodes[nodes.size()-1];
    }
    //shared_ptr<Path> generate_child(unsigned int child){
    Path* generate_child(unsigned int child){
        //shared_ptr<Path> p(new Path(nodes));
        Path *p = new Path(nodes);
        p->add(child);
        return p;
    }
    //void print(){ for(int i=0; i<nodes.size(); i++) cout << " " << nodes[i] << endl; }
    vector<unsigned int> get(){ return nodes; }
};


class BFS{
  private:
    Entry ** table;
    unsigned int src, dst;
    Path *path;
    int code;
    //shared_ptr<Path> path;
    //shared_ptr<set<unsigned int>> seen;
    set<unsigned int> seen;
    vector<Path> *nodes, *tmp;
    //nodes represents all new elements to search, i.e. bottom row in tree
    //tmp is running total of nodes' children until it's filled and becomes 'nodes'
    void iterate();
    void clear();
   public:
    BFS(Entry ** t, unsigned int s, unsigned int d);// : table(t), src(s), dst(d) {};
    //shared_ptr<Path> SHP(){ return path; }
    Path* SHP();
    void print(){
        if(path == NULL){
            cout << "No path found" << endl;
        } else {
            cout << "Path from " << table[src]->title << " to " << table[dst]->title << ":\n";
            for(int i=0; i<path->get().size(); i++){
                cout << "\t" << table[path->get()[i]]->title << "\t=  " << path->get()[i] << ":\n";
            }
            delete path;
        }
    }
};

BFS::BFS(Entry ** t, unsigned int s, unsigned int d) : table(t), src(s), dst(d) {
    nodes = new vector<Path>;
    //tmp  = new vector<Path>;
    path = NULL;
    cout << "New BFS: \n\tsrc : " << table[src]->title << "\n\tdst : " << table[dst]->title << endl;
}

void BFS::iterate(){
    Path *p = NULL, *q;
    list<unsigned int> *tmp_links;
    tmp = new vector<Path>;
    //cycle through *nodes and add all children to *tmp (if not dst), then swap
    for(long i=0; i<nodes->size(); i++){
        //cout << nodes->at(i) << endl; 
        p = &nodes->at(i);
        tmp_links = &(table[p->get_destination()]->links);
        for(list<unsigned int>::iterator j = tmp_links->begin(); j != tmp_links->end(); j++){
            if(*j == dst){
                path = p->generate_child(*j);
                clear();
                return;
            } else if(seen.find(*j) == seen.end()){
                seen.insert(*j);
                q = p->generate_child(*j);
                tmp->push_back(*q);
            }
        }
        //delete p;
    }
    //clear up nodes
    delete nodes;
    //nodes->clear();
    swap(nodes, tmp);
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
    for(int i=0; i<10; i++){
        if(path) return path;
        else iterate();
    }
}

void BFS::clear(){
    if(nodes) delete nodes;
    if(tmp) delete tmp;
   //delete nodes;
   //delete tmp;
}
