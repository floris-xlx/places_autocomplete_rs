use actix_web::{get, HttpResponse, Responder};
use serde_json::{json, Value};
use std::time::Instant;
use tracing::info;

#[get("/")]
pub async fn ping() -> impl Responder {
    let start_time = Instant::now();
    info!("endpoint received request");
    let latency = start_time.elapsed().as_millis();
    let status_info: Value = json!({
        "status": "healthy",
        "message": "api.places.suitsbooks.nl is healthy",
        "version": "0.0.1",
        "latency": latency
    });
    HttpResponse::Ok().json(status_info)
}
