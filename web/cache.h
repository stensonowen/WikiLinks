//#include<iostream>
#include<sqlite3.h>
#include<string>
//#include<vector>
#include<time.h>
#include "abs_path.h"
//#include "../src/BFS.h"
using namespace std;
        

class Cache{
    //wrapper for SQL database of searches
    private:
        sqlite3 *db;
        char* err;
        int rc;
        int status;
        unsigned int size;
        static int select_callback(void *vec, int count, char **columns, char **column);
        static int size_callback(void *s, int count, char **columns, char **column);
        void verify(const char *task);
        string datetime();
    public:
        Abs_path* contains(const string &src, const string &dst);
        void insert(const string &src, const string &dst, Path path, unsigned int time);
        void update(const Abs_path &ap);
        enum sort_by { recent, popular, length };
        vector<Abs_path> retrieve(unsigned int n, sort_by category);
        //should BFS be called within this? probably not
        Cache(const string &file);
        ~Cache(){ sqlite3_close(db); }
};

string Cache::datetime(){
    //return a string of the datetime in the form "YYYY-MM-DD hh:mm"
    time_t rt;
    struct tm * timeinfo;
    char buffer[17];
    time(&rt);
    timeinfo = localtime(&rt);
    strftime(buffer,17,"%F %R",timeinfo);
    return string(buffer);
}

void Cache::verify(const char *task){
    //verify operation described by 'task' completed; otherwise print error
    if(rc != SQLITE_OK){
        cerr << "Failed at task: " << task << ": " << err << endl;
        sqlite3_free(err);
    }
}

int Cache::size_callback(void *s, int count, char **columns, char **column){
    //process command to find size of the table; int s is passed by reference in a void ptr
    assert(count == 1);
    int *size_ = (int*)s;
    *size_ = atoi(columns[0]);
    return 0;
}

int Cache::select_callback(void *v, int count, char **columns, char **column){
    //process results of 'SELECT' statement; return (by ref) vector with path info
    //cast "v" to a vector pointer, and push back this value
    vector<Abs_path>* vec = (vector<Abs_path>*)v;
    Abs_path ap(count, columns);
    vec->push_back(ap); //need to be on heap? will go out of scope?
    //for(int i=0; i<count; i++)
    //    cout << i << ": " << column[i] << ": " << (columns[i] ? columns[i] : "0x0") << endl;
    //cout << endl;
    return 0;
}

Cache::Cache(const string &file){
    //construct cache: create/open database
    size = 0;
    status = 0;
    rc = sqlite3_open(file.c_str(), &db);
    verify("Open table");
    const char* cmd = "CREATE TABLE IF NOT EXISTS CACHE("
        "ID     INT     PRIMARY KEY NOT NULL,"
        "SRC    TEXT    NOT NULL,"
        "DST    TEXT    NOT NULL,"
        "P0     INT     DEFAULT -1,"
        "P1     INT     DEFAULT -1,"
        "P2     INT     DEFAULT -1,"
        "P3     INT     DEFAULT -1,"
        "P4     INT     DEFAULT -1,"
        "P5     INT     DEFAULT -1,"
        "P6     INT     DEFAULT -1,"
        "P7     INT     DEFAULT -1,"
        "P8     INT     DEFAULT -1,"
        "P9     INT     DEFAULT -1,"
        "LAST   DATETIME NOT NULL,"
        "TIME   INT     DEFAULT 0,"
        "COUNT  INT     DEFAULT 1);";
    rc = sqlite3_exec(db, cmd, select_callback, 0, &err);
    verify("Create table"); 
    
    //get current size
    const char* cmd_ = "SELECT COUNT(*) FROM CACHE;";
    rc = sqlite3_exec(db, cmd_, size_callback, &size, &err);
    verify("Count");
}

void Cache::insert(const string &src, const string &dst, Path path, unsigned int t){
    //construct row object from src/dst/path/time and insert it into the
    //user request should never get to this point unless both src and dst are valid articles
    //if someone can pull off an sql injection/XSS using only valid wikipedia article titles, 
    // then they kind of deserve to
    string query1 = "INSERT INTO CACHE (ID,SRC,DST,";
    string query2 = "VALUES (" + to_string(size) + ",'" 
        + string(src) + "','" + string(dst) + "',";
    for(unsigned int i=0; i<path.size(); i++){
        query1 += "P" + to_string(i) + ",";
        query2 += to_string(path[i]) + ",";
    }
    query1 += "LAST,TIME) ";
    query2 += "'" + datetime() + "'," + to_string(t) + ");";
    string query(query1 + query2);
    rc = sqlite3_exec(db, query.c_str(), select_callback, 0, &err);
    verify("Insert element");
    size++;
}

vector<Abs_path> Cache::retrieve(unsigned int n, Cache::sort_by category){
    //retrieve the first n rows sorted by 'category' and return as vector
    vector<Abs_path> results;
    string cmd("SELECT * FROM CACHE ORDER BY ");
    if(category == recent){ //length popular
        cmd += "LAST DESC "; //ASC?
    } else if(category == popular){
        cmd += "COUNT DESC ";
    } else if(category == length){
        for(unsigned int i=9; i>0; i--)
            cmd += "P" + to_string(i) + " DESC,";
        cmd += "P0 DESC ";
    }
    cmd += "LIMIT " + to_string(n) + ";";
    rc = sqlite3_exec(db, cmd.c_str(), select_callback, (void*)&results, &err);
    verify("Retrieve rows");
    return results;
}

Abs_path* Cache::contains(const string &src, const string &dst){
    //check for presence of a path from src to dst;
    //theoretically there should only be one, but the first will be returned (on HEAP)
    //if none are found, should return NULL pointer 
    string cmd("SELECT * FROM CACHE where SRC='" + src + "' and DST='" + dst + "';");
    vector<Abs_path> results;
    rc = sqlite3_exec(db, cmd.c_str(), select_callback, (void*)&results, &err);
    verify("Contains");
    if(results.size() == 0) return NULL;
    //else return &(results[0]);
    /*else{
        * returning pointer on stack results in data corruption;
        * going out of scope I guess?
        cout << "1: src = " << results[0].src << ";  dst = " << results[0].dst << endl;
        Abs_path *ap;
        ap = &results[0];
        cout << "1.5: src = " << ap->src << "; dst = " << ap->dst << endl; 
        return ap;
        return new Abs_path(results[0]);
    }*/
    else return new Abs_path(results[0]);
}

void Cache::update(const Abs_path &ap){
    //use id to locate row; increment count and update 'last' with current time
    string cmd = "UPDATE CACHE set " 
        "COUNT=" + to_string(ap.count+1) + ",LAST='" + datetime() + "'"
        " where ID=" + to_string(ap.id) + ";";
    rc = sqlite3_exec(db, cmd.c_str(), select_callback, 0, &err);
    verify("Update");
}
