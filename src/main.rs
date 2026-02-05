use std::{fs::read_dir, io::Read};
#[allow(unused_imports)]
use std::io::{self, Write};


fn main() {
    let mut command = String::new();
    print!("$ ");
    io::stdout().flush().unwrap();
    
    io::stdin().read_line(&mut command).unwrap();
    
    
    match command {

        _ => println!("{}: command not found", command.trim())
    }
}
