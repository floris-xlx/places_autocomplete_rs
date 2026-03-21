//! CSV load path and generator output directory (see README).
pub const XLX_PLACES_DATA_DIR_ENV: &str = "XLX_PLACES_DATA_DIR";

/// Directory containing `data_nl_*.csv` (and optional `part_*.csv` shards for resume). Default: `data_split`.
pub fn places_data_dir() -> String {
    std::env::var(XLX_PLACES_DATA_DIR_ENV).unwrap_or_else(|_| "data_split".into())
}

pub mod api;
pub mod parser;
pub mod io;
pub mod generator;
pub mod query;
