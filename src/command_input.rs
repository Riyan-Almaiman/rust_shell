use std::{env, path::{self, PathBuf}};

use crate::{CommandType, ShellAction};
use is_executable::is_executable;

#[derive(Debug)]
pub struct CommandInput<'a> {
    pub command_type: CommandType,
   pub command_str: &'a str,
   pub args: Vec<&'a str>,
    pub raw_args: &'a str,
    pub paths: &'a [PathBuf]
    
}
impl<'a> CommandInput<'a> {
    pub fn new(input: &'a str, paths: &'a [PathBuf]) -> Self {
        let (command, args) = input.split_once(' ').unwrap_or((input, ""));
        let cmd = match command {
            "exit" => CommandType::Exit,
            "echo" => CommandType::Echo,
            "type" => CommandType::Type,
            "pwd" => CommandType::PWD,
            "cd"=> CommandType::CD,
            _ => Self::parse_unknown(command, paths),
        };

         Self {
            paths: paths,
            command_type: cmd,
            command_str: command,
            args: args.split_whitespace().collect(),
            raw_args: args,
        }
  
    }
    pub fn get_exe_command(command: &str) -> PathBuf {
        if cfg!(target_os = "windows") && !command.ends_with(".exe") {

            PathBuf::from(format!("{}.exe", command))
        } else {
            PathBuf::from(command)
        }
    }
    fn parse_unknown(command: &str, paths: &[PathBuf]) -> CommandType {
        let exe_name = Self::get_exe_command(command);
        for path in paths {
            let file = path.join(&exe_name);
            if is_executable(&file) {
                return CommandType::Exec {
                    command: exe_name,
                    path: file.into_os_string(),
                };
            }
        }
        CommandType::Unknown
    }
  
   pub fn execute(&self) -> ShellAction {
        let (cmd, ..) = match &self.command_type {
            CommandType::Exec { command, path } => (command, path),
            _ => return ShellAction::Continue
        };
        let mut process = std::process::Command::new(cmd)
            .args(&self.args)
            .spawn()
            .expect("failed to spawn process");

        process.wait().expect("failed to wait for process");

        return ShellAction::Continue;
    }
pub fn echo(&self ) -> ShellAction {
    println!("{}", self.raw_args);
    ShellAction::Continue
}

pub fn type_command(&self) -> ShellAction {
    let cmd = self.args.get(0).map_or("", |v| v);
    if cmd.is_empty() {
        println!("No command found");
        return ShellAction::Continue;
    }
    let command_input = CommandInput::new(cmd, &self.paths);
    match command_input.command_type {
        CommandType::Unknown => println!("{}: not found", cmd),
        CommandType::Exec { path, .. } => println!("{} is {}", cmd, path.display()),
        _ => println!("{} is a shell builtin", cmd),
    }
    ShellAction::Continue
}
pub fn command_not_found(&self) -> ShellAction {
    println!("{}: command not found", self.command_str);
    ShellAction::Continue
}

}