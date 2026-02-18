mod command_input;
mod parser;
use rustyline::error::ReadlineError;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::io::{stderr, stdout};

mod shell;
use shell::Shell;

use crate::{command_input::Cmd, shell::ShellAction};
use crate::builtin::history;

mod builtin;

mod completion_helper;
mod execute;
mod utils;
mod redirection;

fn main() {
    let builtins: Vec<String> = vec![
        "exit".to_string(),
        "echo".to_string(),
        "type".to_string(),
        "cd".to_string(),
        "pwd".to_string(),
        "history".to_string(),
    ];
    let mut shell = Shell::new("PATH", "$ ", builtins, "HISTFILE");

    loop {
        let input = match shell.read_line.readline(&shell.prompt) {
            Ok(line) => line,

            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
        .trim()
        .to_string();

        if input.is_empty() {
            continue;
        }

        let command = Cmd::new(&input, &shell);
        shell.read_line.add_history_entry(input).unwrap();
        match command {
            None => continue,
            Some(cmd) => match cmd.execute(&mut shell) {
                ShellAction::Continue => continue,
                ShellAction::Exit => {
                    let file = shell.history_file.clone();
                    history(
                        &mut shell,
                        &vec!["-a".to_string(), file.to_str().unwrap_or_default().to_string()],
                        &mut stdout(),
                        &mut stderr()
                    );
                    break;
                },
                ShellAction::Error(msg) => {
                    writeln!(&mut io::stderr(), "{}", msg).unwrap();
                    continue;
                }
            },
        }
    }
}
