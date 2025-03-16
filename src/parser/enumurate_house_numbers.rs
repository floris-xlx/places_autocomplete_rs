pub fn enumerate_house_numbers(line: &str) -> Vec<String> {
    let mut result = Vec::new();
    let parts: Vec<&str> = line.split(',').collect();

    if parts.len() < 3 {
        return result; 
    }

    let house_numbers = parts[2];
    if let Some(range_pos) = house_numbers.find(" t/m ") {
        let start = &house_numbers[..range_pos].trim();
        let end = &house_numbers[range_pos + 5..].trim();

        if let (Ok(start_num), Ok(end_num)) = (start.parse::<u32>(), end.parse::<u32>()) {
            for num in start_num..=end_num {
                let mut new_line = parts.clone();
                let num_string = num.to_string();
                new_line[2] = &num_string;
                result.push(new_line.join(","));
            }
        }
    } else {
        result.push(line.to_string());
    }

    result
}
