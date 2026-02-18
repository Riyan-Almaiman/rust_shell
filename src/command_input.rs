use crate::parser::parse_input;
use crate::{Shell, ShellAction};

use std::fs::{File, OpenOptions};

use is_executable::is_executable;
use os_pipe::pipe;
use std::io;
use std::path::PathBuf;

use crate::builtin::{change_directories, echo, exit, print_current_dir, type_command};
use crate::utils::split_by_delimiter;
use std::process::{Child, Command, Stdio};
use crate::redirection::Redirection;

#[derive(Debug)]

pub enum BuiltInCommand {
    Exit,
    Echo(Vec<String>),
    Type(Vec<String>),
    CD(Vec<String>),
    PWD,
    History
}
#[derive(Debug)]
pub enum CommandType {
    Builtin(BuiltInCommand),
    External {
        name: PathBuf,
        path: PathBuf,
        args: Vec<String>,
    },
    Unknown(String),
}


#[derive(Debug)]
pub struct Cmd {
    pub command_type: CommandType,
    pub command_str: String,
    pub redirect_std_out: Option<Redirection>,
    pub redirect_std_error: Option<Redirection>,
    pub child: Option<Box<Cmd>>,

}



impl Cmd {
    pub fn new(input: &str, shell: &Shell) -> Option<Self> {
        let tokens = parse_input(&input);
        let mut cmds_split_by_pipe = split_by_delimiter(tokens.clone(), "|".to_string());

        let command = Self::build_piped_commands(&mut cmds_split_by_pipe, shell);

        command
    }

    pub fn build_piped_commands(cmd_tokens: &mut Vec<Vec<String>>, shell: &Shell) -> Option<Cmd> {
        if cmd_tokens.is_empty() {
            return None;
        }

        let mut current_cmd: Option<Cmd> = None;

        while let Some(mut tokens) = cmd_tokens.pop() {
            if tokens.is_empty() {
                continue;
            }
            let (std_out_file, std_err_file) = Redirection::parse_redirections(&mut tokens);

            let command_str = tokens.remove(0);
            let cmd = Self::get_command_type_from_cmd_name(command_str.as_str(), tokens, &shell);

            let cmd = Cmd {
                command_type: cmd,
                child: match current_cmd {
                    None => None,
                    Some(cmd) => Some(Box::new(cmd)),
                },
                command_str,
                redirect_std_out: std_out_file,
                redirect_std_error: std_err_file,
            };

            current_cmd = Some(cmd);
        }
        current_cmd
    }


     fn get_command_type_from_cmd_name(
        cmd: &str,
        args: Vec<String>,
        shell: &Shell,
    ) -> CommandType {
        match cmd {
            "exit" => CommandType::Builtin(BuiltInCommand::Exit),
            "echo" => CommandType::Builtin(BuiltInCommand::Echo(args)),
            "type" => CommandType::Builtin(BuiltInCommand::Type(args)),
            "pwd" => CommandType::Builtin(BuiltInCommand::PWD),
            "cd" => CommandType::Builtin(BuiltInCommand::CD(args)),
            "history" => CommandType::Builtin(BuiltInCommand::History),
            _ => {
                let exe_name = if cfg!(target_os = "windows") && !cmd.ends_with(".exe") {
                    PathBuf::from(format!("{}.exe", cmd))
                } else {
                    PathBuf::from(cmd)
                };
                let executable = shell.executables.iter().find(|exe| exe.name == exe_name);
                match executable {
                    Some(exe) => {
                        if is_executable(&exe.path) {
                            CommandType::External {
                                args,
                                path: exe.path.clone(),
                                name: exe_name,
                            }
                        } else {
                            CommandType::Unknown(cmd.to_string())
                        }
                    }
                    None => CommandType::Unknown(cmd.to_string()),
                }
            }
        }
    }


    pub fn command_not_found(&self) -> ShellAction {
        println!("{}: command not found", self.command_str);
        ShellAction::Continue
    }
    pub fn flatten(&self) -> Vec<&Cmd> {
        let mut cmds = Vec::new();
        let mut current = Some(self);

        while let Some(cmd) = current {
            cmds.push(cmd);
            current = cmd.child.as_deref();
        }

        cmds
    }
}
