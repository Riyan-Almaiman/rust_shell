use std::{env, io::Write, path::PathBuf};
use std::fs::{File, OpenOptions};
use std::io::Read;
use rustyline::history::History;
use crate::{Shell, ShellAction};

use crate::utils::write_to_dest;

pub fn exit() -> ShellAction {
    ShellAction::Exit
}

pub fn print_current_dir(shell: &mut Shell, dest: &mut dyn Write) -> ShellAction {
    write_to_dest(dest, format!("{}", shell.current_dir.display()).as_str());
    ShellAction::Continue
}
pub fn set_current_dir(shell: &mut Shell, path: &PathBuf, dest_err: &mut dyn Write) {
    match env::set_current_dir(path) {
        Ok(_) => shell.current_dir = env::current_dir().unwrap(),
        Err(_) => {
            write_to_dest(
                dest_err,
                format!("cd: {}: No such file or directory", path.display()).as_str(),
            );
        }
    };
}
pub fn change_directories(
    shell: &mut Shell,
    path: &str,
    dest_out: Option<&mut dyn Write>,
    dest_err: &mut dyn Write,
) -> ShellAction {
    if path == "~" {
        let home = env::home_dir();
        return match home {
            Some(p) => {
                set_current_dir(shell, &p, dest_err);
                ShellAction::Continue
            }
            None => {
                ShellAction::Continue
            }
        }
    };
    set_current_dir(shell, &PathBuf::from(&path), dest_err);
    return ShellAction::Continue;
}
pub fn echo(args: &[String], dest: &mut dyn Write) -> ShellAction {
    let content = args.join(" ");

    write_to_dest(dest, &content);

    ShellAction::Continue
}

pub fn type_command(shell: &Shell, args: &Vec<String>, dest: &mut dyn Write) -> ShellAction {
    if args.is_empty() {
        return ShellAction::Continue;
    }

    let cmd_name = &args[0];

    if shell.builtins.contains(cmd_name) {
        write_to_dest(dest, format!("{} is a shell builtin", cmd_name).as_str());
        return ShellAction::Continue;
    }

    if let Some(exe) = shell.executables.iter().find(|e| e.name == *cmd_name) {
        write_to_dest(
            dest,
            format!("{} is {}", cmd_name, exe.path.display()).as_str(),
        );
        return ShellAction::Continue;
    }

    write_to_dest(dest, format!("{}: not found", cmd_name).as_str());

    ShellAction::Continue
}
pub fn history(shell: &mut Shell, args: &Vec<String>, output: &mut dyn Write, error: &mut dyn Write) -> ShellAction {

    let first_arg = args.get(0).map(|s| s.as_str()).unwrap_or("0");
    let second_arg = args.get(1).map(|s| s.as_str()).unwrap_or("");
    match first_arg {
        "-r" => {
            if second_arg.is_empty() {
                write_to_dest(error, "history: missing file operand");
                return ShellAction::Continue;
            }
            let path = PathBuf::from(second_arg);
            if let Err(e) = shell.read_line.load_history(&path) {
                write_to_dest(error, &format!("history: {}", e));
            }
            return ShellAction::Continue;
        }
        "-w" => {
            if second_arg.is_empty() {
                write_to_dest(error, "history: missing file operand");
                return ShellAction::Continue;
            }

            let path = PathBuf::from(second_arg);

            match File::create(&path) {
                Ok(mut file) => {
                    for entry in shell.read_line.history().iter() {
                        if let Err(e) = writeln!(file, "{}", entry) {
                            write_to_dest(error, &format!("history: {}", e));
                            return ShellAction::Continue;
                        }
                    }
                }
                Err(e) => {
                    write_to_dest(error, &format!("history: {}", e));
                }
            }
            return ShellAction::Continue;
        }
        "-a" => {
            if second_arg.is_empty() {
                write_to_dest(error, "history: missing file operand");
                return ShellAction::Continue;
            }

            let path = PathBuf::from(second_arg);

            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            {
                Ok(mut file) => {
                    let history = shell.read_line.history();

                    for entry in history.iter().skip(shell.last_written_index) {
                        if let Err(e) = writeln!(file, "{}", entry) {
                            write_to_dest(error, &format!("history: {}", e));
                            return ShellAction::Continue;
                        }
                    }

                    shell.last_written_index = history.len();
                }
                Err(e) => {
                    write_to_dest(error, &format!("history: {}", e));
                }
            }

            return ShellAction::Continue;
        }

        _ => (),
    }
    let n = first_arg.parse().unwrap_or(0);
    let history = shell.read_line.history();
    let len = history.len();
    let start = match n {
        0 => 0,
        n => len.saturating_sub(n),
    };
    for (i, entry) in shell.read_line.history().iter().enumerate() {
        if i >= start {
            write_to_dest(output, &format!("{:>5}  {}", i + 1, entry));
        }
    }

    ShellAction::Continue

}

