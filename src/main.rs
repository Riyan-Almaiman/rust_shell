use std::{
    cell::Cell, env::{self}, ffi::OsString, io::Stdout, path::PathBuf, process::Stdio
};
mod command_input;
mod parser;
#[allow(unused_imports)]
use std::io::{self, Write};
use rustyline::{At, Cmd, CompletionType, ConditionalEventHandler, Context, Event, EventContext, EventHandler, Helper, KeyEvent, Movement, RepeatCount, Word, completion::Completer, config::Configurer, error::ReadlineError, highlight::Highlighter, hint::Hinter, history::FileHistory, validate::Validator};
use rustyline::{DefaultEditor};
enum ShellAction {
    Continue,
    Error(String),
    Exit,
}
use command_input::CommandInput;
mod builtin;
#[derive(PartialEq, Debug)]
enum CommandType {
    Exit,
    Echo,
    Type,
    Exec { command: PathBuf, path: OsString },
    PWD,
    CD,
    Unknown,
}struct MyHelper {
    last_tab_pos: Cell<Option<usize>>,
}
impl MyHelper {
    fn new() -> Self {
        Self {
            last_tab_pos: Cell::new(None),
        }
    }
}
impl Completer for MyHelper {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<String>)> {
        let builtin = ["cd ", "echo ", "exit ", "pwd ", "type "];
        let prefix = &line[..pos];
        let word_start = prefix.rfind(' ').map(|i| i + 1).unwrap_or(0);
        let last_word = &prefix[word_start..];

        // 1. Start with matches from your built-ins
        let mut matches: Vec<String> = builtin
            .iter()
            .filter(|cmd| cmd.starts_with(last_word))
            .map(|s| s.to_string())
            .collect();

        // 2. Scan the system PATH for matching binaries
        if let Ok(path_var) = std::env::var("PATH") {
            for path in std::env::split_paths(&path_var) {
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let filename = entry.file_name().to_string_lossy().to_string();
                        if filename.starts_with(last_word) {
                            // Check if it's executable (optional but recommended)
                            matches.push(filename);
                        }
                    }
                }
            }
        }

        // 3. Clean up: Sort and remove duplicates (different PATH folders might have the same binary)
        matches.sort();
        matches.dedup();

        Ok((word_start, matches))
    }
}impl Hinter for MyHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for MyHelper {}

impl Validator for MyHelper {}

// This links them all together
impl Helper for MyHelper {}
struct TabEventHandler;
impl ConditionalEventHandler for TabEventHandler {
    fn handle(&self, evt: &Event, _n: RepeatCount, _: bool, ctx: &EventContext) -> Option<Cmd> {
        let builtin = ["cd ", "echo ", "exit ", "pwd ", "type "];
        let line = ctx.line();
        let pos = ctx.pos();
        // 1. Get the slice of the line before the cursor
        let prefix = &line[..pos];
        
        // 2. Find the start of the word being completed
        let word_start = prefix.rfind(' ').map(|i| i + 1).unwrap_or(0);
        let last_word = &prefix[word_start..];

        if last_word.is_empty() && pos > 0 {
            return None; // Don't complete on empty space
        }

        // 3. Find matches
        let suggestions: Vec<&str> = builtin
            .iter()
            .filter(|cmd| cmd.starts_with(last_word))
            .cloned()
            .collect();

        if let Some(suggestion) = suggestions.first() {
            // Calculate how many characters we need to add
            // We want to replace 'last_word' with 'suggestion'
            let chars_to_add = &suggestion[last_word.len()..];
            // Return Cmd::Replace to update the buffer cleanly
            // This replaces the word fragment with the full command
            let new_line = format!("{}{}{}", &line[..word_start], suggestion, &line[pos..]);
            let new_pos = word_start + suggestion.len();
            io::stdout().flush().unwrap();
        // Replace the current word with "Hello"
        let cmd = Cmd::Replace(
            Movement::ForwardWord(
                1,               // RepeatCount: just one word
                At::Start,       // At: move to the start of the next word
                Word::Big,       // Word: usually 'Big' (space-separated) or 'Emacs'
            ),
            Some("Hello".to_string())
        );

return Some(Cmd::Replace(
        Movement::ForwardWord(0, At::AfterEnd, Word::Big), // Adjust range to cover the fragment
        Some(suggestion.to_string())
    ));        }

        None
    }
}
fn main() {
    let key = "PATH";
    let paths: Vec<PathBuf> =
        env::split_paths(&std::env::var_os(key).unwrap_or(OsString::from(""))).collect();
let mut rl: rustyline::Editor<MyHelper, FileHistory> = rustyline::Editor::new().unwrap();
rl.set_helper(Some(MyHelper::new()));

rl.set_completion_type(CompletionType::Circular);
        loop {
        let mut input = String::new();

        // print!("$ ");
        // io::stdout().flush().unwrap();

        // io::stdin().read_line(&mut input).unwrap();
        match rl.readline("$ ") {
            Ok(line) => {
                input = line;
            }

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
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let command_input = CommandInput::new(input.to_string(), &paths);

        let action = match command_input.command_type {
            CommandType::Exit => ShellAction::Exit,
            CommandType::Type => command_input.type_command(&paths),
            CommandType::Echo => command_input.echo(),
            CommandType::Exec { .. } => command_input.execute(),
            CommandType::PWD => print_working_directory(),
            CommandType::CD => change_directories(&command_input.args.join(" ")),
            CommandType::Unknown => command_input.command_not_found(),
        };

        match action {

            ShellAction::Continue => {
                continue;
            }
            ShellAction::Exit => break,
            ShellAction::Error(msg) => {
                writeln!(&mut io::stderr(), "{}", msg).unwrap();
                continue;
            }
        }
    }
}

fn change_directories(path: &str) -> ShellAction {
   if path == "~" {
        let home = env::home_dir();
        match home {
            Some(p) => {set_dir(&p);
            return ShellAction::Continue},
            None =>  {
                return ShellAction::Continue;
            }

        }
    
    };
     set_dir(&PathBuf::new().join(&path));
    return ShellAction::Continue;
}
fn set_dir(path: &PathBuf){

   match env::set_current_dir(path) {
        Ok(_) => (),
        Err(_) => {
            println!("cd: {}: No such file or directory", path.display());
        }
    };
}
fn print_working_directory() -> ShellAction {
    println!("{}", env::current_dir().unwrap().display());
    ShellAction::Continue
}
