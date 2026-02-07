use std::{
    env::{self, set_current_dir},
    ffi::OsString,
    path::{Path, PathBuf},
};
mod command_input;

#[allow(unused_imports)]
use std::io::{self, Write};
enum ShellAction {
    Continue,
    Exit,
}
use command_input::CommandInput;

#[derive(PartialEq, Debug)]
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

    let l  = key.len();
    loop {
        let mut input = String::new();

        print!("$ ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let command_input = CommandInput::new(input.to_string(), &paths);

        let action = match command_input.command_type {
            CommandType::Exit => ShellAction::Exit,
            CommandType::Type => command_input.type_command(),
            CommandType::Echo => command_input.echo(),
            CommandType::Exec { .. } => command_input.execute(),
            CommandType::PWD => print_working_directory(),
            CommandType::CD => change_directories(&command_input.args.join(" ")),
            CommandType::Unknown => command_input.command_not_found(),
        };

        match action {
            ShellAction::Continue => {}
            ShellAction::Exit => break,
        }
    }
}

fn change_directories(path: &str) -> ShellAction {
   if path == "~" {
        let home = env::home_dir();
        match home {
            Some(p) => {set_dir(&p);
            return ShellAction::Continue},
            None =>  {
                return ShellAction::Continue;
            }

        }
    
    };
     set_dir(&PathBuf::new().join(&path));
    return ShellAction::Continue;
}
fn set_dir(path: &PathBuf){

   match env::set_current_dir(path) {
        Ok(_) => (),
        Err(_) => {
            println!("cd: {}: No such file or directory", path.display());
        }
    };
}
fn print_working_directory() -> ShellAction {
    println!("{}", env::current_dir().unwrap().display());
    ShellAction::Continue
}
