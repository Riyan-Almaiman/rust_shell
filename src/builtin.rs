use std::{env, fs::OpenOptions, io::Write, path::PathBuf};

use crate::{ ShellAction, Shell};

use crate::command_input::{Cmd, CommandType};
use crate::utils::write_to_dest;


    pub fn exit() -> ShellAction {
        ShellAction::Exit
    }

    pub fn print_current_dir(shell: &mut Shell, dest: &mut dyn Write) -> ShellAction {
        write_to_dest(dest, format!("{}", shell.current_dir.display()).as_str());
        ShellAction::Continue
    }
    pub fn set_current_dir(shell: &mut Shell, path: &PathBuf, dest_err: &mut dyn Write) {
        match env::set_current_dir(path) {
            Ok(_) => shell.current_dir = env::current_dir().unwrap(),
            Err(_) => {
                write_to_dest(dest_err, format!("cd: {}: No such file or directory", path.display()).as_str());
            }
        };
    }
    pub fn change_directories(shell: &mut Shell, path: &str, dest_out: Option<&mut dyn Write>, dest_err: &mut dyn Write ) -> ShellAction {

    
        if path == "~" {
            let home = env::home_dir();
            match home {
                Some(p) => {
                    set_current_dir(shell, &p, dest_err);
                    return ShellAction::Continue;
                }
                None => {
                    return ShellAction::Continue;
                }
            }
        };
        set_current_dir( shell, &PathBuf::from(&path), dest_err);
        return ShellAction::Continue;
    }
    pub fn echo(args: &[String], dest: &mut dyn Write) -> ShellAction {
        let content = args.join(" ");

        write_to_dest(dest, &content);

        ShellAction::Continue
    }

pub fn type_command(cmd: &Cmd, dest: &mut dyn Write)-> ShellAction {

        match &cmd.command_type {
            CommandType::Unknown{..} => write_to_dest(dest, format!("{}: not found", cmd.command_str).as_str()),
            CommandType::External{args, path} => {
                write_to_dest(dest, format!("{} is {}", cmd.command_str, path.display()).as_str())
            }
            _ => write_to_dest(dest,format!("{} is a shell builtin", cmd.command_str).as_str()),
        }
        ShellAction::Continue
    }



