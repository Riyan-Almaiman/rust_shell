use std::{fs::read_dir, io::Read};
#[allow(unused_imports)]
use std::io::{self, Write};



fn main() {

  
    let mut exit = false;
    while !exit {
        let mut input = String::new();

        print!("$ ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.trim().is_empty() {
            println!("No Command Found");
            continue;
        }
        let command_string: Vec<String> = input.split(' ').map(String::from).collect();
 
    
        let command = &command_string[0];
        let command_args = &command_string[1..];
        
        exit = match command.as_str() {

            "exit" => true,
            "echo" =>  echo(command_args),
            _ => command_not_found(command)
        }
  }
}

fn command_not_found(command: &str) -> bool {

    println!("{}: command not found", command);
    false
}

fn echo(message: &[String] )-> bool{

    println!("{}", message.join(" "));
    true
}