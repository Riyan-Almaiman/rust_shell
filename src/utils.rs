use std::io::Write;

pub fn write_to_dest(dest: &mut dyn Write, content: &str) {

    if let Err(e) = writeln!(dest, "{}", content) {
        eprintln!("Error writing to output: {}", e);
    }
    let _ = dest.flush();

}