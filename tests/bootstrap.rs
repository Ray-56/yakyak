use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yakyak"))
        .args(args)
        .output()
        .expect("run the actual yakyak binary")
}

#[test]
fn no_arguments_reports_unimplemented_service() {
    let output = run(&[]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(
        output.stdout,
        "YakYak bootstrap：通信服务尚未实现。\n".as_bytes()
    );
}

#[test]
fn help_describes_only_bootstrap_options() {
    let output = run(&["--help"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let help = String::from_utf8(output.stdout).expect("UTF-8 help");
    for required in ["bootstrap", "通信服务尚未实现", "--help", "--version"] {
        assert!(help.contains(required), "missing help text: {required}");
    }
}

#[test]
fn version_comes_from_cargo_package() {
    let output = run(&["--version"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(
        output.stdout,
        format!("yakyak {}\n", env!("CARGO_PKG_VERSION")).as_bytes()
    );
}

#[test]
fn unknown_argument_is_a_usage_error() {
    for argument in ["--unknown", "serve", "-h", "", "--"] {
        let output = run(&[argument]);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, b"error: unknown argument; use --help\n");
    }
}

#[test]
fn multiple_arguments_are_rejected() {
    for args in [
        vec!["--help", "--version"],
        vec!["--help", "--help"],
        vec!["--version", "--unknown"],
        vec!["serve", "--help"],
        vec!["--help", "--version", "--help"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert_eq!(
            output.stderr,
            b"error: invalid argument combination; use --help\n"
        );
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_argument_is_rejected_without_panicking() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let output = Command::new(env!("CARGO_BIN_EXE_yakyak"))
        .arg(OsStr::from_bytes(&[0xff]))
        .output()
        .expect("run the actual yakyak binary");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"error: unknown argument; use --help\n");
}
