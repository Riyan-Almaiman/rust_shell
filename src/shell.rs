use std::{env, path::PathBuf};
use std::path::Path;
use is_executable::is_executable;
use rustyline::{config::Configurer, history::FileHistory, CompletionType, Editor};

use crate::completion_helper::MyHelper;


pub struct Shell {
    pub executables: Vec<Executable>,
    pub path: String,
    pub read_line: Editor<MyHelper, FileHistory>,
    pub current_dir: PathBuf,
    pub prompt: String,
    pub builtins: Vec<String>,
    pub last_written_index: usize,
    pub history_file: PathBuf

}
pub struct Executable {
    pub name: String,
    pub path: PathBuf,
}
pub enum ShellAction {
    Continue,
    Error(String),
    Exit,
}
impl Shell {
    pub fn new(path: &str, prompt: &str, builtins: Vec<String>, history_env_key: &str) -> Self {
        let history_file = PathBuf::from(env::var(history_env_key).unwrap_or_default());
        let mut shell = Shell {
            executables: Vec::new(),
            path: path.to_string(),
            prompt: prompt.to_string(),
            read_line: Editor::<MyHelper, FileHistory>::new().unwrap(),
            current_dir: env::current_dir().unwrap(),
            builtins,
            last_written_index: 0,
            history_file

        };
        if shell.history_file.exists()   {
            let _ = shell.read_line.load_history(shell.history_file.as_path());
        }
        shell.read_line.set_helper(Some(MyHelper::new()));
        shell.get_executables();
        shell
    }

    fn get_executables(&mut self) {
        
        if !self.executables.is_empty() {
            return;
        }
        if let Ok(path_var) = env::var(&self.path) {
            for path in env::split_paths(&path_var) {
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let filename = entry.file_name().to_string_lossy().to_string();

                        if is_executable(&entry.path()) {
                            self.executables.push(Executable {
                                name: filename,
                                path: entry.path(),
                            });
                        }
                    }
                }
            }
        }
    }
}
