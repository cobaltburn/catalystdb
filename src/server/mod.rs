use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::AppState;

#[derive(Debug, Deserialize)]
struct Query {
    query: String,
}

#[post("/ql")]
async fn query(
    state: web::Data<AppState>,
    web::Json(Query { query }): web::Json<Query>,
) -> impl Responder {
    let _graph = state.graph.clone();
    HttpResponse::Ok().body("hello world")
}
