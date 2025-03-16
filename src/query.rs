use csv::ReaderBuilder;
use serde::Deserialize;
use serde::Serialize;
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
}

#[derive(Debug, Deserialize, Clone)]
pub struct LocationData {
    postal_map: HashMap<String, Vec<Row>>, // For exact postal code lookups
    streets: Vec<Row>,                     // For partial street name matching
}

impl LocationData {
    pub fn new() -> Self {
        info!("Creating new LocationData instance");
        Self {
            postal_map: HashMap::new(),
            streets: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, path: &str) {
        let start_time = Instant::now();
        info!("Loading data from CSV file: {}", path);
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)
            .expect("Failed to open CSV file");
        let headers_len = rdr.headers().expect("Failed to read headers").len();
        for result in rdr.deserialize::<Row>() {
            match result {
                Ok(row) => {
                    info!("Deserialized row: {:?}", row);
                    if headers_len == 10 {
                        self.postal_map
                            .entry(row.postal_code.clone())
                            .or_default()
                            .push(row.clone());
                        self.streets.push(row);
                    } else {
                        eprintln!("Warning: CSV header length does not match the expected number of fields, skipping row...");
                    }
                }
                Err(err) => {
                    eprintln!(
                        "Warning: Failed to deserialize a row due to error: {}, skipping...",
                        err
                    );
                }
            }
        }
        let duration = start_time.elapsed();
        info!(
            "Finished loading data from CSV file: {} in {} ms",
            path,
            duration.as_millis()
        );
    }

    pub fn load_all(&mut self, folder: &str) {
        let start_time = Instant::now();
        info!("Loading all CSV files from folder: {}", folder);
        for entry in fs::read_dir(folder).expect("Failed to read directory") {
            let path = entry.expect("Failed to read directory entry").path();
            if path.extension().unwrap_or_default() == "csv" {
                self.load_from_csv(path.to_str().expect("Failed to convert path to string"));
            }
        }
        let duration = start_time.elapsed();
        info!(
            "Finished loading all CSV files from folder: {} in {} ms",
            folder,
            duration.as_millis()
        );
    }

    pub fn lookup_by_postal_code(&self, postal_code: &str) -> Vec<&Row> {
        let start_time = Instant::now();
        let postal_code = postal_code.to_lowercase();
        info!("Looking up postal code: {}", postal_code);
        let mut results: Vec<&Row> = self
            .postal_map
            .iter()
            .filter(|(key, _)| key.to_lowercase().contains(&postal_code))
            .flat_map(|(_, rows)| rows)
            .collect();

        let duration = start_time.elapsed();
        info!(
            "Lookup for postal code {} completed in {} ms",
            postal_code,
            duration.as_millis()
        );
        results
    }

    pub fn search_by_street(&self, query: &str) -> Vec<&Row> {
        let start_time = Instant::now();
        let query = query.to_lowercase();
        info!("Searching for street containing: {}", query);
        let mut results: Vec<&Row> = self
            .streets
            .iter()
            .filter(|row| row.street.to_lowercase().contains(&query))
            .collect();

        let duration = start_time.elapsed();
        info!(
            "Search for street '{}' completed in {} ms",
            query,
            duration.as_millis()
        );
        results
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
    let duration = start_time.elapsed();
    info!(
        "Finished initializing location data in {} ms",
        duration.as_millis()
    );
}

pub fn query_postal_code(postal_code: &str) -> Value {
    let start_time = Instant::now();
    info!("Querying postal code: {}", postal_code);
    let data = LOCATION_DATA.read().expect("Failed to acquire read lock");
    let result: Vec<Value> = data
        .lookup_by_postal_code(postal_code)
        .into_iter()
        .map(|row| json!(row))
        .collect();
    let total_entries = result.len();
    let duration = start_time.elapsed();
    info!(
        "Query result for postal code {}: {} entries found in {} ms",
        postal_code,
        total_entries,
        duration.as_millis()
    );
    json!({
        "entries": result,
        "total_entries": total_entries
    })
}

pub fn query_street(query: &str) -> Value {
    let start_time = Instant::now();
    info!("Querying street with search term: {}", query);
    let data = LOCATION_DATA.read().expect("Failed to acquire read lock");
    let result: Vec<Value> = data
        .search_by_street(query)
        .into_iter()
        .cloned()
        .map(|row| json!(row))
        .collect();
    let total_entries = result.len();
    let duration = start_time.elapsed();
    info!(
        "Query result for street search '{}': {} entries found in {} ms",
        query,
        total_entries,
        duration.as_millis()
    );
    json!({
        "entries": result,
        "total_entries": total_entries
    })
}
