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

using namespace std;
void populate_ctx(crow::mustache::context&, Table&, Cache&, std::string, std::string);
std::string format_link(const Abs_path &ap);

int main(int argc, char* argv[]){
    //construct table:
    if(argc != 2){
        std::cerr << "usage: WikiLinks.out path_to_parsed_file" << std::endl;
        exit(1);
    }
    //construct data objects
    Table t(argv[1]);
    Cache cache("cache1.db");

    //construct web fw
    crow::SimpleApp app;
    crow::mustache::set_base("./templates/");

    //define locations a la flask
    CROW_ROUTE(app, "/")
        ([&t, &cache](const crow::request& req){
        std::string src, dst;
        crow::mustache::context ctx;
        src = (req.url_params.get("src") == nullptr ? "" : req.url_params.get("src"));
        dst = (req.url_params.get("dst") == nullptr ? "" : req.url_params.get("dst"));
        populate_ctx(ctx, t, cache, src, dst);
        return crow::mustache::load("bfs.html").render(ctx);
        });

    //search via url 
    CROW_ROUTE(app, "/bfs/<string>/<string>")
        ([&t, &cache](const crow::request& req, crow::response& res, string src, string dst){
         crow::mustache::context ctx;
         populate_ctx(ctx, t, cache, src, dst);
         res.write(crow::mustache::load("bfs.html").render(ctx));
         res.end();
         });

    app.port(8080).multithreaded().run();

    
    return 0;
}

void populate_ctx(crow::mustache::context &ctx, Table &t, Cache &cache, 
        std::string src, std::string dst){
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
            ctx["path"] = t.htmlPath(ap->path);
            delete ap;
        } else {
            //need to generate path
            Path path = t.search(src_, dst_);
            cache.insert(src, dst, path, -1);
            ctx["path"] = t.htmlPath(path);
        }
    }
    vector<Abs_path> aps = cache.retrieve(20, Cache::sort_by::popular);
    std::string history;
    for(unsigned int i=0; i<aps.size(); i++){
        history += format_link(aps[i]);
    }
    ctx["cache"] = history;
}

std::string format_link(const Abs_path &ap){
    //format to be selectable option
    std::string result = "\t\t<form action=\"/\" >";
    result += "<input name=\"src\" value=\"" + ap.src + "\" readonly />";
    result += "&rarr; " + to_string(ap.path.size()) + " &rarr; ";   //subtract? 2?
    result += "<input name=\"dst\" value=\"" + ap.dst + "\" readonly /> &nbsp;";
    result += "<input type=\"submit\" name=\"submit\" value=\"Submit\"/>";
    result += "</form>\n";
    return result;
}
