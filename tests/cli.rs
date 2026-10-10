use std::process::Command;

#[test]
fn usage_errors_exit_one_without_stdout() {
    for args in [vec![], vec!["--bogus"], vec!["a.png", "b.png"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_haru"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn help_and_version_exit_zero_without_clipboard_access() {
    for argument in ["--help", "--version"] {
        let output = Command::new(env!("CARGO_BIN_EXE_haru"))
            .arg(argument)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let stdout = String::from_utf8(output.stdout).unwrap();
        if argument == "--version" {
            assert_eq!(stdout.trim(), concat!("haru ", env!("CARGO_PKG_VERSION")));
        } else {
            assert!(stdout.contains("Usage:"));
        }
    }
}
