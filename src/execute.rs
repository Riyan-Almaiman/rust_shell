use std::fs::File;
use std::io;
use std::process::{Child, Command, Stdio};
use os_pipe::pipe;
use crate::builtin::{change_directories, echo, exit, print_current_dir, type_command};
use crate::command_input::{BuiltInCommand, Cmd, CommandType};
use crate::shell::{Shell, ShellAction};
use crate::utils::write_to_dest;

enum PipeReaderKind {
    Pipe(os_pipe::PipeReader),
    Child(std::process::ChildStdout),
}

impl Cmd {


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

                    let mut stdout: Box<dyn io::Write> = if let Some(redir) = &cmd.redirect_std_out
                    {
                        let file = redir
                            .options
                            .open(redir.filename.as_ref().unwrap())
                            .unwrap();
                        Box::new(file)
                    } else {
                        Box::new(io::stdout())
                    };

                    if last {
                        return cmd.execute_builtin(shell, &mut *stdin, &mut *stdout, &mut *stderr);
                    }

                    let (reader, writer) = pipe().unwrap();
                    let mut writer = writer;

                    cmd.execute_builtin(shell, &mut *stdin, &mut writer, &mut *stderr);

                    drop(writer);

                    previous_reader = Some(PipeReaderKind::Pipe(reader));
                }

                CommandType::External { path, args, name } => {
                    let mut command = Command::new(name);
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
                        let file = redir
                            .options
                            .open(redir.filename.as_ref().unwrap())
                            .unwrap();
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
            CommandType::Builtin(builtin) => return match builtin {
                BuiltInCommand::Exit => exit(),

                BuiltInCommand::PWD => {
                    print_current_dir(shell, output)
                }

                BuiltInCommand::CD(args) => {
                    let path = args.get(0).map(|s| s.as_str()).unwrap_or("~");
                    change_directories(shell, path, Some(output), error)
                }

                BuiltInCommand::Echo(args) => {
                    echo(args, output)
                }
                BuiltInCommand::Type(args) => {
                    type_command(shell, args, output)
                }
                BuiltInCommand::History => { for (index, entry) in shell.read_line.history().iter().enumerate() {
                    write_to_dest(output, format!("   {}  {}", index, entry).as_str());
                }
                    ShellAction::Continue

                }
            },

            _ => ShellAction::Continue,
        }
    }

    


}