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
        void increment(){x++;}
        Entry() : x(1) {}
        Entry(unsigned int y) : x(y) {}
};

void populate(atomic<Entry> **t){
    unsigned int y;
    for(int i=0; i<NUM_ADDITIONS; i++){
        y = (unsigned int) (rand() % TABLE_SIZE);
        if(t[y] == NULL){
            //t[y] = new atomic<Entry>();
        } else {
            //t[y]->increment();
        }
    }
}

class Entry2{
    public:
        atomic<unsigned int> x;
        void increment(){x++;}
        Entry2(){ x.store(1);  }
};

void populate(Entry **t){
    unsigned int y;
    for(int i=0; i<NUM_ADDITIONS; i++){
        y = (unsigned int) (rand() % TABLE_SIZE);
        if(t[y] == NULL){
            t[y] = new Entry();
        } else {
            t[y]->increment();
        }
   }
}

void populate2(Entry2 **t){
    unsigned int y;
    for(int i=0; i<NUM_ADDITIONS; i++){
        y = (unsigned int) (rand() % TABLE_SIZE);
        if(t[y] == NULL){
            t[y] = new Entry2();
        } else {
            t[y]->increment();
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

long sum2(Entry2 **t){
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
    //Entry* table[TABLE_SIZE];
    Entry2* table2[TABLE_SIZE];
    atomic<Entry>* table3[TABLE_SIZE];
    //for(int i=0; i<TABLE_SIZE; i++) table[i]=NULL;
    for(int i=0; i<TABLE_SIZE; i++) table2[i]=NULL;
    unsigned int t = 10;
    vector<std::thread> threads, threads2, threads3;
    for(int i=0; i<t; i++){
        //threads.push_back(thread(populate, table));
        threads2.push_back(thread(populate2, table2));
    }
    for(int i=0; i<threads2.size(); i++){
        //threads[i].join();
        threads2[i].join();
    }
    //cout << sum(table) << endl;
    cout << sum2(table2) << endl;
    


    return 0;
}
