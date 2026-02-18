use std::io::Write;

pub fn write_to_dest(dest: &mut dyn Write, content: &str) {
    if let Err(e) = writeln!(dest, "{}", content) {
        eprintln!("Error writing to output: {}", e);
    }
    let _ = dest.flush();
}

pub fn split_by_delimiter<T: PartialEq + Clone>(vector: Vec<T>, delimiter: T) -> Vec<Vec<T>> {
    let mut result = Vec::new();
    let mut current = Vec::new();

    for item in vector.iter() {
        if *item == delimiter {
            if !current.is_empty() {
                result.push(current);
                current = Vec::new();
            }
        } else {
            current.push(item.clone());
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}
