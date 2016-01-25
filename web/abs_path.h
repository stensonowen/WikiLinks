#ifndef _abs_path_h_
#define _abs_path_h_

#include<iostream>
#include<vector>
#include "../src/BFS.h"

class Abs_path{
    //holds path as well as metadata
    //represents one row in the database
    public:
        std::string src, dst, last;
        Path path;
        int id, code, count;
        Abs_path(){ id=-1; };
        //~Abs_path();
        void add(unsigned int x){ 
            path.add(x); 
        }
        Abs_path(int len, char **data){
            id = atoi(data[0]);
            src = data[1];
            dst = data[2];
            for(int i=3; i<13; i++){
                int tmp;
                if(data[i]){
                    tmp = atoi(data[i]);
                    if(tmp == -1) break;
                    else add(tmp);
                }
            }
            last = data[13];
            code = atoi(data[14]);
            count = atoi(data[15]);
        }
        Abs_path(const Abs_path &o):src(o.src), dst(o.dst), path(o.path), last(o.last) {
            id = o.id;
            code = o.code;
            count = o.count;
        }
        Abs_path& operator=(const Abs_path &other){
            id = other.id;
            src = other.src;
            dst = other.dst;
            path = other.path;
            last = other.last;
            code = other.code;
            count = other.count;
        }
};

#endif
