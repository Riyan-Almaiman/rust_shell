use crate::{CommandType, ShellAction};
use is_executable::is_executable;
use std::fs::File;
use std::iter::Peekable;
use std::str::CharIndices;
use std::{
    env,
    path::{self, PathBuf},
    process,
};
use std::io::Write;
use std::process::Command;

#[derive(Debug)]
pub struct CommandInput<'a> {
    pub command_type: CommandType,
    pub command_str: String,
    pub args: Vec<String>,
    pub redirect_file: Option<String>,
    pub paths: &'a [PathBuf],
}

fn parse_escape(
    iter: &mut Peekable<CharIndices>,
    token: &mut Option<String>,
    escaped_chars: &Vec<char>,
) {
    if let Some(&(_, next_c)) = iter.peek() {
        if escaped_chars.contains(&next_c) || escaped_chars.is_empty() {
            add_to_token(token, next_c);
            iter.next();
        } else {
            add_to_token(token, '\\');
        }
    }
}
fn parse_delimiter(iter: &mut Peekable<CharIndices>, delimiter: char) -> Option<String> {
    let mut token: Option<String> = None;
    let is_double_quote = delimiter == '"';
    let escaped_chars = vec!['"', '\\', '$', '`', '\n'];

    while let Some((_, c)) = iter.next() {
        if (is_double_quote && c == '\\') {
            parse_escape(iter, &mut token, &escaped_chars);
            continue;
        }
        if c == delimiter {
            if let Some(&(_, next_c)) = iter.peek() {
                if next_c == delimiter {
                    iter.next();
                    continue;
                }
            }
            if token.is_some() {
                return token;
            }

            return None;
        }
        add_to_token(&mut token, c);
    }
    None
}
fn add_to_token(token: &mut Option<String>, value: char) {
    token.get_or_insert_with(String::new).push(value);
}
fn push_token(token: &mut Option<String>, tokens: &mut Vec<String>) {
    if let Some(t) = token.take() {
        tokens.push(t);
    }
}
fn parse_input(input: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut token: Option<String> = None;

    let token_delimiters = ['"', '\''];
    let mut iter = input.char_indices().peekable();
    while let Some((_, c)) = iter.next() {
        if c == '\\' {
            parse_escape(&mut iter, &mut token, &vec![]);
            continue;
        }
        if token_delimiters.contains(&c) {
            match parse_delimiter(&mut iter, c) {
                Some(t) => match &token {
                    Some(tok) => {
                        token.get_or_insert_with(String::new).push_str(&t);
                        push_token(&mut token, &mut tokens);
                    }
                    None => {
                        token.get_or_insert_with(String::new).push_str(&t);
                    }
                },
                None => (),
            }

            continue;
        }
        if c == ' ' {
            push_token(&mut token, &mut tokens);
            continue;
        }
        add_to_token(&mut token, c);
    }
    push_token(&mut token, &mut tokens);

    tokens
}
impl<'a> CommandInput<'a> {
    pub fn new(input: String, paths: &'a [PathBuf]) -> Self {
        let mut tokens = parse_input(&input);
        let mut redirect_file = None;

        if let Some(index) = tokens.iter().position(|t| t == ">" || t == "1>") {
            if index + 1 < tokens.len() {
                redirect_file = Some(tokens.remove(index + 1));
                tokens.remove(index); //
            }
        }
        let parsed_command = tokens.get(0).cloned().unwrap_or_default();

        let parsed_args = tokens.get(1..).map_or(vec![], |s| s.to_vec());

        let cmd = match parsed_command.as_str() {
            "exit" => CommandType::Exit,
            "echo" => CommandType::Echo,
            "type" => CommandType::Type,
            "pwd" => CommandType::PWD,
            "cd" => CommandType::CD,
            _ => Self::parse_unknown(&parsed_command, paths),
        };
        Self {
            paths,
            command_type: cmd,
            command_str: parsed_command,
            args: parsed_args,
            redirect_file,
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
        let (cmd, path) = match &self.command_type {
            CommandType::Exec { command, path } => (command, path),
            _ => return ShellAction::Continue,
        };
        let process: &mut Command =  &mut process::Command::new(path);
        match self.redirect_file.as_ref() {
            Some(file_name) => {
                let file = File::create_new(file_name).unwrap();
                process
                    .args(&self.args)
                    .stdout(file)
                    .spawn()
                    .expect("failed to spawn process")
                    .wait()
                    .expect("failed to wait on process");
            }
            None => {
                process
                    .args(&self.args)
                    .spawn()
                    .expect("failed to spawn process")
                    .wait()
                    .expect("failed to wait on process");
            }
        };
        return ShellAction::Continue;
    }
    pub fn echo(&self) -> ShellAction {
        let mut args =  self.args.join(" ");
        args.push('\n');

        if self.redirect_file.is_some() {
             std::fs::write(self.redirect_file.as_ref().unwrap(), args).unwrap();
        }
        else{
            println!("{}", self.args.join(" "));

        }
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
