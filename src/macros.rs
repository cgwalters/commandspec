/// Create a [`Command`] object by parsing an argument string.
///
/// [`Command`]: https://doc.rust-lang.org/std/process/struct.Command.html
#[macro_export]
macro_rules! command {
    ($fmt:expr) => ( command!($fmt,) );
    ($fmt:expr, $( $id:ident = $value:expr ),* $(,)*) => (
        {
            $crate::commandify(
                format!($fmt, $( $id = $crate::command_arg(&$value) ),*)
            )
        }
    );
}

/// Parse and execute a single command, returning an error if it
/// exits unsuccessfully.
#[macro_export]
macro_rules! execute {
    ($fmt:expr) => ( execute!($fmt,) );
    ($fmt:expr, $( $id:ident = $value:expr ),* $(,)*) => (
        {
            use $crate::{CommandSpecExt};
            $crate::command!($fmt, $( $id = $value ),*).unwrap().execute()
        }
    );
}

/// Create a [`Command`] object that will execute a fragment of (Bash) shell script
/// in "strict mode", i.e. with `set -euo pipefail`.
///
/// [`Command`]: https://doc.rust-lang.org/std/process/struct.Command.html
#[macro_export]
macro_rules! bash_command {
    ($fmt:expr) => ( $crate::bash_command!($fmt,) );
    ($fmt:expr, $( $id:ident = $value:expr ),* $(,)*) => (
        $crate::commandify(
            format!(
                "bash -c {}",
                $crate::command_arg(
                    &format!("set -euo pipefail\n\n{}", format!($fmt, $( $id = $crate::command_arg(&$value) ,)*)),
                ),
            )
        )
    );
}

/// Execute a fragment of Bash shell script, returning an error if the subprocess exits unsuccessfully.
#[macro_export]
macro_rules! bash {
    ($fmt:expr) => ( $crate::bash!($fmt,) );
    ($fmt:expr, $( $id:ident = $value:expr ),* $(,)*) => (
        {
            use $crate::{CommandSpecExt};
            $crate::bash_command!($fmt, $( $id = $value ),*).unwrap().execute()
        }
    );
}
