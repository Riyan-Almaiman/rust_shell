
use std::{
    env::{self, set_current_dir},
    ffi::OsString,
    path::PathBuf,
};
mod command_input;
#[allow(unused_imports)]
use std::io::{self, Write};
enum ShellAction {
    Continue,
    Exit,
}
use command_input::CommandInput;

#[derive(PartialEq)]
#[derive(Debug)]
enum CommandType {
    Exit,
    Echo,
    Type,
    Exec { command: PathBuf, path: OsString },
    PWD,
    CD,
    Unknown,
}

fn main() {
    let key = "PATH";
    let paths: Vec<PathBuf> =
        env::split_paths(&std::env::var_os(key).unwrap_or(OsString::from(""))).collect();

    loop {
        let mut input = String::new();

        print!("$ ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let command_input = CommandInput::new(input, &paths);

        let action = match command_input.command_type {
            CommandType::Exit => ShellAction::Exit,
            CommandType::Type => command_input.type_command(),
            CommandType::Echo => command_input.echo(),
            CommandType::Exec { .. } => command_input.execute(),
            CommandType::PWD => print_working_directory(),
            CommandType::CD => change_directories(command_input.raw_args),
            CommandType::Unknown => command_input.command_not_found(),
            
            
        };

        match action {
            ShellAction::Continue => {}
            ShellAction::Exit => break,
        }
    }
}

fn change_directories(path: &str) -> ShellAction {

        match env::set_current_dir(path){
            Ok(_) => (),
            Err(e) => {
                println!("{}", e);
                
            } 
        };
        return ShellAction::Continue;
    
    }
 fn print_working_directory() -> ShellAction{
    println!("{}", env::current_dir().unwrap().display());
    ShellAction::Continue
}