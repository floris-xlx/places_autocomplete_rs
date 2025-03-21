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

pub async fn read_all_lines(file_path: &str) -> Result<(), Box<dyn Error>> {
    let mut rdr: csv::Reader<File> = ReaderBuilder::new().from_path(file_path)?;

    let mut records = rdr.records();
    while let Some(record) = records.next() {
        match record {
            Ok(record) => {
                info!("Record: {:#?}", record);
            }
            Err(e) => {
                error!("Error reading record: {:#?}", e);
            }
        }
    }

    Ok(())
}

pub async fn count_lines_in_csv(file_path: &str) -> Result<usize, Box<dyn Error>> {
    let mut rdr: csv::Reader<File> = ReaderBuilder::new().from_path(file_path)?;
    let mut count = 0;

    let mut records = rdr.records();
    while let Some(record) = records.next() {
        match record {
            Ok(_) => {
                count += 1;
            }
            Err(e) => {
                error!("Error reading record: {:#?}", e);
            }
        }
    }

    Ok(count)
}
