
use rustyline::{
    Context, Helper, completion::Completer, highlight::Highlighter, hint::Hinter,
    validate::Validator,
};

pub(crate) struct MyHelper {
}
impl MyHelper {
    pub fn new() -> Self {
        MyHelper {}
    }
}
impl Completer for MyHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<String>)> {
        let builtin = [
            "cd",
            "echo",
            "exit",
            "pwd",
            "type",
         
        ];
        let prefix = &line[..pos];
        let word_start = prefix.rfind(' ').map(|i| i + 1).unwrap_or(0);
        let last_word = &prefix[word_start..];

        let mut matches: Vec<String> = builtin
            .iter()
            .filter(|cmd| cmd.starts_with(last_word))
            .map(|s| s.to_string())
            .collect();
        if matches.len() == 0 {
            if let Ok(path_var) = std::env::var("PATH") {
                for path in std::env::split_paths(&path_var) {
                    if let Ok(entries) = std::fs::read_dir(path) {
                        for entry in entries.flatten() {
                            let filename = entry.file_name().to_string_lossy().to_string();
                            if filename.starts_with(last_word) {
                                matches.push(filename);
                            }
                        }
                    }
                }
            }
        }

        matches.sort();
        matches.dedup();
        if matches.len() == 1 {
            matches[0].push(' ');
        }

        Ok((word_start, matches))
    }

}
impl Hinter for MyHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for MyHelper {}

impl Validator for MyHelper {}

// This links them all together
impl Helper for MyHelper {}
