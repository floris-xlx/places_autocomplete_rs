use std::fs;
use std::io;
use std::path::Path;

/// Lists all files in the `./csv_data` directory.
///
/// # Returns
///
/// A `Result` containing a vector of file names as `String` if successful, or an `io::Error` if an error occurs.
pub fn list_all_files_in_csv_data() -> io::Result<Vec<String>> {
    let path: &Path = Path::new("./csv_data");
    let mut file_names: Vec<String> = Vec::new();

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry: fs::DirEntry = entry?;
            let file_name: std::ffi::OsString = entry.file_name();
            if let Some(name) = file_name.to_str() {
                file_names.push(name.to_string());
            }
        }
    }

    Ok(file_names)
}
