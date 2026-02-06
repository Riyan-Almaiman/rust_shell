use crate::{CommandType, ShellAction};
use is_executable::is_executable;
use std::str::CharIndices;
use std::{
    env,
    path::{self, PathBuf},
};

#[derive(Debug)]
pub struct CommandInput<'a> {
    pub command_type: CommandType,
    pub command_str: String,
    pub args: Vec<String>,
    pub raw_args: String,
    pub paths: &'a [PathBuf],
}

fn parse_delimiter(iter: &mut CharIndices, delimiter: char, tokens: &mut Vec<String>) {
    let mut token = String::new();
    for (_, c) in iter.by_ref() {
        if c == delimiter {
            if (!token.is_empty()) {
                tokens.push(token);
            }
            return;
        }
        token.push(c);
    }
}
fn add_to_token(token: &mut Option<String>, value: char){
    token.get_or_insert(value.to_string()).push(value);

}
fn push_token(token: &mut Option<String> , tokens: &mut Vec<String>){
    if let Some(t) = token.take() {
        tokens.push(t);
    }
}
fn parse_input(input: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut token: Option<String> = None;

    let token_delimiters = ['"', '\''];
    let mut iter = input.char_indices();
    while let Some((_, c)) = iter.next() {
        if token_delimiters.contains(&c) {
            push_token(&mut token, &mut tokens);
            parse_delimiter(&mut iter, c, &mut tokens);
            continue;
        }
        if c == ' ' {
            push_token(&mut token, &mut tokens);
            continue;
        }
        token.get_or_insert_with(String::new).push(c);
    }
    push_token(&mut token, &mut tokens);
    tokens
}

impl<'a> CommandInput<'a> {
    pub fn new(input: String, paths: &'a [PathBuf]) -> Self {
        let (command, args) = input.split_once(' ').unwrap_or((&input, ""));
        let cmd = match command {
            "exit" => CommandType::Exit,
            "echo" => CommandType::Echo,
            "type" => CommandType::Type,
            "pwd" => CommandType::PWD,
            "cd" => CommandType::CD,
            _ => Self::parse_unknown(command, paths),
        };
        Self {
            paths,
            command_type: cmd,
            command_str: command.to_string(),
            args: parse_input(args),
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
            _ => return ShellAction::Continue,
        };

        let mut process = std::process::Command::new(cmd)
            .args(&self.args)
            .spawn()
            .expect("failed to spawn process");

        process.wait().expect("failed to wait for process");

        return ShellAction::Continue;
    }
    pub fn echo(&self) -> ShellAction {
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
