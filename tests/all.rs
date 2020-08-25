extern crate sh_inline;
use sh_inline::command;

#[cfg(not(windows))]
mod sh {
    use sh_inline::{bash, bash_command};

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
