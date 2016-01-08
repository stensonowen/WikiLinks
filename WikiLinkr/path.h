#include<iostream>
#include<vector>
#include<cassert>
#include<memory>
using namespace std;

class Path{
public:
    vector<unsigned int> steps;
    unsigned int get_destination();
    shared_ptr<Path> generate_child(unsigned int child);
    Path(){};
    Path(const Path &other);
    ~Path(){};
};

unsigned int Path::get_destination(){
    if(steps.size() == 0){
        assert(false);
        return 0;
    } else {
        return steps[steps.size()-1];
    }
}

Path::Path(const Path &other){
    steps = vector<unsigned int>(other.steps);
}

shared_ptr<Path> Path::generate_child(unsigned int child){
    shared_ptr<Path> p(new Path(*this));
    p->steps.push_back(child);
    return p;
}
