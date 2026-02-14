use std::{env, fs::OpenOptions, io::Write, path::PathBuf};

use crate::{CommandType, ShellAction, Shell};

impl CommandInput {

    pub fn change_directories(shell: &mut Shell, path: &str) -> ShellAction {
        if path == "~" {
            let home = env::home_dir();
            match home {
                Some(p) => {
                    shell.set_current_dir(&p);
                    return ShellAction::Continue;
                }
                None => {
                    return ShellAction::Continue;
                }
            }
        };
        shell.set_current_dir(&PathBuf::new().join(&path));
        return ShellAction::Continue;
    }
    pub fn echo(&self) -> ShellAction {
        let mut args = self.args.join(" ");
        args.push('\n');

        if let Some(file) = &self.redirect_std_out {
            return match OpenOptions::new()
                .create(true)
                .write(true)
                .append(!self.overwrite_std_out_file)
                .truncate(self.overwrite_std_out_file)
                .open(file)
            {
                Ok(mut file) => {
                    file.write_all(args.as_bytes())
                        .expect("failed to write to file");
                    ShellAction::Continue
                }
                Err(error) => ShellAction::Error(error.to_string()),
            };
        }
        if let Some(file) = &self.redirect_std_error {
            match OpenOptions::new()
                .create(true)
                .write(true)
                .append(!self.overwrite_std_err_file)
                .truncate(self.overwrite_std_err_file)
                .open(file)
            {
                Ok(mut file) => {
                    file.write_all("".as_bytes())
                        .expect("failed to write to file");
                }
                Err(error) => return ShellAction::Error(error.to_string()),
            }
        }
        println!("{}", self.args.join(" "));

        ShellAction::Continue
    }

    pub fn type_command(&self, shell: &Shell) -> ShellAction {
        let cmd = self.args.get(0).map_or("", |v| v);
        if cmd.is_empty() {
            println!("No command found");
            return ShellAction::Continue;
        }
        let command_input = CommandInput::new(cmd.to_string(), shell);
        match command_input.command_type {
            CommandType::Unknown => println!("{}: not found", cmd),
            CommandType::Exec { path, .. } => println!("{} is {}", cmd, path.display()),
            _ => println!("{} is a shell builtin", cmd),
        }
        ShellAction::Continue
    }
}
