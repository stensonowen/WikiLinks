//this file:
#include<iostream>
//crow
#include "../include/crow.h"
#include "../include/mustache.h"
//#include "../crow/include/json.h"
//table headers
#include "../src/table.h"

//g++ main.cpp -std=c++11 -lboost_system -pthread

using namespace std;

int main(int argc, char* argv[]){
    //construct table:
    if(argc != 2){
        std::cerr << "usage: WikiLinks.out path_to_parsed_file" << std::endl;
        exit(1);
    }
    Table t(argv[1]);

    //construct web fw
    crow::SimpleApp app;
    crow::mustache::set_base("./templates/");

    //define locations a la flask
    CROW_ROUTE(app, "/")
        ([]() {
         //TODO: create index.html
         return "Hell World\n";
         });
   
    CROW_ROUTE(app, "/bfs/<string>/<string>")
        ([&t](const crow::request& req, crow::response& res, string src, string dst){
         crow::mustache::context ctx;
         ctx["src"] = src;
         ctx["dst"] = dst;
         ctx["path"] = t.htmlPath(src, dst);
         res.write(crow::mustache::load("bfs.html").render(ctx));
         res.end();
         });

    CROW_ROUTE(app, "/bfs")
        ([&t](const crow::request& req){
         std::string src, dst;
         crow::mustache::context ctx;
         src = (req.url_params.get("src") == nullptr ? "" : req.url_params.get("src"));
         dst = (req.url_params.get("dst") == nullptr ? "" : req.url_params.get("dst"));
         ctx["src"] = src;
         ctx["dst"] = dst;
         ctx["path"] = t.htmlPath(src, dst);
         return crow::mustache::load("bfs.html").render(ctx);
         });

    app.port(80).multithreaded().run();

    
    return 0;
}



