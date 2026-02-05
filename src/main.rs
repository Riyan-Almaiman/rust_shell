use std::{fs::read_dir, io::Read};
#[allow(unused_imports)]
use std::io::{self, Write};


fn main() {
    loop {
     let mut command = String::new();

    print!("$ ");
    io::stdout().flush().unwrap();
    
    io::stdin().read_line(&mut command).unwrap();
    
    let command = command.trim();
    match command {
        "exit" => break,
        _ => println!("{}: command not found", command.trim())
    }}
}
