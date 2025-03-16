#![allow(unused_must_use)]

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use std::{io::Result, time::Duration};

use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::body::{BoxBody, EitherBody};
use actix_web::dev::{Service, ServiceResponse};
use actix_web::http::header;
use actix_web::web::Data;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use moka::future::Cache;
use serde_json::{json, Value};
use std::env::var;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[get("/")]
pub async fn ping() -> impl Responder {
    let start_time: Instant = Instant::now();
    info!("endpoint received request");
    let latency: u128 = start_time.elapsed().as_millis();
    let status_info: Value = json!({
        "status": "healthy",
        "message": "api.places.suitsbooks.nl is healthy",
        "version": "0.0.1",
        "latency": latency
    });
    HttpResponse::Ok().json(status_info)
}
