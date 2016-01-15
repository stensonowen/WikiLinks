#include<iostream>
#include "table.h"
//python parsr8_1.py misc_data/simplewiki-20151226-pages-articles
//g++ -std=c++11 main.cpp -pthread -Wall -Wextra -m64
//./a.out ../misc_data/simplewiki-20151226-pages-articles_out0

int main(int argc, char* argv[]){
    if(argc != 2){
        std::cerr << "usage: WikiLinks.out path_to_parsed_file" << std::endl;
        std::cerr <<"(You must first run parsr8)" << std::endl;
        exit(1);
    }
    Table t(argv[1]);
    t.details();
    std::string src, dst;
    while(true){
        std::cout << "Enter src: \t";
        getline(std::cin, src);
        if(src.empty()) break;
        std::cout << "Enter dst: \t";
        getline(std::cin, dst);
        if(dst.empty()) break;
        t.printPath(src, dst);
    }
    return 0;
}
