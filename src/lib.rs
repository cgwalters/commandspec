//! # Macros to run commands/shell inline in Rust
//!
//! In many cases it's convenient to run child processes,
//! particularly via Unix shell script.  However, there
//! are some subtle things to get right in doing this,
//! such as dealing with quoting issues.
//!
//! ```
//! use sh_inline::*;
//! let foo = "variable with spaces";
//! bash!("test {foo} = 'variable with spaces'", foo = foo)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#[doc(hidden)]
pub mod internals;

/// Create a [`Command`] object that will execute a fragment of (Bash) shell script
/// in "strict mode", i.e. with `set -euo pipefail`.
///
/// [`Command`]: https://doc.rust-lang.org/std/process/struct.Command.html
#[macro_export]
macro_rules! bash_command {
    ($fmt:expr) => ( $crate::bash_command!($fmt,) );
    ($fmt:expr, $( $id:ident = $value:expr ),* $(,)*) => (
        $crate::internals::bash_inline(format!($fmt, $( $id = $crate::internals::command_arg(&$value) ,)*))
    );
}

/// Execute a fragment of Bash shell script, returning an error if the subprocess exits unsuccessfully.
/// This is intended as a convenience function;
/// if for example you might want to change behavior based on specific
/// exit codes, it's recommended to use `bash_command()` instead.
#[macro_export]
macro_rules! bash {
    ($fmt:expr) => ( $crate::bash!($fmt,) );
    ($fmt:expr, $( $id:ident = $value:expr ),* $(,)*) => (
        {
            $crate::internals::execute($crate::bash_command!($fmt, $( $id = $value ),*).unwrap())
        }
    );
}
