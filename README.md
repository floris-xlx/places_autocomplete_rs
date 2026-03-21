# places_autocomplete_rs

In-memory Dutch address lookup API (Actix Web). Loads CSV files from a directory at startup.

## Environment variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `XLX_PLACES_DATA_DIR` | `data_split` | Directory containing `data_nl_*.csv` files to load (same path the generator should write to). |
| `XLX_PLACES_AUTOCOMPLETE_API_PORT` | `4444` | HTTP listen port. |

Optional: set `RUST_LOG` (e.g. `info`, `debug`) for `tracing` output.

## Offline CSV generation

Use `places_autocomplete_rs::generator::process_csv_files` with an input CSV whose header row matches:

`postal_code`, `street`, `house_number`, `city`, `area`, `neighborhood`, `municipality`, `province`, `latitude`, `longitude`

Output shards are written as `data_nl_{n}.csv` under `XLX_PLACES_DATA_DIR`. The generator keeps a full in-memory deduplication set; very large national datasets may require a different strategy.
