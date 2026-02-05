
use std::{env, ffi::{OsStr, OsString} };
use is_executable::{is_executable};

#[allow(unused_imports)]
use std::io::{self, Write};
enum ShellAction {
    Continue,
    Exit,
}

#[derive(Debug)]
struct CommandInput {
    command: CommandType,
    command_str: String,
    args: Vec<String>,
    raw_args: String,
    
}
#[derive(Debug)]

enum CommandType {
    Exit , 
    Echo,
    Type ,
    Exec {name: String, path: OsString},
    Unknown,
}

impl CommandType {

    fn parse( input: &str, paths: &OsStr) -> CommandType {
        match input {
            "exit" => CommandType::Exit,
            "echo" => CommandType::Echo,
             "type"=> CommandType::Type,
            _ => CommandType::parse_unknown(&input, &paths)
        }
    }


    fn parse_unknown(command: &str,paths:&OsStr) -> CommandType{

        let paths_split = env::split_paths(paths);
        for path in paths_split {
            let file = path.join(command );
            if is_executable(&file){
                return CommandType::Exec {name:command.to_string(), path: file.into_os_string()};
            }
        }
        CommandType::Unknown
}
}

fn main() {

    let key = "PATH";
    let paths =  std::env::var_os(key).unwrap_or(OsString::from(""));
    
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
            

        let  input_tokens =
             args
             .split_whitespace()
            .map(String::from);

        let command_input = CommandInput {
            command_str: command.to_string(),
            command: CommandType::parse(command, &paths),
            args: input_tokens.collect(),
            raw_args: args.to_string(),
        };

        let action = match command_input.command {

            CommandType::Exit => ShellAction::Exit,
            CommandType::Type=> type_command(&command_input.args.get(0).unwrap_or(&"".to_string()),  &paths),
            CommandType::Echo =>  echo(&command_input.raw_args),
            CommandType::Exec { .. } => execute( &command_input),
            CommandType::Unknown  => command_not_found(&command_input.command_str)
        };

        match action {
            ShellAction::Continue => {},
            ShellAction::Exit => break,
            
        }
  }
}


fn execute( command_input: &CommandInput) -> ShellAction{

  
    let mut process = std::process::Command::new(command_input.command_str.clone())
        .args(command_input.args.clone())
        .spawn().expect("failed to spawn process");

     process.wait().expect("failed to wait for process");

    return ShellAction::Continue;

}
fn type_command(command: &str, paths: &OsStr)->ShellAction{
    if command.is_empty(){
        println!("No command found");
        return ShellAction::Continue
    }
    
    match CommandType::parse(command, paths){
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