use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom, Write};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

// crate imports
use crate::io::create::create_file_if_not_exists;

use crate::parser::csv::open_csv_and_extract_headers;
use crate::parser::csv::{count_lines_in_csv, read_all_lines};
use crate::parser::enumurate_house_numbers::enumerate_house_numbers;

pub async fn process_csv_files(
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let headers: Vec<&str> = vec![
        "postal_code",
        "street",
        "house_numbers",
        "city",
        "area",
        "neighborhood",
        "municipality",
        "province",
    ];

    // Open CSV and extract headers
    if let Err(e) = open_csv_and_extract_headers(file_path).await {
        error!("Error extracting headers: {:#?}", e);
    }
    info!("Headers extracted successfully");

    // Read all lines and process them
    if let Err(e) = read_all_lines(file_path).await {
        error!("Error reading lines: {:#?}", e);
        if let Err(e) = read_all_lines(file_path).await {
            error!("Error reading lines: {:#?}", e);

            // Append the error to failed_lines.txt
            let mut failed_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("failed_lines.txt")?;

            writeln!(
                failed_file,
                "Error reading lines from {}: {:#?}",
                file_path, e
            )?;
        }
    }
    info!("Lines read successfully");

    fn list_files_in_directory(directory: &str) -> std::io::Result<Vec<String>> {
        let mut file_list = Vec::new();
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                    file_list.push(file_name.to_string());
                }
            }
        }
        Ok(file_list)
    }

    // Initialize the unique line count
    let mut unique_line_count = 0;
    let mut file_index = {
        let files = list_files_in_directory("./data")?;
        let mut max_index = 0;

        for file in files {
            if let Some(index_str) = file.strip_prefix("data_nl_").and_then(|s| s.strip_suffix(".csv")) {
                if let Ok(index) = index_str.parse::<usize>() {
                    if index > max_index {
                        max_index = index;
                    }
                }
            }
        }

        if max_index == 0 {
            1
        } else {
            max_index + 1
        }
    };

    // Open the CSV file for reading
    let mut rdr = csv::Reader::from_path(file_path)?;
    let mut output_file_path = format!("./data/data_nl_{}.csv", file_index);
    create_file_if_not_exists(&output_file_path)?;
    let mut writer = csv::Writer::from_path(&output_file_path)?;

    // Write headers to the output file
    writer.write_record(&headers)?;

    // Initialize a set to track unique lines
    let mut unique_lines = HashSet::new();

    for result in rdr.records() {
        let record = result?;
        let line = record.iter().collect::<Vec<&str>>().join(",");
        let enumerated_lines = enumerate_house_numbers(&line);

        for enumerated_line in enumerated_lines {
            if unique_lines.insert(enumerated_line.clone()) {
                writer.write_record(enumerated_line.split(','))?;
                unique_line_count += 1;

                // Check if the file has reached the maximum line count
                if unique_line_count >= 1_000_000 {
                    writer.flush()?;
                    info!("Reached maximum line count for file: {}", output_file_path);
                    file_index += 1;
                    output_file_path = format!("./data/data_nl_{}.csv", file_index);
                    create_file_if_not_exists(&output_file_path)?;
                    writer = csv::Writer::from_path(&output_file_path)?;
                    writer.write_record(&headers)?;
                    unique_line_count = 0;
                }
            }
        }
    }

    writer.flush()?;
    info!("Processing complete");
    info!("Total unique lines written: {}", unique_lines.len());

    info!("Done!");

    Ok(())
}
