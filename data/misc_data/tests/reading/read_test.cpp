/*Threaded file reading tests
 * used to determine whether the bottleneck when reading is the CPU or SSD
 *  the result will determine whether there is realistic speedup from multithreading
 * values were generated in python (random ints < 500k) * 2M
 *  split up with head/tail into halves
 *  read in and used briefly to simulate real program
 *  broken into threads and repeated /timed
 * threaded version were significantly faster, meaning threading would result in speedup *
 */

#include<iostream>
#include<fstream>
#include<set>
#include<pthread.h>
#include<unistd.h>
#include<vector>
#include<thread>
using namespace std;

void *readAllInOne(void *f_){
    /* all 2M lines read in one proc
     * time
     *  real    0m2.904s
     *  user    0m2.881s
     *  sys     0m0.020s
     */
    char *f = (char*)f_;
    ifstream input(f);
    int lines = 0;
    set<string> seen;
    string tmp;
    while(getline(input, tmp)){
        lines++;
        seen.insert(tmp);
    }
    printf("%d unique of %d (%2.0f%%)\n", (int)seen.size(), lines, (float)seen.size()/lines*100);
    pthread_exit(NULL);
}

void read2(char* f){
    ifstream input(f);
    set<string> seen;
    int lines = 0;
    string tmp;
    while(getline(input, tmp)){
        lines++;
        seen.insert(tmp);
    }
    printf("%d unique of %d (%2.0f%%)\n", (int)seen.size(), lines, (float)seen.size()/lines*100);

}

void readInThread(vector<string> files){
    /*  One thread, 2M lines
     *      real    2.896s
     *      user    2.868s
     *      sys     0,024s
     *  Two threads, 2M lines each
     *      real    3.199s
     *      user    6.298s
     *      sys     0,068s
     *  Two threads, 1M lines each
     *      real    1.651s 
     *      user    3.242s
     *      sys     0.020s
     *  Four threads, 1M lines each
     *      reali   1.845s
     *      user    7.135
     *      sys     0.068s
    */
    char *c = (char*)files[0].c_str();
    ifstream input(c);
    vector<pthread_t> threads;
    threads.resize(files.size());

    //pthread_attr_t attr;
    //pthread_attr_init(&attr);
    //pthread_attr_setdetachstate(&attr, PTHREAD_CREATE_JOINABLE);

    for(int i=0; i<files.size(); i++){
        char *c = (char*)files[i].c_str();
        //printf(" %d: %d\n", i, pthread_create(&threads[i], NULL, readAllInOne, NULL));
        printf(" %d: %d\n", i, pthread_create(&threads[i], NULL, readAllInOne, (void*)c));
    }
    //pthread_attr_destroy(&attr);
    void *status;
    for(int i=0; i<files.size(); i++){
        pthread_join(threads[i], &status);
        cout << " " << i << ": " << status << endl;
    }
    pthread_exit(NULL);

}

int main(){
    char c[] = "data_2M";
    string s = "data_2M";
    //readAllInOne(&c);
    vector<string> files;
    char c1[] = "data_2M_1";
    char c2[] = "data_2M_2";
//    files.push_back("data_2M_1");
//    files.push_back("data_2M_2");
    //files.push_back("data_2M");
    //files.push_back("data_2M");
    //files.push_back("data_2M");
    //files.push_back("data_2M");
    //readInThread(files);
    //read2(c);
    thread a(read2,c1);
    thread b(read2,c2);
    a.join();
    b.join();
    return 0;
}
