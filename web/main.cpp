//this file:
#include<iostream>
//crow
#include "../crow/include/crow.h"
#include "../crow/include/mustache.h"
#include "../crow/include/json.h"
//table headers
#include "../WikiLinkr/table.h"

using namespace std;


int main(){
    crow::SimpleApp app;
    crow::mustache::set_base(".");

    CROW_ROUTE(app, "/")
        ([]() {
         //TODO: create index.html
         return "Hell World\n";
         });
   
    //CROW_ROUTE(app, "/bfs/<int>/<int>")
        //([](const crow::request& req,crow::response& res, std::string src, std::string dst){
         //crow::mustache::context ctx;
         //ctx["src"] = src;
         //ctx["dst"] = dst;
         //return crow::mustache::load("templates/bfs.html").render(ctx);

         //return "TEST";
         //});
    CROW_ROUTE(app, "/bfs/<string>/<string>")
        ([](const crow::request& req, crow::response& res, string a, string b){
         //std::ostringstream os;
         //os << a+b;
         crow::mustache::context ctx;
         ctx["src"] = a;
         ctx["dst"] = b;
         res.write(crow::mustache::load("templates/bfs.html").render(ctx));
         res.end();
         });

    CROW_ROUTE(app, "/bfs")
        ([](const crow::request& req){
         std::string src, dst;
         crow::mustache::context ctx;
         if(req.url_params.get("src") != nullptr) src = req.url_params.get("src");
         else src = "SRC NOT FOUND";
         dst = (req.url_params.get("dst") == nullptr ? "DST NOT FOUND" : req.url_params.get("dst"));
         ctx["src"] = src;
         ctx["dst"] = dst;
         return crow::mustache::load("templates/bfs.html").render(ctx);
         });

    app.port(8000).run();

    
    return 0;
}
