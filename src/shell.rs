use std::{env, path::PathBuf};

use is_executable::is_executable;
use rustyline::{CompletionType, Editor, config::Configurer, history::FileHistory};

use crate::completion_helper::MyHelper;

pub struct Shell {
    pub executables: Vec<Executable>,
    pub path: String,
    pub read_line: Editor<MyHelper, FileHistory>,
    pub current_dir: PathBuf,
    pub prompt: String,
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
    pub fn new(path: &str, prompt: &str) -> Self {
        let mut shell = Shell {
            executables: Vec::new(),
            path: path.to_string(),
            prompt: prompt.to_string(),
            read_line: Editor::<MyHelper, FileHistory>::new().unwrap(),
            current_dir: env::current_dir().unwrap(),
        };
        shell.read_line.set_helper(Some(MyHelper::new()));

        shell.read_line.set_completion_type(CompletionType::List);
        shell.get_executables();
        shell
    }
    pub fn print_current_dir(&self) -> ShellAction {
        println!("{}", self.current_dir.display());
        ShellAction::Continue
    }
    pub fn set_current_dir(&mut self, path: &PathBuf) {
        match env::set_current_dir(path) {
            Ok(_) => self.current_dir = env::current_dir().unwrap(),
            Err(_) => {
                println!("cd: {}: No such file or directory", path.display());
            }
        };
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
