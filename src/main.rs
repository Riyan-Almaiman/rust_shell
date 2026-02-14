use std::{
    ffi::OsString,
    path::PathBuf,
};
mod command_input;
mod parser;
use rustyline::{
     error::ReadlineError,
};
#[allow(unused_imports)]
use std::io::{self, Write};

mod shell;
use shell::Shell;
use command_input::CommandInput;

use crate::shell::ShellAction;
mod builtin;
#[derive(PartialEq, Debug)]
enum CommandType {
    Exit,
    Echo,
    Type,
    Exec ,
    PWD,
    CD,
    Unknown,
}
mod completion_helper;

fn main() {
    let mut shell = Shell::new("PATH", "$ ");

    loop {
        let input = match shell.read_line.readline(&shell.prompt) {
            Ok(line) => line,

            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
        .trim()
        .to_string();

        if input.is_empty() {
            continue;
        }

        let command_input = CommandInput::new(input.to_string(), &shell);

        let action = match command_input.command_type {
            CommandType::Exit => ShellAction::Exit,
            CommandType::Type => command_input.type_command(&shell),
            CommandType::Echo => command_input.echo(),
            CommandType::Exec => command_input.execute(),
            CommandType::PWD => shell.print_current_dir(),
            CommandType::CD => shell.change_directories(&command_input.args.join(" ")),
            CommandType::Unknown => command_input.command_not_found(),
        };

        match action {
            ShellAction::Continue => continue,
            ShellAction::Exit => break,
            ShellAction::Error(msg) => {
                writeln!(&mut io::stderr(), "{}", msg).unwrap();
                continue;
            }
        }
    }
}
