
use std::{env, ffi::OsString };
use is_executable::is_executable;

#[allow(unused_imports)]
use std::io::{self, Write};
enum ShellAction {
    Continue,
    Exit,
}

struct CommandInput {
    command: CommandTypes,
    args: Vec<String>,
    raw_args: String,
}

enum CommandTypes {
    Exit, 
    Echo,
    Type ,
    Unknown {name: String, path: Option<OsString>},
}

impl CommandTypes {

    fn parse( input: &str, paths: &OsString) -> CommandTypes {
        match input {
            "exit" => CommandTypes::Exit,
            "echo" => CommandTypes::Echo,
             "type"=> CommandTypes::Type,
            _ => CommandTypes::Unknown{ name: input.to_string(), path: CommandTypes::find_exec(input, &paths)},
        }
    }
    fn find_exec(command: &str,paths: &OsString) -> Option<OsString>{

        let paths_split = env::split_paths(paths);
        for path in paths_split {
            let file = path.join(command );
            if is_executable(&file){
                return Some(file.into_os_string());
            }
        }
        None
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

        let args = CommandInput {
            command: CommandTypes::parse(command, &paths),
            args: input_tokens.collect(),
            raw_args: args.to_string(),
        };


        let action = match args.command {

            CommandTypes::Exit => ShellAction::Exit,
            CommandTypes::Type=> is_builtin(&args.args.get(0).unwrap_or(&"".to_string()),  &paths),
            CommandTypes::Echo =>  echo(&args.raw_args),
            CommandTypes::Unknown { name, .. } => command_not_found(&name)
        };

        match action {
            ShellAction::Continue => {},
            ShellAction::Exit => break,
            
        }
  }
}



fn is_builtin(command: &String, paths: &OsString)->ShellAction{
    if command.is_empty(){
        println!("No command found");
        return ShellAction::Continue
    }
    
    match CommandTypes::parse(command, paths){
        CommandTypes::Unknown { name, path } =>{
            match path{
                Some(p) => {
             println!("{} is {}", name, p.to_str().unwrap());       
                }
                None => {
                    println!("{}: not found", name);
                }
            }}
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