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
    postal_map: HashMap<char, HashMap<String, Vec<Row>>>, // Indexed by first character of postal code
    street_map: HashMap<String, Vec<Row>>,                // Street name lookups
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
                if let Some(first_char) = row.postal_code.chars().next() {
                    self.postal_map
                        .entry(first_char)
                        .or_default()
                        .entry(row.postal_code.clone())
                        .or_default()
                        .push(row.clone());
                }

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
        if let Some(first_char) = postal_code.chars().next() {
            self.postal_map
                .get(&first_char)
                .and_then(|map| map.get(postal_code))
        } else {
            None
        }
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
    let postal_code = postal_code
        .replace('_', "")
        .replace('-', "")
        .replace(' ', "");
    info!("Querying postal code: {}", postal_code);

    let data = LOCATION_DATA.read().expect("Failed to acquire read lock");

    let result: Vec<&Row> = if postal_code.len() == 4 && postal_code.chars().all(char::is_numeric) {
        // Partial match for postal codes with only 4 digits
        if let Some(first_char) = postal_code.chars().next() {
            data.postal_map
                .get(&first_char)
                .map(|map| {
                    map.iter()
                        .filter(|(key, _)| key.starts_with(&postal_code))
                        .flat_map(|(_, rows)| rows)
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    } else {
        // Exact match for full postal codes
        data.lookup_by_postal_code(&postal_code)
            .map(|rows| rows.iter().collect())
            .unwrap_or_default()
    };

    let response = if !result.is_empty() {
        let first_street = &result[0].street;
        if result.iter().all(|entry| entry.street == *first_street) {
            let house_numbers: Vec<&str> =
                result.iter().map(|row| row.house_number.as_str()).collect();
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
        "Query result for postal code {}: {} entries found in {} ms",
        postal_code,
        result.len(),
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
        let house_numbers: Vec<&str> = result.iter().map(|row| row.house_number.as_str()).collect();

        // Check if the query ends with a number
        if let Some(last_char) = query.chars().last() {
            if last_char.is_digit(10) {
                let query_number = query.split_whitespace().last().unwrap_or("");
                if house_numbers.contains(&query_number) {
                    let filtered_result: Vec<Row> = result
                        .into_iter()
                        .filter(|row| row.house_number == query_number)
                        .cloned()
                        .collect();
                    return json!({
                        "entries": filtered_result,
                        "house_numbers": vec![query_number],
                        "total_entries": filtered_result.len(),
                        "consistent_street": filtered_result.iter().all(|entry| entry.street == *first_street)
                    });
                }
            }
        }

        json!({
            "entries": result,
            "house_numbers": house_numbers,
            "total_entries": result.len(),
            "consistent_street": result.iter().all(|entry| entry.street == *first_street)
        })
    } else {
        json!({ "entries": [], "total_entries": 0, "house_numbers": [], "consistent_street": false })
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
    info!(
        "Querying closest locations to coordinates: ({}, {})",
        latitude, longitude
    );

    let data = LOCATION_DATA.read().expect("Failed to acquire read lock");

    let mut entries_with_distances: Vec<(&Row, f64)> = Vec::new();

    for rows in data
        .postal_map
        .values()
        .flat_map(|map| map.values())
        .chain(data.street_map.values())
    {
        for row in rows {
            let row_latitude: f64 = row.latitude;
            let row_longitude: f64 = row.longitude;

            let distance = haversine_distance(latitude, longitude, row_latitude, row_longitude);
            entries_with_distances.push((row, distance));
        }
    }

    // Sort by distance
    entries_with_distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    // Collect unique streets
    let mut unique_streets = Vec::new();
    let mut seen_streets = std::collections::HashSet::new();

    for (entry, distance) in entries_with_distances {
        if seen_streets.insert(&entry.street) {
            unique_streets.push((entry, distance));
        }
        if unique_streets.len() == 100 {
            break;
        }
    }

    let response = json!({
        "entries": unique_streets.iter().map(|(entry, distance)| json!({
            "entry": entry,
            "distance": distance
        })).collect::<Vec<_>>(),
        "total_entries": unique_streets.len()
    });

    info!(
        "Query result for coordinates ({}, {}): {} unique streets found in {} ms",
        latitude,
        longitude,
        unique_streets.len(),
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
