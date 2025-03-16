use moka::future::Cache;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;


pub mod api;
pub mod cache;
pub mod parser;
pub mod io;
pub mod generator;
pub mod query;

/// Define a type alias for the shared cache
pub type SharedCache = Arc<Mutex<Cache<String, Value>>>;