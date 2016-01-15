#include "table.h"
#include<iostream>
//python parsr8_1.py misc_data/simplewiki-20151226-pages-articles
//g++ -std=c++11 table_test.cpp -pthread -Wall -Wextra -m64
//./a.out ../misc_data/simplewiki-20151226-pages-articles_out0

int main(int argc, char* argv[]){
    if(argc != 2){
        cout << "usage: WikiLinks.out path_to_parsed_file" << endl;
        cout <<"(You must first run parsr8.py)" << endl;
        exit(1);
    }
    Table t(argv[1]);
    //t.details();
    string src, dst;
    while(true){
        cout << "Enter src: \t";
        getline(cin, src);
        if(src.empty()) break;
        cout << "Enter dst: \t";
        getline(cin, dst);
        if(dst.empty()) break;
        t.printPath(src, dst);
    }
    return 0;
}
