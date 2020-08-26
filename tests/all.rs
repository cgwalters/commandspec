extern crate sh_inline;
use sh_inline::command;

#[cfg(not(windows))]
mod sh {
    use std::path::Path;
    use sh_inline::{bash, bash_command};
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    #[test]
    fn sh_exit() {
        assert!(bash!(r"exit {a}", a = 42).is_err())
    }

    #[test]
    fn sh_echo1() {
        let res = bash_command!(r"A={a}; echo $A", a = "SENTINEL")
            .unwrap()
            .output()
            .unwrap();
        assert_eq!(res.stdout, b"SENTINEL\n");
    }

    #[test]
    fn sh_echo2() {
        let res = bash_command!(r"A={a}; echo $A", a = "SENTINEL",)
            .unwrap()
            .output()
            .unwrap();
        assert_eq!(res.stdout, b"SENTINEL\n");
    }

    #[test]
    fn sh_unset_var() {
        assert!(bash!(r"echo $UNSETVALUE").is_err());
    }

    #[test]
    fn sh_empty() {
        bash!(r"true").unwrap();
    }

    #[test]
    fn sh_empty_comma() {
        bash!(r"true",).unwrap();
    }

    #[test]
    fn sh_path() {
        let p = Path::new("/no/such/path");
        bash!(r"echo {p} >/dev/null", p = p).unwrap();
    }

    #[test]
    fn sh_path_binary() {
        let p = Path::new(OsStr::from_bytes(&[0x21, 0, 0xFF, 0x22, 0x61]));
        bash!(r#"test {p} = $'!\x00\xFF\"a'"#, p = p).unwrap();
    }
}

#[test]
fn cmd_rustc() {
    let args = vec!["-V"];
    let res = command!(
        r"
            rustc {args}
        ",
        args = args,
    )
    .unwrap()
    .output()
    .unwrap();
    assert!(res.stdout.starts_with(b"rustc "));
}
