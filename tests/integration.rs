use rust_dbg_fmt::format_debug;

// ========================================
// Library API tests
// ========================================

#[test]
fn complex_nested_struct() {
    let input = r#"Config { server: Server { host: "localhost", port: 8080, tls: Some(TlsConfig { cert: "/path/to/cert", key: "/path/to/key" }) }, database: Database { url: "postgres://localhost/db", pool_size: 10, options: Options {} }, features: ["auth", "logging", "metrics"] }"#;
    let expected = r#"Config {
  server: Server {
    host: "localhost",
    port: 8080,
    tls: Some(
      TlsConfig {
        cert: "/path/to/cert",
        key: "/path/to/key",
      },
    ),
  },
  database: Database {
    url: "postgres://localhost/db",
    pool_size: 10,
    options: Options {},
  },
  features: [
    "auth",
    "logging",
    "metrics",
  ],
}"#;
    assert_eq!(format_debug(input, 2), expected);
}

#[test]
fn nested_arrays() {
    let input = "Matrix { rows: [[1, 2, 3], [4, 5, 6]] }";
    let expected = "\
Matrix {
  rows: [
    [
      1,
      2,
      3,
    ],
    [
      4,
      5,
      6,
    ],
  ],
}";
    assert_eq!(format_debug(input, 2), expected);
}

#[test]
fn enum_variants() {
    let input = "Result { ok: Ok(42), err: Err(Error { msg: \"oops\", code: 500 }) }";
    let expected = "\
Result {
  ok: Ok(42),
  err: Err(
    Error {
      msg: \"oops\",
      code: 500,
    },
  ),
}";
    assert_eq!(format_debug(input, 2), expected);
}

#[test]
fn empty_input() {
    assert_eq!(format_debug("", 2), "");
}

#[test]
fn single_value() {
    assert_eq!(format_debug("42", 2), "42");
}

#[test]
fn unit_variant() {
    assert_eq!(format_debug("None", 2), "None");
}

#[test]
fn tuple_struct() {
    let input = "Point(1, 2, 3)";
    let expected = "\
Point(
  1,
  2,
  3,
)";
    assert_eq!(format_debug(input, 2), expected);
}

#[test]
fn string_with_special_chars() {
    let input = r#"Foo { msg: "hello \"world\" {test} [arr]" }"#;
    let expected = "Foo {\n  msg: \"hello \\\"world\\\" {test} [arr]\",\n}";
    assert_eq!(format_debug(input, 2), expected);
}

#[test]
fn indent_width_4() {
    let input = "Foo { bar: 1, baz: 2 }";
    let expected = "\
Foo {
    bar: 1,
    baz: 2,
}";
    assert_eq!(format_debug(input, 4), expected);
}

#[test]
fn lib_used_in_app_context() {
    // Simulate how an app would use the library: format a Debug output of a real struct
    #[allow(dead_code)]
    #[derive(Debug)]
    struct User {
        name: String,
        age: u32,
        tags: Vec<String>,
    }

    let user = User {
        name: "Alice".to_string(),
        age: 30,
        tags: vec!["admin".to_string(), "active".to_string()],
    };

    let debug_str = format!("{:?}", user);
    let pretty = format_debug(&debug_str, 2);

    assert!(pretty.contains("name:"));
    assert!(pretty.contains("age:"));
    assert!(pretty.contains("tags:"));
    assert!(pretty.contains('\n'));
}

#[test]
fn hashmap_output() {
    let input = r#"{1: "a", 2: "b"}"#;
    let expected = "\
{
  1: \"a\",
  2: \"b\",
}";
    assert_eq!(format_debug(input, 2), expected);
}

#[test]
fn hashset_output() {
    let input = "{1, 2, 3}";
    let expected = "\
{
  1,
  2,
  3,
}";
    assert_eq!(format_debug(input, 2), expected);
}

#[test]
fn socket_addr_in_struct() {
    let input = "Server { addr: 127.0.0.1:8080, name: \"web\" }";
    let expected = "\
Server {
  addr: 127.0.0.1:8080,
  name: \"web\",
}";
    assert_eq!(format_debug(input, 2), expected);
}

#[test]
fn single_value_enum_inline() {
    assert_eq!(format_debug("Some(42)", 2), "Some(42)");
    assert_eq!(format_debug("Ok(\"hello\")", 2), "Ok(\"hello\")");
}
