use std::{env, io::Write, path::PathBuf};

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
