use std::fs::File;
use std::io::BufReader;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;
use indicatif::ProgressBar;

// crate imports
use crate::io::create::create_file_if_not_exists;
use crate::io::list::list_all_files_in_csv_data;
use crate::parser::csv::open_csv_and_extract_headers;
use crate::parser::csv::{count_lines_in_csv, read_all_lines};
use crate::parser::enumurate_house_numbers::enumerate_house_numbers;

pub async fn process_csv_files() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file_path: &str = "./csv_data/postcodes_20190613.csv";
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

    // Create the output file if it doesn't exist
    let output_file_path = "./data/data_nl_1.csv";
    create_file_if_not_exists(output_file_path)?;

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

            use std::io::Write;
            writeln!(
                failed_file,
                "Error reading lines from {}: {:#?}",
                file_path, e
            )?;
        }
    }
    info!("Lines read successfully");

    // Initialize the progress bar
    let mut line_count = 0;
    let progress_bar = ProgressBar::new(0); // Will be updated later

    // Open the CSV file for reading
    let mut rdr = csv::Reader::from_path(file_path)?;
    let mut writer = csv::Writer::from_path(output_file_path)?;

    // Write headers to the output file
    writer.write_record(&headers)?;

    for result in rdr.records() {
        let record = result?;
        let line = record.iter().collect::<Vec<&str>>().join(",");
        let enumerated_lines = enumerate_house_numbers(&line);

        for enumerated_line in enumerated_lines {
            writer.write_record(enumerated_line.split(','))?;
            line_count += 1;
            progress_bar.inc(1);

            // Check if the file has reached the maximum line count
            if line_count >= 5_000_000 {
                writer.flush()?;
                info!("Reached maximum line count for file: {}", output_file_path);
                progress_bar.finish_with_message("Processing complete");
                return Ok(());
            }
        }
    }

    writer.flush()?;
    progress_bar.finish_with_message("Processing complete");
    info!("Total lines written: {}", line_count);

    // Count lines in the CSV
    if let Err(e) = count_lines_in_csv(output_file_path).await {
        error!("Error counting lines in CSV: {:#?}", e);
    } else {
        info!("Total lines in output CSV: {}", line_count);
    }

    info!("Done!");

    Ok(())
}
