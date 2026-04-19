mod color;
mod formatter;
mod tokenizer;

use std::fmt;

use tokenizer::Token;

/// Error returned when the input contains invalid bracket structure.
#[derive(Debug)]
pub struct FormatError {
    pub line: usize,
    pub column: usize,
    pub message: String,
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

impl std::error::Error for FormatError {}

pub fn format_debug(input: &str, indent_width: usize) -> Result<String, FormatError> {
    format_lines(input, indent_width, false)
}

pub fn format_debug_colored(input: &str, indent_width: usize) -> Result<String, FormatError> {
    format_lines(input, indent_width, true)
}

fn format_lines(input: &str, indent_width: usize, colored: bool) -> Result<String, FormatError> {
    let mut results = Vec::new();

    for (line_idx, line) in input.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let (prefix, value) = strip_dbg_prefix(trimmed);

        let column_offset = prefix.as_ref().map_or(0, |p| p.len());
        validate_brackets(value, line_idx + 1, column_offset)?;

        let tokens = tokenizer::tokenize(value);
        let groups = split_into_values(&tokens);

        for group in &groups {
            let formatted = if colored {
                color::format_tokens_colored(group, indent_width)
            } else {
                formatter::format_tokens(group, indent_width)
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
fn strip_dbg_prefix(line: &str) -> (Option<String>, &str) {
    if !line.starts_with('[') {
        return (None, line);
    }

    let bracket_end = match line.find("] ") {
        Some(i) => i + 2,
        None => return (None, line),
    };

    let rest = &line[bracket_end..];
    let eq_pos = match rest.find(" = ") {
        Some(i) => bracket_end + i + 3,
        None => return (None, line),
    };

    let prefix = &line[..eq_pos];
    let value = &line[eq_pos..];

    (Some(prefix.to_string()), value)
}

/// Validate that brackets are balanced in the input.
/// `column_offset` adjusts reported columns when a prefix (e.g. dbg!) was stripped.
fn validate_brackets(
    input: &str,
    line_num: usize,
    column_offset: usize,
) -> Result<(), FormatError> {
    let mut stack: Vec<(char, usize)> = Vec::new();
    let mut chars = input.char_indices().peekable();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_format() {
        assert_eq!(
            format_debug("Foo { bar: 1 }", 4).unwrap(),
            "Foo {\n    bar: 1,\n}"
        );
    }

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
        let output = format_debug(input, 4).unwrap();
        assert_eq!(
            output,
            "[src/main.rs:5:5] my_struct = Foo {\n    bar: 1,\n    baz: 2,\n}"
        );
    }

    #[test]
    fn format_multiple_dbg_lines() {
        let input = "[src/main.rs:5:5] x = Foo { a: 1 }\n[src/main.rs:6:5] y = Bar { b: 2 }";
        let output = format_debug(input, 4).unwrap();
        assert_eq!(
            output,
            "[src/main.rs:5:5] x = Foo {\n    a: 1,\n}\n[src/main.rs:6:5] y = Bar {\n    b: 2,\n}"
        );
    }

    // === multi-value tests ===

    #[test]
    fn multi_value_same_line() {
        let input = "Foo { x: 1 } Bar { y: 2 }";
        let output = format_debug(input, 4).unwrap();
        assert_eq!(output, "Foo {\n    x: 1,\n}\nBar {\n    y: 2,\n}");
    }

    #[test]
    fn multi_value_separate_lines() {
        let input = "Foo { x: 1 }\nBar { y: 2 }";
        let output = format_debug(input, 4).unwrap();
        assert_eq!(output, "Foo {\n    x: 1,\n}\nBar {\n    y: 2,\n}");
    }

    #[test]
    fn multi_value_bare_values() {
        let input = "42\nNone\n\"hello\"";
        let output = format_debug(input, 4).unwrap();
        assert_eq!(output, "42\nNone\n\"hello\"");
    }

    #[test]
    fn multi_value_bare_same_line() {
        let input = "Some(42) None";
        let output = format_debug(input, 4).unwrap();
        assert_eq!(output, "Some(42)\nNone");
    }

    // === bracket validation tests ===

    #[test]
    fn error_unclosed_brace() {
        let err = format_debug("Foo { bar: 1", 4).unwrap_err();
        assert_eq!(err.line, 1);
        assert_eq!(err.column, 5);
        assert!(err.message.contains("unclosed"));
    }

    #[test]
    fn error_unexpected_close() {
        let err = format_debug("Foo } bar", 4).unwrap_err();
        assert_eq!(err.line, 1);
        assert_eq!(err.column, 5);
        assert!(err.message.contains("unexpected"));
    }

    #[test]
    fn error_mismatched_brackets() {
        let err = format_debug("Foo { bar: 1 )", 4).unwrap_err();
        assert_eq!(err.line, 1);
        assert_eq!(err.column, 14);
        assert!(err.message.contains("mismatched"));
    }

    #[test]
    fn error_on_second_line() {
        let err = format_debug("Foo { x: 1 }\nBar { y: 2", 4).unwrap_err();
        assert_eq!(err.line, 2);
        assert!(err.message.contains("unclosed"));
    }

    #[test]
    fn empty_input() {
        assert_eq!(format_debug("", 4).unwrap(), "");
    }

    #[test]
    fn blank_lines_skipped() {
        let input = "Foo { x: 1 }\n\n\nBar { y: 2 }";
        let output = format_debug(input, 4).unwrap();
        assert_eq!(output, "Foo {\n    x: 1,\n}\nBar {\n    y: 2,\n}");
    }

    #[test]
    fn dbg_prefix_with_array() {
        let input = "[src/main.rs:5:5] items = [1, 2, 3]";
        let output = format_debug(input, 4).unwrap();
        assert_eq!(
            output,
            "[src/main.rs:5:5] items = [\n    1,\n    2,\n    3,\n]"
        );
    }

    #[test]
    fn error_column_offset_with_dbg_prefix() {
        // "[src/main.rs:5:5] x = " is 22 chars, then "Foo { bar: 1" has unclosed '{' at position 5
        let err = format_debug("[src/main.rs:5:5] x = Foo { bar: 1", 4).unwrap_err();
        assert_eq!(err.line, 1);
        assert_eq!(err.column, 27); // 22 (prefix) + 5 ('{' position in value)
        assert!(err.message.contains("unclosed"));
    }

    #[test]
    fn multi_value_with_comma_separator() {
        let input = "Foo { x: 1 }, Bar { y: 2 }";
        let output = format_debug(input, 4).unwrap();
        assert_eq!(output, "Foo {\n    x: 1,\n}\nBar {\n    y: 2,\n}");
    }
}
