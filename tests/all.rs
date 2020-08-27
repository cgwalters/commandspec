extern crate sh_inline;
use sh_inline::{bash, bash_command};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

#[test]
fn sh_exit_var() {
    let a = 42;
    assert!(bash!(r"exit ${a}", a).is_err())
}

#[test]
fn multi_vars() -> Result<(), std::io::Error> {
    let a = "foo";
    let b = Path::new("bar");
    let c = 42;
    let d: String = "baz".into();
    bash!(
        r#"test "${a} ${b} ${c} ${d}" = "foo bar 42 baz""#,
        a,
        b,
        c,
        d
    )?;
    Ok(())
}

#[test]
fn sh_echo1() {
    let a = "SENTINEL";
    let res = bash_command!(r"A=${a}; echo $A", a).output().unwrap();
    assert_eq!(res.stdout, b"SENTINEL\n");
}

#[test]
fn sh_unset_var() {
    assert!(bash!(r"echo $UNSETVALUE").is_err());
}

#[test]
fn sh_pipefail() {
    assert!(bash!(r"false | true").is_err());
}

#[test]
fn sh_empty() {
    bash!(r"true").unwrap();
}

#[test]
fn sh_path() {
    let p = Path::new("/no/such/path");
    bash!(r"echo ${p} >/dev/null", p).unwrap();
}

#[test]
fn sh_path_binary() {
    let p = Path::new(OsStr::from_bytes(&[0x21, 0, 0xFF, 0x22, 0x61]));
    bash!(r#"test ${p} = $'!\x00\xFF\"a'"#, p).unwrap();
}
