
use std::{env, ffi::{ OsString}, path::PathBuf };
use is_executable::{is_executable};

#[allow(unused_imports)]
use std::io::{self, Write};
enum ShellAction {
    Continue,
    Exit,
}

struct CommandInput<'a> {
    command_type: CommandType,
    command_str: &'a str,   
    args: Vec<&'a str>,    
    raw_args: &'a str,      
}
impl<'a> CommandInput<'a>{
    fn execute(&self) -> ShellAction{

        let mut process = std::process::Command::new(self.command_str)
            .args(&self.args)
            .spawn().expect("failed to spawn process");

        process.wait().expect("failed to wait for process");

        return ShellAction::Continue;

    }
}
#[derive(PartialEq)]
enum CommandType {
    Exit , 
    Echo,
    Type ,
    Exec {name: String, path: OsString},
    Unknown,
}

impl CommandType {

    fn parse( input: &str) -> CommandType {
        match input {
            "exit" => CommandType::Exit,
            "echo" => CommandType::Echo,
             "type"=> CommandType::Type,
            _ => CommandType::Unknown
        }
    }

    fn parse_unknown(command: &str, paths: &[PathBuf]) -> CommandType{
    
        for path in paths {
            let file = path.join(command );
            if is_executable(&file){
                return CommandType::Exec {name:command.to_string(), path: file.into_os_string()};
            }
        }
        CommandType::Unknown
    }
    fn parse_type(command: &str, paths: &[PathBuf]) -> CommandType{
        
        let cmd_type = match CommandType::parse(command){
            CommandType::Unknown=> CommandType::parse_unknown(command, paths),
            builtin=> builtin

        };
        cmd_type
    }
}

fn main() {

    let key = "PATH";
    let paths: Vec<PathBuf> = env::split_paths(&std::env::var_os(key).unwrap_or(OsString::from(""))).collect();
    
    loop {
        let mut input = String::new();

        print!("$ ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }
        let  ( command,  args) = input
                .split_once(' ')
                .unwrap_or((input, ""));
            
        let type_of_command = match CommandType::parse(command) {
                CommandType::Unknown => CommandType::parse_unknown(command, &paths),
                cmd =>  cmd

        };
  
        let command_input = CommandInput {
            command_str: command,
            command_type: type_of_command ,
            args:  args.split_whitespace().collect(),
            raw_args: args,
        };

        let action = match command_input.command_type {

            CommandType::Exit => ShellAction::Exit,
            CommandType::Type=> type_command(&command_input.args.get(0).map_or("", |v| v),  &paths),
            CommandType::Echo =>  echo(&command_input.raw_args),
            CommandType::Exec { .. } => command_input.execute(),
            CommandType::Unknown  => command_not_found(&command_input.command_str)
        };

        match action {
            ShellAction::Continue => {},
            ShellAction::Exit => break,
            
        }
  }
}



fn type_command(command: &str, paths:&[PathBuf] )->ShellAction{
    if command.is_empty(){
        println!("No command found");
        return ShellAction::Continue
    }
    
    match CommandType::parse_type(command, paths){
        CommandType::Unknown =>  println!("{}: not found", command),
        CommandType::Exec { path, name } => println!("{} is {}", name, path.display()),
        _ => println!("{} is a shell builtin", command)
    }
    ShellAction::Continue
}
fn command_not_found(command: &str) -> ShellAction {

    println!("{}: command not found", command);
    ShellAction::Continue
}

fn echo(message: &str )-> ShellAction{

    println!("{}", message);
    ShellAction::Continue
}