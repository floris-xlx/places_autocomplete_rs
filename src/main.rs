#![allow(unused_must_use)]


use std::fs::File;
use std::io::BufReader;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

// crate imports
use places_autocomplete_rs::parser::csv::open_csv_and_extract_headers;
use places_autocomplete_rs::io::list::list_all_files_in_csv_data;   



#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_tracing();

    let file_names: Vec<String> = list_all_files_in_csv_data()?;
    info!("Files in csv_data directory: {:#?}", file_names);

    for file_name in file_names {
        let file_path: String = format!("./csv_data/{}", file_name);
        info!("Opening file: {}", file_path);


        match open_csv_and_extract_headers(file_path).await {
            Ok(headers) => {
                info!("Headers: {:#?}", headers);
            }
            Err(e) => {
                error!("Error: {:#?}", e);
            }
        }
    }

    info!("Done!");




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
