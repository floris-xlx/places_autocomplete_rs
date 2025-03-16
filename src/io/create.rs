use std::fs;
use std::io;
use std::path::Path;

/// Creates a file at the specified path if it does not already exist.
/// If the file already exists, the function does nothing.
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the path of the file to be created.
///
/// # Returns
///
/// A `Result` which is `Ok` if the file was created or already exists, or an `io::Error` if an error occurs.
pub fn create_file_if_not_exists(file_path: &str) -> io::Result<()> {
    let path = Path::new(file_path);

    if !path.exists() {
        fs::File::create(path)?;
    }

    Ok(())
}
