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
use places_autocomplete_rs::query::{
    initialize_location_data, query_by_coordinates, query_postal_code, query_street,
};

#[get("/search_by_coordinates")]
async fn search_by_coordinates(
    web::Query(info): web::Query<HashMap<String, String>>,
) -> impl Responder {
    info!(
        "Received request for search_by_coordinates with query: {:?}",
        info
    );

    let response = if let (Some(lat), Some(lon)) = (info.get("latitude"), info.get("longitude")) {
        info!(
            "Latitude and longitude parameters found: lat={}, lon={}",
            lat, lon
        );
        if let (Ok(latitude), Ok(longitude)) = (lat.parse::<f64>(), lon.parse::<f64>()) {
            info!(
                "Parsed latitude and longitude successfully: latitude={}, longitude={}",
                latitude, longitude
            );
            query_by_coordinates(latitude, longitude)
        } else {
            warn!(
                "Invalid latitude or longitude format: lat={}, lon={}",
                lat, lon
            );
            json!({ "error": "Invalid latitude or longitude format" })
        }
    } else {
        warn!(
            "Missing latitude or longitude parameters in query: {:?}",
            info
        );
        json!({ "error": "Missing latitude or longitude parameters" })
    };

    info!("Response for search_by_coordinates: {:?}", response);
    HttpResponse::Ok().json(response)
}

#[get("/search")]
async fn search(
    web::Query(info): web::Query<HashMap<String, String>>,
    data: Data<SharedCache>,
) -> impl Responder {
    info!("Received request for search with query: {:?}", info);

    let mut response = json!({});
    let mut found = false;
    let limit: usize = info.get("limit").and_then(|l| l.parse().ok()).unwrap_or(10);
    let unique_street_only: bool = info
        .get("unique_street_only")
        .map_or(false, |v| v.parse().unwrap_or(false));
    info!("Limit for search results set to: {}", limit);
    info!("Unique street only flag set to: {}", unique_street_only);

    if let Some(postal_code) = info.get("postal_code") {
        info!("Postal code parameter found: {}", postal_code);
        let mut location_data = query_postal_code(postal_code);
        if let Some(house_number) = info.get("house_number") {
            info!("House number parameter found: {}", house_number);
            if let Some(entry) = location_data.get_mut("entry") {
                if entry
                    .get("house_number")
                    .map_or(false, |hn| hn != house_number)
                {
                    info!("House number does not match entry house number, clearing location data");
                    location_data = json!({});
                }
            }
        }
        if let Some(entries) = location_data.get_mut("entries") {
            if let Some(entries_array) = entries.as_array_mut() {
                if unique_street_only {
                    let mut seen_streets = std::collections::HashSet::new();
                    entries_array.retain(|entry| {
                        entry
                            .get("street")
                            .map_or(false, |street| seen_streets.insert(street.clone()))
                    });
                    info!("Filtered entries to unique streets");
                }
                entries_array.truncate(limit);
                info!("Truncated entries to limit: {}", limit);
            }
            if !entries.as_array().unwrap_or(&vec![]).is_empty() {
                response["postal_code"] = location_data;
                found = true;
                info!("Postal code entries found and added to response");
            }
        } else if location_data.get("entry").is_some() {
            response["postal_code"] = location_data;
            found = true;
            info!("Single postal code entry found and added to response");
        }
    }

    if let Some(street) = info.get("street") {
        info!("Street parameter found: {}", street);
        let mut location_data = query_street(street);
        if let Some(house_number) = info.get("house_number") {
            info!("House number parameter found: {}", house_number);
            if let Some(entries) = location_data.get_mut("entries") {
                if let Some(entries_array) = entries.as_array_mut() {
                    entries_array.retain(|entry| {
                        entry
                            .get("house_number")
                            .map_or(false, |hn| hn == house_number)
                    });
                    info!("Filtered entries by house number: {}", house_number);
                }
            }
        }
        if let Some(entries) = location_data.get_mut("entries") {
            if let Some(entries_array) = entries.as_array_mut() {
                if unique_street_only {
                    let mut seen_streets = std::collections::HashSet::new();
                    entries_array.retain(|entry| {
                        entry
                            .get("street")
                            .map_or(false, |street| seen_streets.insert(street.clone()))
                    });
                    info!("Filtered entries to unique streets");
                }
                entries_array.truncate(limit);
                info!("Truncated entries to limit: {}", limit);
            }
            if !entries.as_array().unwrap_or(&vec![]).is_empty() {
                response["street"] = location_data;
                found = true;
                info!("Street entries found and added to response");
            }
        }
    }

    if found {
        info!("Search successful, returning response");
        HttpResponse::Ok().json(response)
    } else {
        warn!("No matching data found for search query: {:?}", info);
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
            .service(search_by_coordinates)
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
