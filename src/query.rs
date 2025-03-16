use csv::ReaderBuilder;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::sync::RwLock;
use std::time::Instant;
use tracing::info;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Row {
    pub postal_code: String,
    pub street: String,
    pub house_number: String,
    pub city: String,
    pub area: String,
    pub neighborhood: String,
    pub municipality: String,
    pub province: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug)]
pub struct LocationData {
    postal_map: HashMap<String, Vec<Row>>, // Exact postal code lookups
    street_map: HashMap<String, Vec<Row>>, // Street name lookups
}

impl LocationData {
    pub fn new() -> Self {
        info!("Creating new LocationData instance");
        Self {
            postal_map: HashMap::new(),
            street_map: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, path: &str) {
        let start_time = Instant::now();
        info!("Loading data from CSV file: {}", path);

        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)
            .expect("Failed to open CSV file");

        for result in rdr.deserialize::<Row>() {
            if let Ok(row) = result {
                self.postal_map
                    .entry(row.postal_code.clone())
                    .or_default()
                    .push(row.clone());

                self.street_map
                    .entry(row.street.to_lowercase())
                    .or_default()
                    .push(row);
            }
        }

        info!(
            "Finished loading data from {} in {} ms",
            path,
            start_time.elapsed().as_millis()
        );
    }

    pub fn load_all(&mut self, folder: &str) {
        let start_time = Instant::now();
        info!("Loading all CSV files from folder: {}", folder);

        for entry in fs::read_dir(folder).expect("Failed to read directory") {
            let path = entry.expect("Failed to read directory entry").path();
            if path.extension().unwrap_or_default() == "csv" {
                self.load_from_csv(path.to_str().unwrap());
            }
        }

        info!(
            "Finished loading all CSV files in {} ms",
            start_time.elapsed().as_millis()
        );
    }

    pub fn lookup_by_postal_code(&self, postal_code: &str) -> Option<&Vec<Row>> {
        self.postal_map.get(postal_code)
    }

    pub fn search_by_street(&self, query: &str) -> Vec<&Row> {
        let query = query.to_lowercase();
        self.street_map
            .iter()
            .filter(|(street, _)| street.contains(&query))
            .flat_map(|(_, rows)| rows)
            .collect()
    }
}

lazy_static::lazy_static! {
    pub static ref LOCATION_DATA: RwLock<LocationData> = RwLock::new(LocationData::new());
}

pub fn initialize_location_data(folder: &str) {
    let start_time = Instant::now();
    info!("Initializing location data from folder: {}", folder);

    let mut data = LOCATION_DATA.write().expect("Failed to acquire write lock");
    data.load_all(folder);

    info!(
        "Finished initializing location data in {} ms",
        start_time.elapsed().as_millis()
    );
}

pub fn query_postal_code(postal_code: &str) -> Value {
    let start_time = Instant::now();
    info!("Querying postal code: {}", postal_code);

    let data = LOCATION_DATA.read().expect("Failed to acquire read lock");

    let result = data.lookup_by_postal_code(postal_code);
    let response = match result {
        Some(rows) if !rows.is_empty() => {
            let first_street = &rows[0].street;
            if rows.iter().all(|entry| entry.street == *first_street) {
                let house_numbers: Vec<&str> = rows.iter().map(|row| row.house_number.as_str()).collect();
                json!({
                    "entry": rows[0],
                    "house_numbers": house_numbers,
                    "total_entries": rows.len()
                })
            } else {
                json!({
                    "entries": rows,
                    "total_entries": rows.len()
                })
            }
        }
        _ => json!({ "entries": [], "total_entries": 0 }),
    };

    info!(
        "Query result for postal code {}: {} entries found in {} ms",
        postal_code,
        result.map(|r| r.len()).unwrap_or(0),
        start_time.elapsed().as_millis()
    );

    response
}

pub fn query_street(query: &str) -> Value {
    let start_time = Instant::now();
    info!("Querying street with search term: {}", query);

    let data = LOCATION_DATA.read().expect("Failed to acquire read lock");
    let result = data.search_by_street(query);

    let response = if !result.is_empty() {
        let first_street = &result[0].street;
        if result.iter().all(|entry| entry.street == *first_street) {
            let house_numbers: Vec<&str> = result.iter().map(|row| row.house_number.as_str()).collect();
            json!({
                "entry": result[0],
                "house_numbers": house_numbers,
                "total_entries": result.len()
            })
        } else {
            json!({
                "entries": result,
                "total_entries": result.len()
            })
        }
    } else {
        json!({ "entries": [], "total_entries": 0 })
    };

    info!(
        "Query result for street search '{}': {} entries found in {} ms",
        query,
        result.len(),
        start_time.elapsed().as_millis()
    );

    response
}


pub fn query_by_coordinates(latitude: f64, longitude: f64) -> Value {
    let start_time = Instant::now();
    info!("Querying closest location to coordinates: ({}, {})", latitude, longitude);

    let data = LOCATION_DATA.read().expect("Failed to acquire read lock");

    let mut closest_entry: Option<&Row> = None;
    let mut min_distance = f64::MAX;

    for rows in data.postal_map.values().chain(data.street_map.values()) {
        for row in rows {
            let row_latitude: f64 = row.latitude;
            let row_longitude: f64 = row.longitude;

            let distance = haversine_distance(latitude, longitude, row_latitude, row_longitude);
            if distance < min_distance {
                min_distance = distance;
                closest_entry = Some(row);
            }
        }
    }

    let response = match closest_entry {
        Some(entry) => json!({
            "entry": entry,
            "distance": min_distance
        }),
        None => json!({ "entry": null, "distance": null }),
    };

    info!(
        "Query result for coordinates ({}, {}): closest entry found in {} ms",
        latitude,
        longitude,
        start_time.elapsed().as_millis()
    );

    response
}

fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0; // Radius of the Earth in kilometers
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();

    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    r * c
}
