#include "table.h"
#include<iostream>
#define DEBUG 1
//python parsr8_1.py misc_data/ssimplewiki-20151226-pages-articles
//g++ -std=c++11 table_test.cpp -pthread
//./a.out ../misc_data/simplewiki-20151226-pages-articles_out

int main(int argc, char* argv[]){
    if(argc == 1){
        cout << "Usage: \"" << "WikiLinkr.exe" << " path_to_parsed_file.txt\"" << endl;
        cout <<"(You must first run \"python parsr8.py path_to_wikipedia_dump.xml path_to_parsed_file.txt\")" << endl;
        exit(1);
    }
    //Table t((char*)"../misc_data/simplewiki-20151020_out2");
    Table t(argv[1]);
    //t.details();
    if(argc == 4){
        string a(argv[2]);
        string b(argv[3]);
        t.printPath(a, b);
    }
    //getchar();
    return 0;
}
