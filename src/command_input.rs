use std::{env, path::{self, PathBuf}};

use crate::{CommandType, ShellAction};
use is_executable::is_executable;

#[derive(Debug)]
pub struct CommandInput<'a> {
    pub command_type: CommandType,
   pub command_str: String,
   pub args: Vec<String>,
    pub raw_args: String,
    pub paths: &'a [PathBuf]
    
}
   fn extract_single_quoted(input: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut start: Option<usize> = None;
    let mut found = false;
    let mut token: Option<String> = None;
    for (i, c) in input.char_indices() {
        if c == '\'' {
            match start {
                None => start = Some(i + 1), // opening '
                Some(begin) => {
                    if begin != i { 
                        if input.chars().nth(i+1).unwrap_or('\'') != '\'' &&  input.chars().nth(i-1).unwrap_or('\'') != '\''{
                        result.push(format!("'{}'", input[begin..i].to_string())); // closing '
                        start = None;
                    }
                    }
                    else{
                       
                        start = None;
                    }
                    
                }
            }
        }else{
        match start {
            

            None=> match token{
                None=> if !c.is_whitespace() {token = Some(String::from(c)); },
                Some(  ref mut current) =>  match c.is_whitespace() || c== '\''  {
                    false=> {current.push(c);     },
                    true=>{result.push(current.trim().to_string()); token = None;}
                }
            },
            Some(_)=>()
        };
    }}
    match token {
        None=> match start {
            None=> (),
            Some(l)=>{result.push(format!("'{}", input[l..].to_string()));
        }
    },
            Some(l)=> result.push(l.trim().to_string())

}

    result
}
impl<'a> CommandInput<'a> {
 

    pub fn new(input: String, paths: &'a [PathBuf]) -> Self {
        let (command, args) = input.split_once(' ').unwrap_or((&input, ""));
        let cmd = match command {
            "exit" => CommandType::Exit,
            "echo" => CommandType::Echo,
            "type" => CommandType::Type,
            "pwd" => CommandType::PWD,
            "cd"=> CommandType::CD,
            _ => Self::parse_unknown(command, paths),
        };
         Self {
            paths,
            command_type: cmd,
            command_str: command.to_string(),
            args:extract_single_quoted(args),
            raw_args: args.to_string(),
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
    println!("{}", self.args.join(" ").replace("'", ""));
    ShellAction::Continue
}

pub fn type_command(&self) -> ShellAction {
    let cmd = self.args.get(0).map_or("", |v| v);
    if cmd.is_empty() {
        println!("No command found");
        return ShellAction::Continue;
    }
    let command_input = CommandInput::new(cmd.to_string(), &self.paths);
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