/*REQUIREMENTS
 * hold complex object: essentially a bunch of threads
 * searchable: determine if request is already being processed
 * a request should be popped after it finishes: FI?O (a list?)
 * custom object, I guess? must contain a thread object
 * additional set to keep track of src/dst values
 */

#include<iostream>
#include<set>
#include<list>
#include<thread>
#include "../src/table.h"
using namespace std;

#define SEARCH_THREADS 2

class Request{
    private:
        unsigned int src, dst;
        const Table *table;
    public:
        std::thread proc;
        Path *path;
        bool running;
        Request(){};
        Request(const Table *t, const unsigned int s, const unsigned int d){
            src = s;
            dst = d;
            running = false;
            path = NULL;
            table = t;
        }
        bool equals(const unsigned int s, const unsigned int d){
            return src == s && dst == d;
        }
        void search(){
            running = true;
            *path = table->search(src, dst);
            running = false;
        }
        void start(){
            proc = thread(&Request::search);
        }
        pair<unsigned int, unsigned int> value(){
            return pair<unsigned int, unsigned int>(src,dst);
        }
};

class Queue{ //rename due to stl conflict?
    private:
        //need quickly searchable list of in-progress elements
        //will have to O(n) search through to actually find the right one,
        // but this will make eliminating options fast
        set<pair<unsigned int, unsigned int>> directory;
        //structure to actually store in-progress requests
        //must be fast/efficient/safe to pop nodes from anywhere
        list<Request> requests;
        //needs handle to table to pass on to request
        const Table *table;
        //not sure if this is necessary 
        mutex mtx;
        int live;
    public:
        Queue(const Table *t){ table = t; }
        Path push_back(Request &request);
        Path emplace(const unsigned int s, const unsigned int d);
        void update();
};
        
void Queue::update(){
    //remove all finished jobs;
    for(list<Request>::iterator itr = requests.begin(); itr != requests.end(); itr++){
        if(!itr->running){
            if(itr->path){
                //finished
                mtx.lock();
                requests.erase(itr);
                live--;
                mtx.unlock();
            } else if(live < SEARCH_THREADS){
                //not yet started and not at quota
                mtx.lock();
                itr->start();
                live++;
                mtx.unlock();
            }
        }
        //if it's running leave it alone
    }
}

Path Queue::push_back(Request &request){
    if(directory.find(request.value()) == directory.end()){
        //a process with the same return value already exists
        //find it:
        for(list<Request>::iterator itr = requests.begin(); itr != requests.end(); itr++){
            if(itr->value() == request.value()){
                //presumably if a thread is finished then .join() won't hurt
                itr->proc.join(); 
                return *(itr->path);
            }
        }
    }
    //else omitted intentionally: either way the right thread was not found
    //create new entry
    mtx.lock();
    directory.insert(request.value());
    requests.push_back(request);
    mtx.unlock();
    //start process if necessary
    update();
    //now wait
    request.proc.join();    //problem joining an unstarted thread?
    return *(request.path); //problem joining to param and not vector element?
}

Path Queue::emplace(const unsigned int s, const unsigned int d){
    Request request(table, s, d);
    push_back(request);
    //push_back called .join()
    return *(request.path);
}




int main(int argc, char* argv[]){
    cout << "hw" << endl;
    return 0;
}
