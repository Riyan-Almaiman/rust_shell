
#[allow(unused_imports)]
use std::io::{self, Write};
enum ShellAction {
    Continue,
    Exit,
}

struct CommandInput {
    command: CommandTypes,
    args: Vec<String>
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
            println!("No Command Found");
            continue;
        }
        let mut command_string = input.split(' ');

        let command_input = CommandInput {
            command: CommandTypes::parse(command_string.next().unwrap()),
            args: command_string.map(String::from).collect()
        };
 
        
        let action = match command_input.command {

            CommandTypes::Exit => ShellAction::Exit,
            CommandTypes::Echo =>  echo(&command_input.args),
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

fn echo(message: &Vec<String> )-> ShellAction{

    println!("{}", message.join(" "));
    ShellAction::Continue
}