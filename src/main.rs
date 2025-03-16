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
use std::collections::HashMap;
use std::env::var;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;


use places_autocomplete_rs::SharedCache;

use places_autocomplete_rs::api::actix_client::ping;
use places_autocomplete_rs::query::{initialize_location_data, query_postal_code, query_street};

#[get("/search")]
async fn search(
    web::Query(info): web::Query<HashMap<String, String>>,
    data: Data<SharedCache>,
) -> impl Responder {
    let mut response = json!({});
    let mut found = false;
    let limit: usize = info.get("limit").and_then(|l| l.parse().ok()).unwrap_or(10);

    if let Some(postal_code) = info.get("postal_code") {
        let mut location_data = query_postal_code(postal_code);
        if let Some(house_number) = info.get("house_number") {
            if let Some(entry) = location_data.get_mut("entry") {
                if entry.get("house_number").map_or(false, |hn| hn != house_number) {
                    location_data = json!({});
                }
            }
        }
        if let Some(entries) = location_data.get_mut("entries") {
            if let Some(entries_array) = entries.as_array_mut() {
                entries_array.truncate(limit);
            }
            if !entries.as_array().unwrap_or(&vec![]).is_empty() {
                response["postal_code"] = location_data;
                found = true;
            }
        } else if location_data.get("entry").is_some() {
            response["postal_code"] = location_data;
            found = true;
        }
    }

    if let Some(street) = info.get("street") {
        let mut location_data = query_street(street);
        if let Some(house_number) = info.get("house_number") {
            if let Some(entries) = location_data.get_mut("entries") {
                if let Some(entries_array) = entries.as_array_mut() {
                    entries_array.retain(|entry| {
                        entry.get("house_number").map_or(false, |hn| hn == house_number)
                    });
                }
            }
        }
        if let Some(entries) = location_data.get_mut("entries") {
            if let Some(entries_array) = entries.as_array_mut() {
                entries_array.truncate(limit);
            }
            if !entries.as_array().unwrap_or(&vec![]).is_empty() {
                response["street"] = location_data;
                found = true;
            }
        }
    }

    if found {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::NotFound().body("No matching data found")
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    println!("Hello, world!");
    initialize_location_data("./data_split");

    // Initialize tracing
    // floris; fixme
    init_tracing();

    dotenv::dotenv().ok();

    let port: u16 = var("XLX_PLACES_AUTOCOMPLETE_API_PORT")
        .unwrap_or("4444".to_string())
        .parse()
        .unwrap_or(4444);

    let cache: SharedCache = Arc::new(Mutex::new(
        Cache::builder()
            .time_to_live(Duration::from_secs(60 * 60 * 5000))
            .build(),
    ));

    // http builder
    HttpServer::new(move || {
        let cors: Cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .wrap_fn(|req, srv| {
                let fut = srv.call(req);
                async move {
                    let mut res: ServiceResponse<EitherBody<BoxBody>> = fut.await?;
                    res.headers_mut()
                        .insert(header::SERVER, "XYLEX/0".parse().unwrap());
                    Ok(res)
                }
            })
            // cache injecting middleware
            .app_data(Data::new(cache.clone()))
            // endpoints // docs
            .service(ping)
            .service(search)
          
    })
    .workers(4)
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

/// ## Initialize Tracing
///
/// This function sets up the tracing subscriber for logging and monitoring.
///
/// ### Example
///
/// ```
/// init_tracing();
/// ```
fn init_tracing() {
    let filter: EnvFilter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt().with_env_filter(filter).init()
}
