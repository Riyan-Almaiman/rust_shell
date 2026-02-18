use std::{iter::Peekable, str::CharIndices, vec};

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
