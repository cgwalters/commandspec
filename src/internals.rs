use std::fmt;
use std::path::Path;
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

/// Execute a [`Command`] object.  Only intended
///
/// [`Command`]: https://doc.rust-lang.org/std/process/struct.Command.html
pub fn execute(mut cmd: Command) -> Result<(), std::io::Error> {
    let r = cmd.status()?;
    if !r.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("bash script failed: {}", r),
        ));
    }
    Ok(())
}
