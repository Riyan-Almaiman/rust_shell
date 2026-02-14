use is_executable::is_executable;
use std::{iter::Peekable, path::PathBuf, str::CharIndices, vec};

use crate::{
    CommandType, Shell,
    command_input::{Cmd, RedirectionType},
};

pub fn get_exe_command(command: &str) -> PathBuf {
    if cfg!(target_os = "windows") && !command.ends_with(".exe") {
        PathBuf::from(format!("{}.exe", command))
    } else {
        PathBuf::from(command)
    }
}

pub fn parse_commandtype_from_cmd(cmd: &mut Cmd, shell: &Shell) {
    cmd.command_type = match cmd.command_str.as_str() {
        "exit" => CommandType::Exit,
        "echo" => CommandType::Echo,
        "type" => CommandType::Type,
        "pwd" => CommandType::PWD,
        "cd" => CommandType::CD,
        _ => {
            let exe_name = get_exe_command(&cmd.command_str);
            let executable = shell.executables.iter().find(|exe| exe.name == exe_name);
            match executable {
                Some(exe) => {
                    if is_executable(&exe.path) {
                        cmd.path = Some(exe.path.clone());
                        CommandType::Exec
                    } else {
                        CommandType::Unknown
                    }
                }
                None => CommandType::Unknown,
            }
        }
    };
}

fn parse_redirections(cmd: &mut Cmd) {
    while let Some(index) = cmd.args.iter().position(|t| {
        t == ">" || t == "1>" || t == ">>" || t == "1>>" || t == "2>" || t == "2>>" 
    }) {
        let mut filename = None;
            let operator = cmd.args.remove(index);
            match index + 1 <= cmd.args.len() {
                true => filename = Some(cmd.args.remove(index)),
                false => (),
            }
            match operator.as_str() {
                ">" | "1>" => {
                    cmd.redirect_std_out = Some(RedirectionType::File {
                        filename,
                        overwrite: true,
                    });
                }
                ">>" | "1>>" => {
                    cmd.redirect_std_out = Some(RedirectionType::File {
                        filename,
                        overwrite: false,
                    });
                }
                "2>" => {
                    cmd.redirect_std_error = Some(RedirectionType::File {
                        filename,
                        overwrite: true,
                    });
                }
                "2>>" => {
                    cmd.redirect_std_error = Some(RedirectionType::File {
                        filename,
                        overwrite: false,
                    });
                }
                _ => {}
            }
        }
    }

pub fn parse_commands(cmd_tokens: &mut Vec<Vec<String>>, shell: &Shell) -> Option<Cmd> {
    if cmd_tokens.is_empty() {
        return None;
    }

    let mut current_cmd: Option<Cmd> = None;

    while let Some(mut tokens) = cmd_tokens.pop() {
        if tokens.is_empty() { continue; }

        let command_str = tokens.remove(0);
        
        let mut cmd = Cmd {
            command_type: CommandType::Unknown,
            path: None,
            command_str,
            args: tokens,
            redirect_std_out: current_cmd.map(|c| RedirectionType::NextCmd {
                cmd: Some(Box::new(c)),
            }),
            redirect_std_error: None,
        };

        parse_redirections(&mut cmd);
        parse_commandtype_from_cmd(&mut cmd, shell);

        current_cmd = Some(cmd);
    }

    current_cmd
}
pub fn split_by_delimiter<T: PartialEq + Clone>(vector: Vec<T>, delimiter: T) -> Vec<Vec<T>> {
    let mut result = Vec::new();
    let mut current = Vec::new();

    for item in vector.iter() {
        if *item == delimiter {
            if !current.is_empty() {
                result.push(current);
                current = Vec::new();
            }
        } else {
            current.push(item.clone());
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
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
fn add_to_token(token: &mut Option<String>, value: char) {
    token.get_or_insert_with(String::new).push(value);
}
fn push_token(token: &mut Option<String>, tokens: &mut Vec<String>) {
    if let Some(t) = token.take() {
        tokens.push(t);
    }
}
fn parse_delimiter(iter: &mut Peekable<CharIndices>, delimiter: char) -> Option<String> {
    let mut token: Option<String> = None;
    let is_double_quote = delimiter == '"';
    let escaped_chars = vec!['"', '\\', '$', '`', '\n'];

    while let Some((_, c)) = iter.next() {
        if is_double_quote && c == '\\' {
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

pub fn parse_input(input: &str) -> Vec<String> {
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
                    Some(_tok) => {
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
