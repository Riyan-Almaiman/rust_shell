use std::{fs::OpenOptions, io::Write, path::PathBuf};

use crate::{CommandType, ShellAction, command_input::CommandInput};

impl CommandInput  {

     pub fn echo(&self) -> ShellAction {
        
        let mut args = self.args.join(" ");
        args.push('\n');

        if let Some(file) = &self.redirect_std_out {
            return match OpenOptions::new()
                .create(true)
                .write(true)
                .append(!self.overwrite_std_out_redirect)
                .truncate(self.overwrite_std_out_redirect)
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

    pub fn type_command(&self, paths: &[PathBuf]) -> ShellAction {
        let cmd = self.args.get(0).map_or("", |v| v);
        if cmd.is_empty() {
            println!("No command found");
            return ShellAction::Continue;
        }
        let command_input = CommandInput::new(cmd.to_string(), paths);
        match command_input.command_type {
            CommandType::Unknown => println!("{}: not found", cmd),
            CommandType::Exec { path, .. } => println!("{} is {}", cmd, path.display()),
            _ => println!("{} is a shell builtin", cmd),
        }
        ShellAction::Continue
    }
       
   }
   
  