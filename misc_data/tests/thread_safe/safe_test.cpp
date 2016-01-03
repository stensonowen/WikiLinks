#include<iostream>
#include<thread>
#include<stdlib.h>
#include<time.h>
#include<vector>
#include<atomic>
using namespace std;

#define TABLE_SIZE 10000
#define NUM_ADDITIONS 100000

class Entry{
    public:
        unsigned int x;
        Entry() : x(1) {}
        Entry(unsigned int y) : x(y) {}
};

void populate(Entry **t){
    unsigned int y;
    for(int i=0; i<NUM_ADDITIONS; i++){
        y = (unsigned int) (rand() % TABLE_SIZE);
        if(t[y] == NULL){
            t[y] = new Entry();
        } else {
            t[y]->x++;
        }
   }
}

long sum(Entry **t){
    long s = 0;
    for(int i=0; i<TABLE_SIZE; i++){
        if(t[i] != NULL){
            s += (t[i]->x);
        }
    }
    return s;
}

int main(){
    srand(time(NULL));
    atomic<int> a;
    //cout << "hallowed whorl" << endl;
    Entry* table[TABLE_SIZE];
    for(int i=0; i<TABLE_SIZE; i++) table[i]=NULL;
    //cout << rand() << endl;
    unsigned int t = 10;
    vector<std::thread> threads;
    for(int i=0; i<t; i++){
        threads.push_back(thread(populate, table));
    }
    for(int i=0; i<threads.size(); i++){
        threads[i].join();
    }
    cout << sum(table) << endl;
    


    return 0;
}
