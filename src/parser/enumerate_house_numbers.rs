use csv::StringRecord;

/// Expands house number ranges like `1 t/m 5` into one record per number; otherwise returns the row unchanged.
pub fn enumerate_house_numbers(record: &StringRecord) -> Vec<StringRecord> {
    let mut out = Vec::new();
    if record.len() < 3 {
        return out;
    }
    let house_numbers = record.get(2).unwrap_or("");
    if let Some(range_pos) = house_numbers.find(" t/m ") {
        let start = house_numbers[..range_pos].trim();
        let end = house_numbers[range_pos + 5..].trim();
        if let (Ok(start_num), Ok(end_num)) = (start.parse::<u32>(), end.parse::<u32>()) {
            for num in start_num..=end_num {
                let mut fields: Vec<String> = (0..record.len())
                    .map(|i| record.get(i).unwrap_or("").to_string())
                    .collect();
                fields[2] = num.to_string();
                out.push(StringRecord::from(fields));
            }
        } else {
            out.push(record.clone());
        }
    } else {
        out.push(record.clone());
    }
    out
}
