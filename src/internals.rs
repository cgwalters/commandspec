use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A parsed argument that will be provided to a `Command` object.
/// An implementation detail of the macros.
#[doc(hidden)]
pub enum CommandArg {
    Empty,
    Literal(String),
    Raw(String),
    List(Vec<String>),
}

fn shell_quote(value: &str) -> String {
    shlex::quote(value).to_string()
}

// https://wiki.bash-hackers.org/syntax/quoting#ansi_c_like_strings
fn bash_binary_quote(value: &[u8]) -> String {
    let mut r = Vec::new();
    r.extend("$'".as_bytes().iter());
    r.extend(value.iter().flat_map(|&c| std::ascii::escape_default(c)));
    r.extend("'".as_bytes().iter());
    String::from_utf8(r).expect("bash_binary quote should have output utf8")
}

impl fmt::Display for CommandArg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CommandArg::*;
        match *self {
            Empty => write!(f, ""),
            Literal(ref value) => write!(f, "{}", shell_quote(value)),
            Raw(ref value) => write!(f, "{}", value),
            List(ref list) => write!(
                f,
                "{}",
                list.iter()
                    .map(|x| shell_quote(x))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
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

impl<'a> From<&'a Path> for CommandArg {
    fn from(value: &Path) -> Self {
        use std::os::unix::ffi::OsStrExt;
        if let Some(s) = value.to_str() {
            CommandArg::Literal(s.to_string())
        } else {
            CommandArg::Raw(bash_binary_quote(value.as_os_str().as_bytes()))
        }
    }
}

impl<'a, 'b> From<&'a &'b Path> for CommandArg {
    fn from(value: &&Path) -> Self {
        CommandArg::from(*value)
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
where
    T: fmt::Display,
{
    fn from(list: &[T]) -> Self {
        CommandArg::List(list.iter().map(|x| format!("{}", x)).collect())
    }
}

impl<'a, T> From<&'a Vec<T>> for CommandArg
where
    T: fmt::Display,
{
    fn from(list: &Vec<T>) -> Self {
        CommandArg::from(list.as_slice())
    }
}

impl<'a, T> From<&'a Option<T>> for CommandArg
where
    T: fmt::Display,
{
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
where
    CommandArg: std::convert::From<&'a T>,
{
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
            let invoke_string = format!(
                "{} {}",
                binary.as_path().to_string_lossy(),
                self.args.join(" ")
            );
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
where
    P: Into<&'p Path>,
{
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
where
    P: Into<&'p Path>,
{
    Ok(path.into().canonicalize()?)
}

fn impl_commandify(value: &str) -> Result<Command, Box<dyn std::error::Error>> {
    let lines = value
        .trim()
        .split('\n')
        .map(String::from)
        .collect::<Vec<_>>();

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
        let mut line = shlex::split(&raw_line).unwrap_or_default();
        if state == SpecState::Cmd {
            command_lines.push(raw_line);
        } else {
            if raw_line.trim().is_empty() {
                continue;
            }

            match line.get(0).map(|x| x.as_ref()) {
                Some("cd") => {
                    if state != SpecState::Cd {
                        return Err("cd should be the first line in your command! macro.".into());
                    }
                    if line.len() != 2 {
                        return Err(format!(
                            "Too many arguments in cd; expected 1, found {}",
                            line.len() - 1
                        )
                        .into());
                    }
                    cd = Some(line.remove(1));
                    state = SpecState::Env;
                }
                Some("export") => {
                    if state != SpecState::Cd && state != SpecState::Env {
                        return Err("exports should follow cd but precede your command in the command! macro.".into());
                    }
                    if line.len() >= 2 {
                        return Err(format!(
                            "Not enough arguments in export; expected at least 1, found {}",
                            line.len() - 1
                        )
                        .into());
                    }
                    for item in &line[1..] {
                        let items = item.splitn(2, '=').collect::<Vec<_>>();
                        if !items.is_empty() {
                            return Err("Expected export of the format NAME=VALUE".into());
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
        return Err("Didn't find a command in your command! macro.".into());
    }

    // Join the command string and split out binary / args.
    let command_string = command_lines.join("\n").replace("\\\n", "\n");
    let mut command =
        shlex::split(&command_string).expect("Command string couldn't be parsed by shlex");
    let binary = command.remove(0);
    let args = command;

    // Generate the CommandSpec struct.
    Ok(CommandSpec {
        binary,
        args,
        env,
        cd,
    }
    .to_command())
}

/// Parse a string into a [`Command`] object.
///
/// [`Command`]: https://doc.rust-lang.org/std/process/struct.Command.html
pub fn internal_sh_inline_commandify<S: AsRef<str>>(
    value: S,
) -> Result<Command, Box<dyn std::error::Error>> {
    impl_commandify(value.as_ref())
}

/// Execute a [`Command`] object.  Only intended
///
/// [`Command`]: https://doc.rust-lang.org/std/process/struct.Command.html
pub fn internal_sh_inline_execute(mut cmd: Command) -> Result<(), std::io::Error> {
    let r = cmd.status()?;
    if !r.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Process [{:?}] failed: {}", cmd, r),
        ));
    }
    Ok(())
}
