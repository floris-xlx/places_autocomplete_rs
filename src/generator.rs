use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;

use tracing::info;

use crate::io::create::create_file_if_not_exists;
use crate::parser::csv::validate_csv_headers;
use crate::parser::enumerate_house_numbers::enumerate_house_numbers;
use crate::places_data_dir;

const RECORD_SEP: &str = "\x1e";

fn record_dedup_key(record: &csv::StringRecord) -> String {
    record.iter().collect::<Vec<_>>().join(RECORD_SEP)
}

fn list_filenames_in_directory(directory: &PathBuf) -> std::io::Result<Vec<String>> {
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

fn next_data_nl_index(data_dir: &PathBuf) -> std::io::Result<usize> {
    let files = list_filenames_in_directory(data_dir)?;
    let mut max_index: usize = 0;
    for file in files {
        if let Some(index_str) = file
            .strip_prefix("part_")
            .and_then(|s| s.strip_suffix(".csv"))
        {
            if let Ok(index) = index_str.parse::<usize>() {
                if index > max_index {
                    max_index = index;
                }
            }
        }
    }
    Ok(if max_index == 0 { 1 } else { max_index + 1 })
}

pub async fn process_csv_files(
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let headers: Vec<&str> = vec![
        "postal_code",
        "street",
        "house_number",
        "city",
        "area",
        "neighborhood",
        "municipality",
        "province",
        "latitude",
        "longitude",
    ];

    validate_csv_headers(file_path, &headers)?;

    let data_dir = PathBuf::from(places_data_dir());
    std::fs::create_dir_all(&data_dir)?;

    let mut unique_line_count = 0usize;
    let mut file_index = next_data_nl_index(&data_dir)?;

    let mut rdr: csv::Reader<File> = csv::Reader::from_path(file_path)?;
    let mut output_path = data_dir.join(format!("data_nl_{}.csv", file_index));
    create_file_if_not_exists(output_path.to_str().ok_or("invalid output path")?)?;
    let mut writer: csv::Writer<File> = csv::Writer::from_path(&output_path)?;

    writer.write_record(&headers)?;

    // Holds full expanded rows in memory; large national runs may need a different dedup strategy.
    let mut unique_lines: HashSet<String> = HashSet::new();

    for result in rdr.records() {
        let record: csv::StringRecord = result?;
        for expanded in enumerate_house_numbers(&record) {
            let key = record_dedup_key(&expanded);
            if unique_lines.insert(key) {
                writer.write_record(expanded.iter())?;
                unique_line_count += 1;

                if unique_line_count >= 1_000_000 {
                    writer.flush()?;
                    info!(
                        "Reached maximum line count for file: {}",
                        output_path.display()
                    );
                    file_index += 1;
                    output_path = data_dir.join(format!("data_nl_{}.csv", file_index));
                    create_file_if_not_exists(output_path.to_str().ok_or("invalid output path")?)?;
                    writer = csv::Writer::from_path(&output_path)?;
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
