use is_executable::is_executable;
use std::{iter::Peekable, path::PathBuf, str::CharIndices};

use crate::{CommandType, command_input::CommandInput};

pub fn get_exe_command(command: &str) -> PathBuf {
    if cfg!(target_os = "windows") && !command.ends_with(".exe") {
        PathBuf::from(format!("{}.exe", command))
    } else {
        PathBuf::from(command)
    }
}

pub fn parse_commandtype_from_cmd_str(command_input: &mut CommandInput, paths: &[PathBuf]) {
    command_input.command_type = match command_input.command_str.as_str() {
        "exit" => CommandType::Exit,
        "echo" => CommandType::Echo,
        "type" => CommandType::Type,
        "pwd" => CommandType::PWD,
        "cd" => CommandType::CD,
        _ => parse_unknown(&command_input.command_str, paths),
    };
}
pub(crate) fn parse_unknown(command: &str, paths: &[PathBuf]) -> CommandType {
    let exe_name = get_exe_command(command);
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

pub fn parse_redirections(input: &mut CommandInput) {
    while let Some(index) = input
        .tokens
        .iter()
        .position(|t| t == ">" || t == "1>" || t == ">>" || t == "1>>" || t == "2>" || t == "2>>")
    {
        if index + 1 < input.tokens.len() {
            let operator = input.tokens.remove(index);
            let filename = input.tokens.remove(index);

            match operator.as_str() {
                ">" | "1>" => {
                    input.redirect_std_out = Some(filename);
                    input.overwrite_std_out_redirect = true;
                }
                ">>" | "1>>" => {
                    input.redirect_std_out = Some(filename);
                    input.overwrite_std_out_redirect = false;
                }
                "2>" => {
                    input.redirect_std_error = Some(filename);
                    input.overwrite_std_err_file = true;
                }
                "2>>" => {
                    input.redirect_std_error = Some(filename);
                    input.overwrite_std_err_file = false;
                }
                _ => {}
            }
        } else {
            input.tokens.remove(index);
            break;
        }
    }
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
