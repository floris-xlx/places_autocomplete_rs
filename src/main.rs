#![allow(unused_must_use)]

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

// crate imports
use places_autocomplete_rs::generator::process_csv_files;
use places_autocomplete_rs::io::create::create_file_if_not_exists;
use places_autocomplete_rs::io::list::list_all_files_in_csv_data;
use places_autocomplete_rs::parser::csv::open_csv_and_extract_headers;
use places_autocomplete_rs::parser::csv::{count_lines_in_csv, read_all_lines};
use places_autocomplete_rs::parser::enumurate_house_numbers::enumerate_house_numbers;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_tracing();

    let (meow, meow2, meow3, meow4, meow5) = tokio::join!(
        process_csv_files("./csv_data/postcodes_20190613.csv"),
        process_csv_files("./csv_data/postcodes_20190622_1.csv"),
        process_csv_files("./csv_data/postcodes_20190622_2.csv"),
        process_csv_files("./csv_data/postcodes_20190622_3.csv"),
        process_csv_files("./csv_data/postcodes_20190622_4.csv")
    );
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
