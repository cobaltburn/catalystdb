use actix::{Actor, Addr};
use actix_web::{get, web::Data, App, HttpResponse, HttpServer, Responder};
use dbs::graph::Graph;

pub mod dbs;
pub mod err;
pub mod func;
pub mod parser;
pub mod ql;
pub mod resp;
pub mod server;

struct AppState {
    graph: Addr<Graph>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let address = "0.0.0.0";
    let port = 8080;

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                graph: Graph::new().start(),
            }))
            .service(server::query)
    })
    .bind((address, port))?
    .run()
    .await
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("hello world")
}
