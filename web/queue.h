/*REQUIREMENTS
 * Should take src and dst hash as input
 * Should perform t.search() on them and return a Path
 * Should limit searches to a few threads at a time
 *
 * Optional TODO
 *  Request object with public mutex member
 *      Directory of processes could point to object: duplicate requests could join in-progress
 *      Requests could use previous requests's mutexes as a line
 *          This would actually be a queue rather than this thread-safe race condition
 */

#include<iostream>
#include<map>
#include<thread>
#include<future>
#include "../src/table.h"

using namespace std;

#define SEARCH_THREADS 2

//request class
//  have mutex and path pointer: can be used when duplicate finishes
//
/*
class Request{
    private:
        unsigned int src, dst;

    public:
        mutex mtx;
        Path *path;
        pair<unsigned int,unsigned int> value(){return pair<unsigned int,unsigned int>(src,dst);}
        Request(const unsigned int s, const unsigned int d){
            src = s;
            dst = d;
            path = NULL;
        }
};*/


class Queue{ //rename due to stl conflict?
    private:
        //map from src/dst pairs to 
        //map<pair<unsigned int,unsigned int>, *Request> in_progress;
        mutex mtx[SEARCH_THREADS];
        const Table *table;
    public:
        Queue(const Table *t){ table = t; }
        //Path push_back(Request &request);
        //Path emplace(const unsigned int s, const unsigned int d);
        //void update();
        std::pair<Path,int> enqueue(const unsigned int src, const unsigned int dst){
            std::pair<Path,int> path;
            //first try to use all threads, as empty threads should be used first
            for(unsigned int i=0; i<SEARCH_THREADS; i++){
                if(mtx[i].try_lock()){
                    //found an empty mutex
                    //silly functor workaround to get return value
                    //std::thread thread([&] {path = table->search(src, dst);} );
                    //thread.join();
                    path = table->search(src, dst);
                    mtx[i].unlock();
                    return path;
                }
            }
            //all mutexes are full
            //commit to wait for one of them
            int i = (src + dst) % SEARCH_THREADS;   //don't think overflow will matter
            mtx[i].lock();
            //std::thread thread([&] {path = table->search(src, dst);} );
            //thread.join();
            path = table->search(src, dst);
            mtx[i].unlock();
            return path;
        }
};
