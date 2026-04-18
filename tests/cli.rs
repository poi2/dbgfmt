use std::io::Write;
use std::process::{Command, Stdio};

fn cargo_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dbgfmt"))
}

// ========================================
// CLI E2E tests
// ========================================

#[test]
fn cli_argument_input() {
    let output = cargo_bin()
        .args(["Foo", "{", "bar:", "1,", "baz:", "2", "}"])
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.trim(),
        "\
Foo {
  bar: 1,
  baz: 2,
}"
    );
}

#[test]
fn cli_stdin_input() {
    let mut child = cargo_bin()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"Foo { bar: 1, baz: 2 }")
        .expect("failed to write");
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("failed to wait");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.trim(),
        "\
Foo {
  bar: 1,
  baz: 2,
}"
    );
}

#[test]
fn cli_help_flag() {
    let output = cargo_bin()
        .arg("--help")
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage:"));
}

#[test]
fn cli_version_flag() {
    let output = cargo_bin()
        .arg("--version")
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dbgfmt"));
}

#[test]
fn cli_complex_pipe() {
    let input = r#"Config { server: Server { host: "localhost", port: 8080 }, items: [1, 2, 3] }"#;

    let mut child = cargo_bin()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .expect("failed to write");
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("failed to wait");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.trim(),
        r#"Config {
  server: Server {
    host: "localhost",
    port: 8080,
  },
  items: [
    1,
    2,
    3,
  ],
}"#
    );
}

#[test]
fn cli_indent_option() {
    let output = cargo_bin()
        .args(["--indent", "4", "Foo { bar: 1 }"])
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.trim(),
        "\
Foo {
    bar: 1,
}"
    );
}

#[test]
fn cli_indent_equals_syntax() {
    let output = cargo_bin()
        .args(["--indent=4", "Foo { bar: 1 }"])
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.trim(),
        "\
Foo {
    bar: 1,
}"
    );
}

#[test]
fn cli_short_indent_option() {
    let output = cargo_bin()
        .args(["-i", "4", "Foo { bar: 1 }"])
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.trim(),
        "\
Foo {
    bar: 1,
}"
    );
}

#[test]
fn cli_color_always() {
    let output = cargo_bin()
        .args(["--color", "always", "Foo { bar: 1 }"])
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\x1b["));
}

#[test]
fn cli_color_never() {
    let output = cargo_bin()
        .args(["--color", "never", "Foo { bar: 1 }"])
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("\x1b["));
}

#[test]
fn cli_no_input_shows_usage() {
    // stdin is not a pipe (default inherits terminal), and no args → should fail with usage
    let output = cargo_bin()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn")
        .wait_with_output()
        .expect("failed to wait");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage:"));
}
