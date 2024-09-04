use actix::{Addr, SyncArbiter};
use actix_web::{get, web::Data, App, HttpResponse, HttpServer, Responder};
use graph::Graph;

pub(crate) mod graph;
pub(crate) mod parser;
pub(crate) mod ql;
pub(crate) mod server;

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
                graph: SyncArbiter::start(16, || Graph::new()),
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
