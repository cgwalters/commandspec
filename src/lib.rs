//! # Macros to run bash scripts inline in Rust
//!
//! In many cases it's convenient to run child processes,
//! particularly via Unix shell script.  Writing the
//! Rust code to use `std::process::Command` directly
//! will get very verbose quickly.  You can generate
//! a script "manually" by using e.g. `format!()` but
//! there are some important yet subtle things to get right,
//! such as dealing with quoting issues.
//!
//! This macro takes Rust variable names at the start
//! that are converted to a string (quoting as necessary)
//! and bound into the script as bash variables.
//!
//! Further, the generated scripts use "bash strict mode"
//! by default, i.e. `set -euo pipefail`.
//!
//! ```
//! use sh_inline::*;
//! let foo = "variable with spaces";
//! bash!(r#"test "${foo}" = 'variable with spaces'"#, foo)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! This generates and executes bash script as follows:
//! ```sh
//! set -euo pipefail
//! foo="variable with spaces"
//! test ${foo} = 'variable with spaces'
//! ```

#[doc(hidden)]
pub mod internals;

/// Create a [`Command`] object that will execute a fragment of (Bash) shell script
/// in "strict mode", i.e. with `set -euo pipefail`.  The script will be substituted
/// similarly to `format!`.
///
/// [`Command`]: https://doc.rust-lang.org/std/process/struct.Command.html
#[macro_export]
macro_rules! bash_command {
    ($s:expr) => { $crate::bash_command!($s,) };
    ($s:expr, $( $id:ident ),*) => {
        {
            use std::fmt::Write;
            let mut tmp_cmd = std::process::Command::new("bash");
            tmp_cmd.arg("-c");
            let mut script: String = "set -euo pipefail\n".into();
            $(
                write!(&mut script, "{}={}\n", stringify!($id), $crate::internals::command_arg(&$id)).unwrap();
            )*
            script.push_str(&$s);
            tmp_cmd.arg(script);
            tmp_cmd
        }
    };
}

/// Execute a fragment of Bash shell script, returning an error if the subprocess exits unsuccessfully.
/// This is intended as a convenience function;
/// if for example you might want to change behavior based on specific
/// exit codes, it's recommended to use `bash_command()` instead.
#[macro_export]
macro_rules! bash {
    ($s:expr) => { $crate::bash!($s,) };
    ($s:expr, $( $id:ident ),*) => {
        $crate::internals::execute($crate::bash_command!($s, $( $id ),*))
    };
}
