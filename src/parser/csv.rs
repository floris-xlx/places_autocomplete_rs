use csv::ReaderBuilder;
use std::path::Path;

/// Ensures the CSV has a header row matching `expected` (same length, same strings in order).
pub fn validate_csv_headers<P: AsRef<Path>>(
    path: P,
    expected: &[&str],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path(path.as_ref())?;
    let hdrs = rdr.headers()?;
    if hdrs.len() != expected.len() {
        return Err(format!(
            "CSV header count mismatch: expected {} columns, got {}",
            expected.len(),
            hdrs.len()
        )
        .into());
    }
    for (i, exp) in expected.iter().enumerate() {
        if hdrs.get(i) != Some(*exp) {
            return Err(format!(
                "CSV header mismatch at column {}: expected {:?}, got {:?}",
                i,
                exp,
                hdrs.get(i)
            )
            .into());
        }
    }
    Ok(())
}
