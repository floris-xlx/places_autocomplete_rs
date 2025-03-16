use csv::{ReaderBuilder, StringRecord};
use futures::stream::StreamExt;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use tracing::{error, info, warn};

pub async fn open_csv_and_extract_headers<P: AsRef<Path>>(
    file_path: P,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut rdr: csv::Reader<File> = ReaderBuilder::new().from_path(file_path)?;

    let headers: Vec<String> = Vec::new();
    info!("Extracting headers from CSV file");
    info!("Headers: {:#?}", rdr.headers());

    Ok(headers)
}
