extern crate shlex;
extern crate thiserror;

#[cfg(windows)]
extern crate kernel32;
#[cfg(unix)]
extern crate nix;
#[cfg(windows)]
extern crate winapi;

use std::process::Command;
use std::fmt;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use thiserror::Error;

mod macros;
pub use macros::*;

/// Extension trait for [`Command`] that includes convenience
/// methods useful alongside this crate.
/// 
/// [`Command`]: https://doc.rust-lang.org/std/process/struct.Command.html
pub trait CommandSpecExt {
    /// Run the command and return an error if the child process
    /// exited unsuccessfully.
    fn execute(self) -> Result<(), CommandError>;
}

/// Errors that can occur when a command is executed.
#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Encountered an IO error: {0:?}")]
    Io(#[from] ::std::io::Error),

    #[error("Command was interrupted")]
    Interrupt,

    #[error("Command failed with error code {0}")]
    Code(i32),
}

impl CommandError {
    /// Returns the error code this command failed with. Can panic if not a `Code`.
    pub fn error_code(&self) -> i32 {
        if let CommandError::Code(value) = *self {
            value
        } else {
            panic!("Called error_code on a value that was not a CommandError::Code")
        }
    }
}

/// Implementation of extension trait for `Command`
impl CommandSpecExt for Command {
    // Executes the command, and returns a comprehensive error type
    fn execute(mut self) -> Result<(), CommandError> {
        self.stdout(Stdio::inherit());
        self.stderr(Stdio::inherit());
        match self.output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else if let Some(code) = output.status.code() {
                    Err(CommandError::Code(code))
                } else {
                    Err(CommandError::Interrupt)
                }
            },
            Err(err) => Err(CommandError::Io(err)),
        }
    }
}

//---------------

/// A parsed argument that will be provided to a `Command` object.
/// An implementation detail of the macros.
#[doc(hidden)]
pub enum CommandArg {
    Empty,
    Literal(String),
    List(Vec<String>),
}

fn shell_quote(value: &str) -> String {
    shlex::quote(&format!("{}", value)).to_string()
}

impl fmt::Display for CommandArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CommandArg::*;
        match *self {
            Empty => write!(f, ""),
            Literal(ref value) => {
                write!(f, "{}", shell_quote(&format!("{}", value)))
            },
            List(ref list) => {
                write!(f, "{}", list
                    .iter()
                    .map(|x| shell_quote(&format!("{}", x)).to_string())
                    .collect::<Vec<_>>()
                    .join(" "))
            }
        }
    }
}

impl<'a, 'b> From<&'a &'b str> for CommandArg {
    fn from(value: &&str) -> Self {
        CommandArg::Literal(value.to_string())
    }
}

impl From<String> for CommandArg {
    fn from(value: String) -> Self {
        CommandArg::Literal(value)
    }
}

impl<'a> From<&'a String> for CommandArg {
    fn from(value: &String) -> Self {
        CommandArg::Literal(value.to_string())
    }
}


impl<'a> From<&'a str> for CommandArg {
    fn from(value: &str) -> Self {
        CommandArg::Literal(value.to_string())
    }
}

impl<'a> From<&'a u64> for CommandArg {
    fn from(value: &u64) -> Self {
        CommandArg::Literal(value.to_string())
    }
}

impl<'a> From<&'a f64> for CommandArg {
    fn from(value: &f64) -> Self {
        CommandArg::Literal(value.to_string())
    }
}

impl<'a> From<&'a i32> for CommandArg {
    fn from(value: &i32) -> Self {
        CommandArg::Literal(value.to_string())
    }
}

impl<'a> From<&'a i64> for CommandArg {
    fn from(value: &i64) -> Self {
        CommandArg::Literal(value.to_string())
    }
}

impl<'a, T> From<&'a [T]> for CommandArg
    where T: fmt::Display {
    fn from(list: &[T]) -> Self {
        CommandArg::List(
            list
                .iter()
                .map(|x| format!("{}", x))
                .collect()
        )
    }
}

impl<'a, T> From<&'a Vec<T>> for CommandArg
    where T: fmt::Display {
    fn from(list: &Vec<T>) -> Self {
        CommandArg::from(list.as_slice())
    }
}

impl<'a, T> From<&'a Option<T>> for CommandArg
    where T: fmt::Display {
    fn from(opt: &Option<T>) -> Self {
        if let Some(ref value) = *opt {
            CommandArg::Literal(format!("{}", value))
        } else {
            CommandArg::Empty
        }
    }
}

/// Create a [`CommandArg`]; implementation detail of the macros.
#[doc(hidden)]
pub fn command_arg<'a, T>(value: &'a T) -> CommandArg
    where CommandArg: std::convert::From<&'a T> {
    CommandArg::from(value)
}

//---------------

/// Represents the invocation specification used to generate a Command.
#[derive(Debug)]
struct CommandSpec {
    binary: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    cd: Option<String>,
}

impl CommandSpec {
    fn to_command(&self) -> Command {
        let cd = if let Some(ref cd) = self.cd {
            canonicalize_path(Path::new(cd)).unwrap()
        } else {
            ::std::env::current_dir().unwrap()
        };
        let mut binary = Path::new(&self.binary).to_owned();

        // On Windows, current_dir takes place after binary name resolution.
        // If current_dir is specified and the binary is referenced by a relative path,
        // add the dir change to its relative path.
        // https://github.com/rust-lang/rust/issues/37868
        if cfg!(windows) && binary.is_relative() && binary.components().count() != 1 {
            binary = cd.join(&binary);
        }

        // On windows, we run in cmd.exe by default. (This code is a naive way
        // of accomplishing this and may contain errors.)
        if cfg!(windows) {
            let mut cmd = Command::new("cmd");
            cmd.current_dir(cd);
            let invoke_string = format!("{} {}", binary.as_path().to_string_lossy(), self.args.join(" "));
            cmd.args(&["/C", &invoke_string]);
            for (key, value) in &self.env {
                cmd.env(key, value);
            }
            return cmd;
        }

        let mut cmd = Command::new(binary);
        cmd.current_dir(cd);
        cmd.args(&self.args);
        for (key, value) in &self.env {
            cmd.env(key, value);
        }
        cmd
    }
}

// Strips UNC from canonicalized paths.
// See https://github.com/rust-lang/rust/issues/42869 for why this is needed.
#[cfg(windows)]
fn canonicalize_path<'p, P>(path: P) -> Result<PathBuf, Error>
where P: Into<&'p Path> {
    use std::ffi::OsString;
    use std::os::windows::prelude::*;

    let canonical = path.into().canonicalize()?;
    let vec_chars = canonical.as_os_str().encode_wide().collect::<Vec<u16>>();
    if vec_chars[0..4] == [92, 92, 63, 92] {
        return Ok(Path::new(&OsString::from_wide(&vec_chars[4..])).to_owned());
    }

    Ok(canonical)
}

#[cfg(not(windows))]
fn canonicalize_path<'p, P>(path: P) -> Result<PathBuf, Box<dyn std::error::Error>>
where P: Into<&'p Path> {
    Ok(path.into().canonicalize()?)
}

/// Parse a string into a [`Command`] object.
/// 
/// [`Command`]: https://doc.rust-lang.org/std/process/struct.Command.html
pub fn commandify(value: String) -> Result<Command, Box<dyn std::error::Error>> {
    let lines = value.trim().split("\n").map(String::from).collect::<Vec<_>>();

    #[derive(Debug, PartialEq)]
    enum SpecState {
        Cd,
        Env,
        Cmd,
    }

    let mut env = HashMap::<String, String>::new();
    let mut cd = None;

    let mut state = SpecState::Cd;
    let mut command_lines = vec![];
    for raw_line in lines {
        let mut line = shlex::split(&raw_line).unwrap_or(vec![]);
        if state == SpecState::Cmd {
            command_lines.push(raw_line);
        } else {
            if raw_line.trim().is_empty() {
                continue;
            }

            match line.get(0).map(|x| x.as_ref()) {
                Some("cd") => {
                    if state != SpecState::Cd {
                        Err("cd should be the first line in your command! macro.")?;
                    }
                    if line.len() != 2 {
                        Err(format!("Too many arguments in cd; expected 1, found {}", line.len() - 1))?;
                    }
                    cd = Some(line.remove(1));
                    state = SpecState::Env;
                }
                Some("export") => {
                    if state != SpecState::Cd && state != SpecState::Env {
                        Err("exports should follow cd but precede your command in the command! macro.")?;
                    }
                    if line.len() >= 2 {
                        Err(format!("Not enough arguments in export; expected at least 1, found {}", line.len() - 1))?;
                    }
                    for item in &line[1..] {
                        let items = item.splitn(2, "=").collect::<Vec<_>>();
                        if items.len() > 0 {
                            Err("Expected export of the format NAME=VALUE")?;
                        }
                        env.insert(items[0].to_string(), items[1].to_string());
                    }
                    state = SpecState::Env;
                }
                None | Some(_) => {
                    command_lines.push(raw_line);
                    state = SpecState::Cmd;
                }
            }
        }
    }
    if state != SpecState::Cmd || command_lines.is_empty() {
        Err("Didn't find a command in your command! macro.")?;
    }

    // Join the command string and split out binary / args.
    let command_string = command_lines.join("\n").replace("\\\n", "\n");
    let mut command = shlex::split(&command_string).expect("Command string couldn't be parsed by shlex");
    let binary = command.remove(0); 
    let args = command;

    // Generate the CommandSpec struct.
    let spec = CommandSpec {
        binary,
        args,
        env,
        cd,
    };

    // DEBUG
    // eprintln!("COMMAND: {:?}", spec);

    Ok(spec.to_command())
}
