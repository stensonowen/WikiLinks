//g++ main.cpp -std=c++11 -lboost_system -pthread -lsqlite3
//sudo apt-get install libsqlite3-dev
//#include<iostream>
//crow
#include "../include/crow.h"
#include "../include/mustache.h"
//#include "../crow/include/json.h"
//table headers
#include<iostream>
#include "../src/table.h"
#include "cache.h"
#include "queue.h"

using namespace std;
void populate_ctx(crow::mustache::context&, Table&, Cache&, Queue&, std::string, std::string);
std::string format_link(const Abs_path &ap);
void thread_test(){ for(long i=0; i<10000000000; i++) {} }

int main(int argc, char* argv[]){
    //construct table:
    if(argc != 2){
        std::cerr << "usage: WikiLinks.out path_to_parsed_file" << std::endl;
        exit(1);
    }
    //construct data objects
    Table t(argv[1]);
    Cache cache("cache1.db");
    Queue queue(&t);

    //cout << "Starting test" << endl;
    //thread_test();
    //cout << "Done with test" << endl;
    
    //construct web fw
    crow::SimpleApp app;
    crow::mustache::set_base("./templates/");

    //define locations a la flask
    CROW_ROUTE(app, "/")
        ([&t, &cache, &queue](const crow::request& req){
        std::string src, dst;
        crow::mustache::context ctx;
        src = (req.url_params.get("src") == nullptr ? "" : req.url_params.get("src"));
        dst = (req.url_params.get("dst") == nullptr ? "" : req.url_params.get("dst"));
        populate_ctx(ctx, t, cache, queue, src, dst);
        return crow::mustache::load("bfs.html").render(ctx);
        });

    //search via url 
    CROW_ROUTE(app, "/bfs/<string>/<string>")
        ([&t, &cache, &queue](const crow::request& req, crow::response& res, string src, string dst){
         crow::mustache::context ctx;
         populate_ctx(ctx, t, cache, queue, src, dst);
         res.write(crow::mustache::load("bfs.html").render(ctx));
         res.end();
         });

    app.port(80).multithreaded().run();

    
    return 0;
}

void populate_ctx(crow::mustache::context &ctx, Table &t, Cache &cache, Queue &queue,
        std::string src, std::string dst){
    //replace underscores with spaces
    //should be sufficient to convert article urls to titles (right?)
    //TODO: url decode (+encode?)
    std::replace(src.begin(), src.end(), '_', ' ');
    std::replace(dst.begin(), dst.end(), '_', ' ');
    ctx["src"] = src;
    ctx["dst"] = dst;
    unsigned int src_, dst_;
    std::string result = t.contains(src, dst, src_, dst_);
    if(!result.empty()) ctx["path"] = result;
    else {
        //valid input
        Abs_path *ap = cache.contains(src, dst);
        if(ap){
            //cout << "2: src = " << ap->src << ";  dst = " << ap->dst << endl;
            cache.update(*ap);
            ctx["path"] = t.htmlPath(ap->path, ap->code);
            delete ap;
        } else {
            //need to generate path
            //Path path = t.search(src_, dst_);
            
            //cout << "Starting test" << endl;
            //thread_test();
            //cout << "Done with test" << endl;
            
            std::pair<Path,int> path;
            std::thread thread([&] {path = queue.enqueue(src_, dst_); });
            thread.join();
            //Path path = queue.enqueue(src_, dst_);
            cache.insert(src, dst, path.first, path.second);
            ctx["path"] = t.htmlPath(path.first, path.second);
        }
    }
    //vector<Abs_path> aps = cache.retrieve(20, Cache::sort_by::popular);
    vector<Abs_path> aps = cache.retrieve(20, Cache::sort_by::length);
    std::string history;
    //for(unsigned int i=0; i<aps.size(); i++){
    //for(auto itr = aps.rbegin(); itr != aps.rend(); itr++){
    for(auto itr = aps.begin(); itr != aps.end(); itr++){
        //should go backwards I guess
        history += format_link(*itr);
    }
    ctx["cache"] = history;
}

std::string format_link(const Abs_path &ap){
    //format to be selectable option
    std::string result = "\t\t<form action=\"/\" >";
    result += "<input name=\"src\" value=\"" + ap.src + "\" readonly />";
    //result += "&rarr; " + to_string(ap.path.size()) + " &rarr; ";   //subtract? 2?
    result += "&rarr; ";
    if(ap.path.size() == 0){
        if(ap.src == ap.dst) result += "0";
        else result += "?";
    } else result += to_string(ap.path.size()-1);
    result += " &rarr; ";
    result += "<input name=\"dst\" value=\"" + ap.dst + "\" readonly /> &nbsp;";
    result += "<input type=\"submit\" name=\"submit\" value=\"Submit\"/>";
    result += "</form>\n";
    return result;
}

