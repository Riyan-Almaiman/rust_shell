
#[allow(unused_imports)]
use std::io::{self, Write};
enum ShellAction {
    Continue,
    Exit,
}

struct CommandInput {
    command: CommandTypes,
    args: Vec<String>,
    raw_args: String
}

enum CommandTypes {
    Exit, 
    Echo,
    Unknown {name: String},
}

impl CommandTypes {

    fn parse( input: &str) -> CommandTypes {
        match input {
            "exit" => CommandTypes::Exit,
            "echo" => CommandTypes::Echo,
            _ => CommandTypes::Unknown{ name: input.to_string()},
        }
    }
}

fn main() {

  
    loop {
        let mut input = String::new();

        print!("$ ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }
        let mut input_tokens =
             input
             .split_inclusive(" ")
            .map(String::from);

        let command_input = CommandInput {
            command: CommandTypes::parse(&input_tokens.next().unwrap().trim()),
            args: input_tokens.clone().filter_map(|c| if !c.trim().is_empty(){
                Some(c.trim().to_string())
            }else{
                None
            }
            
        ).collect(),
            raw_args: input_tokens.collect()
        };
 

        let action = match command_input.command {

            CommandTypes::Exit => ShellAction::Exit,
            CommandTypes::Echo =>  echo(&command_input.raw_args),
            CommandTypes::Unknown { name } => command_not_found(&name)
        };

        match action {
            ShellAction::Continue => {},
            ShellAction::Exit => break,
            
        }
  }
}

fn command_not_found(command: &str) -> ShellAction {

    println!("{}: command not found", command);
    ShellAction::Continue
}

fn echo(message: &str )-> ShellAction{

    println!("{}", message);
    ShellAction::Continue
}