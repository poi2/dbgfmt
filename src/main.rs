use std::fmt;
use std::io::{self, IsTerminal, Read};

use dbgfmt::Token;

struct Options {
    indent_width: usize,
    color: ColorMode,
    recover: bool,
    input: Option<String>,
}

enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug)]
struct FormatError {
    line: usize,
    column: usize,
    message: String,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

fn main() {
    let opts = match parse_args() {
        Ok(opts) => opts,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    let input = if let Some(input) = opts.input {
        input
    } else if !io::stdin().is_terminal() {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
            eprintln!("error: failed to read stdin: {e}");
            std::process::exit(1);
        });
        if buf.trim().is_empty() {
            print_usage();
            std::process::exit(1);
        }
        buf
    } else {
        print_usage();
        std::process::exit(1);
    };

    let use_color = match opts.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => io::stdout().is_terminal(),
    };

    match format_cli_input(input.trim_end(), opts.indent_width, use_color, opts.recover) {
        Ok(output) => println!("{output}"),
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!("hint: use --recover to best-effort format broken input");
            std::process::exit(1);
        }
    }
}

/// CLI-specific formatting: handles multi-line input, dbg! prefix stripping,
/// bracket validation, and multi-value splitting.
/// When `recover` is enabled, bracket validation is skipped and broken
/// bracket structure is best-effort repaired before formatting.
fn format_cli_input(
    input: &str,
    indent_width: usize,
    colored: bool,
    recover: bool,
) -> Result<String, FormatError> {
    let mut results = Vec::new();

    for (line_idx, line) in input.lines().enumerate() {
        let trimmed = line.trim_end();
        if trimmed.trim_start().is_empty() {
            continue;
        }

        let (prefix, value) = strip_dbg_prefix(trimmed);

        if !recover {
            let column_offset = prefix.as_ref().map_or(0, |p| p.chars().count());
            validate_brackets(value, line_idx + 1, column_offset)?;
        }

        let mut tokens = dbgfmt::tokenize(value);
        if recover {
            tokens = recover_tokens(tokens);
        }
        let groups = split_into_values(&tokens);

        for group in &groups {
            let formatted = if colored {
                dbgfmt::format_tokens_colored(group, indent_width)
            } else {
                dbgfmt::format_tokens(group, indent_width)
            };

            if let Some(ref p) = prefix {
                results.push(format!("{p}{formatted}"));
            } else {
                results.push(formatted);
            }
        }
    }

    Ok(results.join("\n"))
}

/// Strip `[file:line:col] expr = ` prefix from `dbg!()` output.
/// Handles leading whitespace (e.g. indented log output).
fn strip_dbg_prefix(line: &str) -> (Option<String>, &str) {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('[') {
        return (None, line);
    }

    let leading_ws = line.len() - trimmed.len();
    let bracket_end = match trimmed.find("] ") {
        Some(i) => i + 2,
        None => return (None, line),
    };

    let rest = &trimmed[bracket_end..];
    let eq_pos = match rest.find(" = ") {
        Some(i) => leading_ws + bracket_end + i + 3,
        None => return (None, line),
    };

    let prefix = &line[..eq_pos];
    let value = &line[eq_pos..];

    (Some(prefix.to_string()), value)
}

/// Validate that brackets are balanced in the input.
fn validate_brackets(
    input: &str,
    line_num: usize,
    column_offset: usize,
) -> Result<(), FormatError> {
    let mut stack: Vec<(char, usize)> = Vec::new();
    let mut chars = input.chars().enumerate().peekable();

    while let Some((col, ch)) = chars.next() {
        match ch {
            '"' => {
                while let Some((_, c)) = chars.next() {
                    if c == '\\' {
                        chars.next();
                    } else if c == '"' {
                        break;
                    }
                }
            }
            '\'' => {
                while let Some((_, c)) = chars.next() {
                    if c == '\\' {
                        chars.next();
                    } else if c == '\'' {
                        break;
                    }
                }
            }
            '{' | '[' | '(' => stack.push((ch, column_offset + col + 1)),
            '}' | ']' | ')' => {
                let expected = match ch {
                    '}' => '{',
                    ']' => '[',
                    ')' => '(',
                    _ => unreachable!(),
                };
                match stack.pop() {
                    None => {
                        return Err(FormatError {
                            line: line_num,
                            column: column_offset + col + 1,
                            message: format!("unexpected '{ch}'"),
                        });
                    }
                    Some((open, open_col)) => {
                        if open != expected {
                            return Err(FormatError {
                                line: line_num,
                                column: column_offset + col + 1,
                                message: format!(
                                    "mismatched bracket: expected '{}' to close '{open}' (column {open_col}), found '{ch}'",
                                    match open {
                                        '{' => '}',
                                        '[' => ']',
                                        '(' => ')',
                                        _ => unreachable!(),
                                    }
                                ),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if let Some((ch, col)) = stack.last() {
        return Err(FormatError {
            line: line_num,
            column: *col,
            message: format!("unclosed '{ch}'"),
        });
    }

    Ok(())
}

/// Best-effort recovery of broken bracket structure in a token stream.
/// - Removes unexpected close brackets (no matching opener)
/// - Fixes mismatched close brackets (replaces with correct closer)
/// - Appends missing close brackets at the end
fn recover_tokens(tokens: Vec<Token>) -> Vec<Token> {
    let mut result = Vec::with_capacity(tokens.len());
    let mut stack: Vec<Token> = Vec::new();

    for token in tokens {
        match &token {
            Token::OpenBrace | Token::OpenBracket | Token::OpenParen => {
                stack.push(token.clone());
                result.push(token);
            }
            Token::CloseBrace | Token::CloseBracket | Token::CloseParen => {
                if let Some(opener) = stack.last() {
                    let expected_closer = match opener {
                        Token::OpenBrace => Token::CloseBrace,
                        Token::OpenBracket => Token::CloseBracket,
                        Token::OpenParen => Token::CloseParen,
                        _ => unreachable!(),
                    };
                    // Use the correct closer regardless of what was in the input
                    result.push(expected_closer);
                    stack.pop();
                }
                // If stack is empty, skip the orphan closer
            }
            _ => {
                result.push(token);
            }
        }
    }

    // Close any remaining open brackets (in reverse order)
    while let Some(opener) = stack.pop() {
        let closer = match opener {
            Token::OpenBrace => Token::CloseBrace,
            Token::OpenBracket => Token::CloseBracket,
            Token::OpenParen => Token::CloseParen,
            _ => unreachable!(),
        };
        result.push(closer);
    }

    result
}

/// Split a token stream into separate values based on nesting depth.
fn split_into_values(tokens: &[Token]) -> Vec<Vec<Token>> {
    if tokens.is_empty() {
        return vec![];
    }

    let mut groups: Vec<Vec<Token>> = Vec::new();
    let mut current: Vec<Token> = Vec::new();
    let mut depth: usize = 0;

    for (i, token) in tokens.iter().enumerate() {
        // Skip top-level commas between values
        if depth == 0 && matches!(token, Token::Comma) {
            continue;
        }

        current.push(token.clone());

        match token {
            Token::OpenBrace | Token::OpenBracket | Token::OpenParen => {
                depth += 1;
            }
            Token::CloseBrace | Token::CloseBracket | Token::CloseParen => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    groups.push(std::mem::take(&mut current));
                }
            }
            Token::Text(_) if depth == 0 => {
                // Check if next token is an opener — if so, this text is a type name
                let next_is_opener = tokens.get(i + 1).is_some_and(|t| {
                    matches!(t, Token::OpenBrace | Token::OpenBracket | Token::OpenParen)
                });
                if !next_is_opener {
                    groups.push(std::mem::take(&mut current));
                }
            }
            _ => {}
        }
    }

    if !current.is_empty() {
        groups.push(current);
    }

    groups
}

fn parse_args() -> Result<Options, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut indent_width: usize = 4;
    let mut color = ColorMode::Auto;
    let mut recover = false;
    let mut positional = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--" => {
                i += 1;
                while i < args.len() {
                    positional.push(args[i].clone());
                    i += 1;
                }
                break;
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("dbgfmt {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "-i" | "--indent" => {
                i += 1;
                let val = args.get(i).ok_or("--indent requires a value")?;
                indent_width = parse_indent(val)?;
            }
            "--color" => {
                i += 1;
                let val = args.get(i).ok_or("--color requires a value")?;
                color = parse_color(val)?;
            }
            "-r" | "--recover" => {
                recover = true;
            }
            arg if arg.starts_with("--indent=") => {
                let val = &arg["--indent=".len()..];
                indent_width = parse_indent(val)?;
            }
            arg if arg.starts_with("--color=") => {
                let val = &arg["--color=".len()..];
                color = parse_color(val)?;
            }
            _ => positional.push(args[i].clone()),
        }
        i += 1;
    }

    let input = if positional.is_empty() {
        None
    } else {
        Some(positional.join(" "))
    };

    Ok(Options {
        indent_width,
        color,
        recover,
        input,
    })
}

const MAX_INDENT: usize = 32;

fn parse_indent(val: &str) -> Result<usize, String> {
    let n: usize = val
        .parse()
        .map_err(|_| format!("invalid indent value: {val}"))?;
    if n > MAX_INDENT {
        return Err(format!("indent value too large: {n} (max: {MAX_INDENT})"));
    }
    Ok(n)
}

fn parse_color(val: &str) -> Result<ColorMode, String> {
    match val {
        "auto" => Ok(ColorMode::Auto),
        "always" => Ok(ColorMode::Always),
        "never" => Ok(ColorMode::Never),
        _ => Err(format!(
            "invalid color value: {val} (expected auto, always, never)"
        )),
    }
}

fn print_usage() {
    eprintln!(
        "\
Usage: dbgfmt [OPTIONS] [INPUT]

Arguments:
  [INPUT]    Rust Debug format string to pretty-print. If omitted, reads from stdin.

Options:
  -i, --indent <N>     Indent width (default: 4)
      --color <WHEN>    Color output: auto, always, never (default: auto)
  -r, --recover        Best-effort recovery of broken bracket structure
  -h, --help           Print help
  -V, --version        Print version"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // === dbg! prefix tests ===

    #[test]
    fn strip_dbg_prefix_basic() {
        let (prefix, value) = strip_dbg_prefix("[src/main.rs:5:5] my_struct = Foo { bar: 1 }");
        assert_eq!(prefix.unwrap(), "[src/main.rs:5:5] my_struct = ");
        assert_eq!(value, "Foo { bar: 1 }");
    }

    #[test]
    fn strip_dbg_prefix_no_prefix() {
        let (prefix, value) = strip_dbg_prefix("Foo { bar: 1 }");
        assert!(prefix.is_none());
        assert_eq!(value, "Foo { bar: 1 }");
    }

    #[test]
    fn strip_dbg_prefix_not_dbg() {
        let (prefix, value) = strip_dbg_prefix("[1, 2, 3]");
        assert!(prefix.is_none());
        assert_eq!(value, "[1, 2, 3]");
    }

    #[test]
    fn format_dbg_output() {
        let input = "[src/main.rs:5:5] my_struct = Foo { bar: 1, baz: 2 }";
        let output = format_cli_input(input, 4, false, false).unwrap();
        assert_eq!(
            output,
            "[src/main.rs:5:5] my_struct = Foo {\n    bar: 1,\n    baz: 2,\n}"
        );
    }

    #[test]
    fn format_multiple_dbg_lines() {
        let input = "[src/main.rs:5:5] x = Foo { a: 1 }\n[src/main.rs:6:5] y = Bar { b: 2 }";
        let output = format_cli_input(input, 4, false, false).unwrap();
        assert_eq!(
            output,
            "[src/main.rs:5:5] x = Foo {\n    a: 1,\n}\n[src/main.rs:6:5] y = Bar {\n    b: 2,\n}"
        );
    }

    #[test]
    fn dbg_prefix_with_array() {
        let input = "[src/main.rs:5:5] items = [1, 2, 3]";
        let output = format_cli_input(input, 4, false, false).unwrap();
        assert_eq!(
            output,
            "[src/main.rs:5:5] items = [\n    1,\n    2,\n    3,\n]"
        );
    }

    // === multi-value tests ===

    #[test]
    fn multi_value_same_line() {
        let input = "Foo { x: 1 } Bar { y: 2 }";
        let output = format_cli_input(input, 4, false, false).unwrap();
        assert_eq!(output, "Foo {\n    x: 1,\n}\nBar {\n    y: 2,\n}");
    }

    #[test]
    fn multi_value_separate_lines() {
        let input = "Foo { x: 1 }\nBar { y: 2 }";
        let output = format_cli_input(input, 4, false, false).unwrap();
        assert_eq!(output, "Foo {\n    x: 1,\n}\nBar {\n    y: 2,\n}");
    }

    #[test]
    fn multi_value_bare_values() {
        let input = "42\nNone\n\"hello\"";
        let output = format_cli_input(input, 4, false, false).unwrap();
        assert_eq!(output, "42\nNone\n\"hello\"");
    }

    #[test]
    fn multi_value_bare_same_line() {
        let input = "Some(42) None";
        let output = format_cli_input(input, 4, false, false).unwrap();
        assert_eq!(output, "Some(42)\nNone");
    }

    #[test]
    fn multi_value_with_comma_separator() {
        let input = "Foo { x: 1 }, Bar { y: 2 }";
        let output = format_cli_input(input, 4, false, false).unwrap();
        assert_eq!(output, "Foo {\n    x: 1,\n}\nBar {\n    y: 2,\n}");
    }

    // === bracket validation tests ===

    #[test]
    fn error_unclosed_brace() {
        let err = format_cli_input("Foo { bar: 1", 4, false, false).unwrap_err();
        assert_eq!(err.line, 1);
        assert_eq!(err.column, 5);
        assert!(err.message.contains("unclosed"));
    }

    #[test]
    fn error_unexpected_close() {
        let err = format_cli_input("Foo } bar", 4, false, false).unwrap_err();
        assert_eq!(err.line, 1);
        assert_eq!(err.column, 5);
        assert!(err.message.contains("unexpected"));
    }

    #[test]
    fn error_mismatched_brackets() {
        let err = format_cli_input("Foo { bar: 1 )", 4, false, false).unwrap_err();
        assert_eq!(err.line, 1);
        assert_eq!(err.column, 14);
        assert!(err.message.contains("mismatched"));
    }

    #[test]
    fn error_on_second_line() {
        let err = format_cli_input("Foo { x: 1 }\nBar { y: 2", 4, false, false).unwrap_err();
        assert_eq!(err.line, 2);
        assert!(err.message.contains("unclosed"));
    }

    #[test]
    fn error_column_offset_with_dbg_prefix() {
        let err =
            format_cli_input("[src/main.rs:5:5] x = Foo { bar: 1", 4, false, false).unwrap_err();
        assert_eq!(err.line, 1);
        assert_eq!(err.column, 27);
        assert!(err.message.contains("unclosed"));
    }

    #[test]
    fn empty_input() {
        assert_eq!(format_cli_input("", 4, false, false).unwrap(), "");
    }

    #[test]
    fn blank_lines_skipped() {
        let input = "Foo { x: 1 }\n\n\nBar { y: 2 }";
        let output = format_cli_input(input, 4, false, false).unwrap();
        assert_eq!(output, "Foo {\n    x: 1,\n}\nBar {\n    y: 2,\n}");
    }

    // === recover tests ===

    #[test]
    fn recover_unclosed_brace() {
        let output = format_cli_input("Foo { bar: 1", 4, false, true).unwrap();
        assert_eq!(output, "Foo {\n    bar: 1,\n}");
    }

    #[test]
    fn recover_unclosed_nested() {
        let output = format_cli_input("Foo { bar: Bar { x: 1", 4, false, true).unwrap();
        assert_eq!(output, "Foo {\n    bar: Bar {\n        x: 1,\n    },\n}");
    }

    #[test]
    fn recover_unexpected_close() {
        let output = format_cli_input("Foo } bar", 4, false, true).unwrap();
        assert_eq!(output, "Foo\nbar");
    }

    #[test]
    fn recover_mismatched_bracket() {
        let output = format_cli_input("Foo { bar: 1 )", 4, false, true).unwrap();
        assert_eq!(output, "Foo {\n    bar: 1,\n}");
    }

    #[test]
    fn recover_truncated_value() {
        let output = format_cli_input("Foo { bar: Bar { x: 1, y:", 4, false, true).unwrap();
        assert_eq!(
            output,
            "Foo {\n    bar: Bar {\n        x: 1,\n        y:,\n    },\n}"
        );
    }

    #[test]
    fn recover_extra_close_brackets() {
        let output = format_cli_input("Foo { bar: 1 }}", 4, false, true).unwrap();
        assert_eq!(output, "Foo {\n    bar: 1,\n}");
    }

    #[test]
    fn recover_valid_input_unchanged() {
        let output = format_cli_input("Foo { bar: 1, baz: 2 }", 4, false, true).unwrap();
        assert_eq!(output, "Foo {\n    bar: 1,\n    baz: 2,\n}");
    }

    #[test]
    fn recover_unclosed_bracket() {
        let output = format_cli_input("[1, 2, 3", 4, false, true).unwrap();
        assert_eq!(output, "[\n    1,\n    2,\n    3,\n]");
    }

    #[test]
    fn recover_unclosed_paren() {
        let output = format_cli_input("Some(42", 4, false, true).unwrap();
        assert_eq!(output, "Some(42)");
    }

    #[test]
    fn recover_multiline() {
        let output = format_cli_input("Foo { x: 1 }\nBar { y: 2", 4, false, true).unwrap();
        assert_eq!(output, "Foo {\n    x: 1,\n}\nBar {\n    y: 2,\n}");
    }

    // === recover_tokens unit tests ===

    #[test]
    fn recover_tokens_balanced() {
        let tokens = dbgfmt::tokenize("Foo { bar: 1 }");
        let recovered = recover_tokens(tokens.clone());
        assert_eq!(recovered, tokens);
    }

    #[test]
    fn recover_tokens_unclosed() {
        let tokens = dbgfmt::tokenize("Foo { bar: 1");
        let recovered = recover_tokens(tokens);
        assert_eq!(
            recovered,
            vec![
                Token::Text("Foo".into()),
                Token::OpenBrace,
                Token::Text("bar".into()),
                Token::Colon,
                Token::Text("1".into()),
                Token::CloseBrace,
            ]
        );
    }

    #[test]
    fn recover_tokens_orphan_closer() {
        let tokens = dbgfmt::tokenize("Foo } bar");
        let recovered = recover_tokens(tokens);
        assert_eq!(
            recovered,
            vec![Token::Text("Foo".into()), Token::Text("bar".into()),]
        );
    }

    #[test]
    fn recover_tokens_mismatched() {
        let tokens = dbgfmt::tokenize("Foo { bar: 1 )");
        let recovered = recover_tokens(tokens);
        assert_eq!(
            recovered,
            vec![
                Token::Text("Foo".into()),
                Token::OpenBrace,
                Token::Text("bar".into()),
                Token::Colon,
                Token::Text("1".into()),
                Token::CloseBrace,
            ]
        );
    }
}
