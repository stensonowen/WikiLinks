#include<iostream>
#include "queue.h"
using namespace std;
int main(){
    //cout << "hw" << endl;
    string f = "../data/simplewiki-20151020-pages-articles_out0";
    Table t((char*)f.c_str());
    Queue queue(&t);
    Path path = queue.enqueue(1953050, 576805);    
    cout << t.htmlPath(path) << endl;

    return 0;
}
