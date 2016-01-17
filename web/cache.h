#include<iostream>
#include<sqlite3.h>
#include<string>
#include<vector>
#include "../src/BFS.h"
using namespace std;

class Abs_path{
    //holds path as well as metadata
    //represents one row in the database
    public:
    string src, dst;
    Path path;
    int time, count;
    Abs_path(){};
    //~Abs_path();
    //Abs_path(const string &s, const string &d, int t, int c=0):src(s), dst(d), time(t), count(c){}
    void add(unsigned int x){ path.add(x); }
    Abs_path& operator=(const Abs_path &other){//:src(other.src), dst(other.dst), path(other.path) {
        src = other.src;
        dst = other.dst;
        path = other.path;
        time = other.time;
        count = other.count;
    }
};

class Cache{
    private:
        sqlite3 *db;
        char* err;
        int rc;
        int status;
        unsigned int size;
        static int select_callback(void *vec, int count, char **columns, char **column);
        static int size_callback(void *s, int count, char **columns, char **column);
        void verify(const char *task);
    public:
        Abs_path contains(const string &src, const string &dst);
        void insert(const string &src, const string &dst, Path path, unsigned int time);
        void update(const string &src, const string &dst);
        //should BFS be called within this? probably not
        Cache(const string &file);
        ~Cache(){};
};

void Cache::verify(const char *task){
    if(rc != SQLITE_OK){
        cerr << "Failed at task: " << task << ": " << err << endl;
        sqlite3_free(err);
    }
}

int Cache::size_callback(void *s, int count, char **columns, char **column){
    cout << "size-cache" << endl;
    return 0;
}

int Cache::select_callback(void *v, int count, char **columns, char **column){
    //cast "v" to a vector pointer, and push back this value
    //MetaPath(const string &s, const string &d, int t, int c=0):src(s), dst(d), time(t), count(c){}
    //ID, SRC, DST, P0-9, Time, Count
    vector<Abs_path>* vec = (vector<Abs_path>*)v;
    Abs_path ap;
    ap.src = columns[1];
    ap.dst = columns[2];
    for(int i=3; i<13; i++){
        if(columns[i] && columns[i] != "-1"){
            ap.add(atoi(columns[i]));
        } else {
            break;
        }
    }
    ap.time = atoi(columns[13]);
    ap.count = atoi(columns[14]);
    vec->push_back(ap); //need to be on heap? will go out of scope?
    for(int i=0; i<count; i++)
        cout << column[i] << ": " << (columns[i] ? columns[i] : "0x0") << endl;
    cout << endl;
    return 0;
}

Cache::Cache(const string &file){
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
        "TIME   INT     DEFAULT 0,"
        "COUNT  INT     DEFAULT 1);";
    rc = sqlite3_exec(db, cmd, select_callback, 0, &err);
    verify("Create table"); 
}

void Cache::insert(const string &src, const string &dst, Path path, unsigned int time){
    //request should never get to this point unless both src and dst are valid articles
    //if someone can pull off an sql injection using only valid wikipedia article titles, 
    // then they kind of deserve to
    string query1 = "INSERT INTO CACHE (ID,SRC,DST,";
    string query2 = "VALUES (" + to_string(size) + ",'" + string(src) + "','" + string(dst) + "',";
    for(unsigned int i=0; i<path.size(); i++){
        query1 += "DST,";
        query2 += to_string(path[i]);
    }
    query1 += "TIME) ";
    query2 += to_string(time) + ");";
    string query(query1 + query2);
    rc = sqlite3_exec(db, query.c_str(), select_callback, 0, &err);
    verify("Insert element");
}

Abs_path Cache::contains(const string &src, const string &dst){
    string cmd("SELECT * FROM CACHE where SRC='" + src + "' and DST='" + dst + "'");
    vector<Abs_path> results;// = new vector<Abs_path>;
    rc = sqlite3_exec(db, cmd.c_str(), select_callback, (void*)&results, &err);
    verify("Contains");
    //cout << results.size() << endl;
    if(results.size() == 0) return Abs_path();
    else return results[0];
}
//void update(const string &src, const string &dst);
