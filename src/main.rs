#![allow(unused_must_use)]

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

// crate imports
use places_autocomplete_rs::parser::csv::open_csv_and_extract_headers;
use places_autocomplete_rs::io::list::list_all_files_in_csv_data;
use places_autocomplete_rs::parser::enumurate_house_numbers::enumerate_house_numbers;
use places_autocomplete_rs::parser::csv::{read_all_lines, count_lines_in_csv};
use places_autocomplete_rs::io::create::create_file_if_not_exists;
use places_autocomplete_rs::generator::process_csv_files;


#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_tracing();




    let meow: Result<(), Box<dyn Error + Send + Sync>> = process_csv_files("./csv_data/postcodes_20190613.csv").await;
    let meow2: Result<(), Box<dyn Error + Send + Sync>> = process_csv_files("./csv_data/postcodes_20190622_1.csv").await;
    let meow3: Result<(), Box<dyn Error + Send + Sync>> = process_csv_files("./csv_data/postcodes_20190622_2.csv").await;
    let meow4: Result<(), Box<dyn Error + Send + Sync>> = process_csv_files("./csv_data/postcodes_20190622_3.csv").await;
    let meow5: Result<(), Box<dyn Error + Send + Sync>> = process_csv_files("./csv_data/postcodes_20190622_4.csv").await;
    println!("{:#?}", meow);
    println!("{:#?}", meow2);
    println!("{:#?}", meow3);
    println!("{:#?}", meow4);
    println!("{:#?}", meow5);
    



    Ok(())
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
