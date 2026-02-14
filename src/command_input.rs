use crate::parser::{self, parse_input};
use crate::{CommandType, Shell, ShellAction};

use std::fs::OpenOptions;

use std::{path::PathBuf, process};

#[derive(Debug)]
pub struct CommandInput {

    pub commands: Vec<Cmd>,
    pub tokens: Vec<String>,
}
#[derive(Debug)]
pub  struct Cmd {
    pub command_type: CommandType,
    pub path: Option<PathBuf>,
    pub command_str: String,
    pub args: Vec<String>,
    pub redirect_std_out: Option<RedirectionType>,
    pub redirect_std_error: Option<RedirectionType>,
}
#[derive(Debug)]

pub enum RedirectionType {
    File {
        filename: String,
        overwrite: bool,
    },
    NextCmd {
     cmd: Option<Box<Cmd>>,
    },
}
impl CommandInput {
    pub fn new(input: String, shell: &Shell) -> Self {
        let mut command_input = CommandInput {
    
            tokens: parse_input(&input),
            commands: vec![],
        };
             
        parser::parse_redirections(&mut command_input);

        command_input.command_str = command_input.tokens.get(0).cloned().unwrap_or_default();

        command_input.args = command_input.tokens.get(1..).map_or(vec![], |s| s.to_vec());

        parser::parse_commandtype_from_cmd_str(&mut command_input, shell);
        for token in &command_input.tokens {
            println!("Token: {}", token);
        }

        command_input
    }

    pub fn execute(&self) -> ShellAction {
        let (cmd, _path) = match &self.command_type {
            CommandType::Exec { command, path } => (command, path),
            _ => return ShellAction::Continue,
        };
        let mut process = process::Command::new(cmd);
        let mut process = process.args(&self.args);
        if let Some(file) = &self.redirect_std_out {
            match OpenOptions::new()
                .create(true)
                .write(true)
                .append(!self.overwrite_std_out_file)
                .truncate(self.overwrite_std_out_file)
                .open(file)
            {
                Ok(file) => process = process.stdout(file),
                Err(error) => return ShellAction::Error(error.to_string()),
            }
        }
        if let Some(file) = &self.redirect_std_error {
            match OpenOptions::new()
                .create(true)
                .write(true)
                .append(!self.overwrite_std_err_file)
                .truncate(self.overwrite_std_err_file)
                .open(file)
            {
                Ok(file) => process = process.stderr(file),
                Err(error) => return ShellAction::Error(error.to_string()),
            }
        }
        process
            .spawn()
            .expect("failed to spawn process")
            .wait()
            .expect("failed to wait on process");

        return ShellAction::Continue;
    }

    pub fn command_not_found(&self) -> ShellAction {
        println!("{}: command not found", self.command_str);
        ShellAction::Continue
    }
}
