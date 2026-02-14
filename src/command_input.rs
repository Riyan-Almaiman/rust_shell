use crate::parser::{self, parse_input, split_by_delimiter};
use crate::{ Shell, ShellAction};

use std::fs::{File, OpenOptions};

use std::{path::PathBuf, process};
use std::ffi::CString;
use std::io::Write;
use os_pipe::pipe;
use std::io;

use std::process::{Child, Command, Stdio};
use crate::builtin::{change_directories, echo, exit, print_current_dir, type_command};

#[derive(Debug)]

pub enum BuiltIn{
     Exit,
     Echo(Vec<String>),
     Type(Vec<String>),
     CD(Vec<String>),
     PWD,
}
#[derive(Debug)]
pub enum CommandType {
    Builtin(BuiltIn),
    External {
        path: PathBuf,
        args: Vec<String>,
    },
    Unknown (String),
}
#[derive(Debug)]

pub struct Redirection {
    pub options: OpenOptions,
    pub filename: Option<String>,
}
impl Redirection {
    pub fn new(overwrite: bool, filename: Option<String>) -> Redirection {
        let  file_options = OpenOptions::new()
            .create(true)
            .write(true)
        .truncate(overwrite)
            .append(!overwrite).clone();

        Redirection { options: file_options, filename }
    }
}
#[derive(Debug)]
pub  struct Cmd {
    pub command_type: CommandType,
    pub command_str: String,
    pub redirect_std_out: Option<Redirection>,
    pub redirect_std_error: Option<Redirection>,
    pub child: Option<Box<Cmd>>,

}


enum PipeReaderKind {
    Pipe(os_pipe::PipeReader),
    Child(std::process::ChildStdout),
}

impl Cmd {
    pub fn new(input: String, shell: &Shell) -> Option<Self> {
        let tokens = parse_input(&input);
        let mut cmd_strs = split_by_delimiter(tokens.clone(), "|".to_string());

        let  command =  parser::parse_commands(&mut cmd_strs, shell);
        command
    }
    pub fn flatten(&self) -> Vec<&Cmd> {
        let mut cmds = Vec::new();
        let mut current = Some(self);

        while let Some(cmd) = current {
            cmds.push(cmd);
            current = cmd.child.as_deref();
        }

        cmds
    }
    pub fn execute(&self, shell: &mut Shell) -> ShellAction {
        let pipeline = self.flatten();

        let mut previous_reader: Option<PipeReaderKind> = None;
        let mut children: Vec<Child> = Vec::new();

        for (i, cmd) in pipeline.iter().enumerate() {
            let last = i == pipeline.len() - 1;

            match &cmd.command_type {

                // ---------------- BUILTIN ----------------
                CommandType::Builtin(_) => {

                    let mut stdin: Box<dyn io::Read> = match previous_reader.take() {
                        Some(PipeReaderKind::Pipe(r)) => Box::new(r),
                        Some(PipeReaderKind::Child(r)) => Box::new(r),
                        None => Box::new(io::stdin()),
                    };

                    let mut stderr: Box<dyn io::Write> =
                        if let Some(redir) = &cmd.redirect_std_error {
                            let file = redir
                                .options
                                .open(redir.filename.as_ref().unwrap())
                                .unwrap();
                            Box::new(file)
                        } else {
                            Box::new(io::stderr())
                        };

                    let mut stdout: Box<dyn io::Write> =
                        if let Some(redir) = &cmd.redirect_std_out {
                            let file = redir
                                .options
                                .open(redir.filename.as_ref().unwrap())
                                .unwrap();
                            Box::new(file)
                        } else {
                            Box::new(io::stdout())
                        };

                    if last {
                        return cmd.execute_builtin(
                            shell,
                            &mut *stdin,
                            &mut *stdout,
                            &mut *stderr,
                        );
                    }


                    let (reader, writer) = pipe().unwrap();
                    let mut writer = writer;

                    cmd.execute_builtin(
                        shell,
                        &mut *stdin,
                        &mut writer,
                        &mut *stderr,
                    );

                    drop(writer);

                    previous_reader = Some(PipeReaderKind::Pipe(reader));
                }

                CommandType::External { path, args } => {

                    let mut command = Command::new(path);
                    command.args(args);

                    if let Some(reader) = previous_reader.take() {
                        match reader {
                            PipeReaderKind::Pipe(r) => {
                                command.stdin(Stdio::from(r));
                            }
                            PipeReaderKind::Child(r) => {
                                command.stdin(Stdio::from(r));
                            }
                        }
                    }

                    let mut stdout_file: Option<File> = None;

                    if let Some(redir) = &cmd.redirect_std_out {
                        let file = redir
                            .options
                            .open(redir.filename.as_ref().unwrap())
                            .unwrap();

                        stdout_file = Some(file);
                    }

                    if let Some(file) = stdout_file {
                        command.stdout(Stdio::from(file));
                    } else if !last {
                        command.stdout(Stdio::piped());
                    }
                    if let Some(redir) = &cmd.redirect_std_error {
                        let file = redir.options.open(
                            redir.filename.as_ref().unwrap()
                        ).unwrap();
                        command.stderr(Stdio::from(file));
                    }


                    let mut child = command.spawn().unwrap();

                    if !last {
                        let stdout = child.stdout.take().unwrap();
                        previous_reader = Some(PipeReaderKind::Child(stdout));
                    }

                    children.push(child);
                }


                CommandType::Unknown(_) => {
                    return cmd.command_not_found();
                }
            }
        }

        for mut child in children {
            child.wait().unwrap();
        }

        ShellAction::Continue
    }

    pub fn execute_builtin(
        &self,
        shell: &mut Shell,
        input: &mut dyn io::Read,
        output: &mut dyn io::Write,
        error: &mut dyn io::Write,
    ) -> ShellAction {

        match &self.command_type {
            CommandType::Builtin(builtin) => match builtin {

                BuiltIn::Exit => return exit(),

                BuiltIn::PWD => {
                    return print_current_dir(shell, output);
                }

                BuiltIn::CD(args) => {
                    let path = args.get(0).map(|s| s.as_str()).unwrap_or("~");
                    return change_directories(shell, path, Some(output), error);
                }

                BuiltIn::Echo(args) => {
                    return echo(args, output);
                }

                BuiltIn::Type(_) => {
                    return type_command(self, output);
                }
            },

            _ => ShellAction::Continue,
        }
    }



    pub fn command_not_found(&self) -> ShellAction {
        println!("{}: command not found", self.command_str);
        ShellAction::Continue
    }


 }
