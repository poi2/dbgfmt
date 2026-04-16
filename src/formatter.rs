use crate::tokenizer::Token;

pub fn format_tokens(tokens: &[Token], indent_width: usize) -> String {
    let mut output = String::new();
    let mut indent_level: usize = 0;
    let indent_unit = " ".repeat(indent_width);
    let len = tokens.len();
    let mut i = 0;

    while i < len {
        match &tokens[i] {
            Token::OpenBrace | Token::OpenBracket | Token::OpenParen => {
                let ch = match &tokens[i] {
                    Token::OpenBrace => '{',
                    Token::OpenBracket => '[',
                    Token::OpenParen => '(',
                    _ => unreachable!(),
                };

                // Check if next token is the matching close delimiter (empty body)
                let is_empty = i + 1 < len
                    && matches!(
                        (&tokens[i], &tokens[i + 1]),
                        (Token::OpenBrace, Token::CloseBrace)
                            | (Token::OpenBracket, Token::CloseBracket)
                            | (Token::OpenParen, Token::CloseParen)
                    );

                if is_empty {
                    let close_ch = match ch {
                        '{' => '}',
                        '[' => ']',
                        '(' => ')',
                        _ => unreachable!(),
                    };
                    output.push(ch);
                    output.push(close_ch);
                    i += 2; // skip both open and close
                    continue;
                }

                // Check if paren contains a single value with no commas (e.g. Some(42))
                // Keep inline as Some(42) instead of expanding to multiple lines
                if matches!(&tokens[i], Token::OpenParen) {
                    if let Some(single) = single_paren_value(&tokens[i..]) {
                        output.push('(');
                        output.push_str(&single);
                        output.push(')');
                        // Skip open paren + value + optional trailing comma + close paren
                        let mut skip = 1; // skip open paren (already at i)
                        skip += 1; // skip value
                        if i + skip < len && matches!(tokens[i + skip], Token::Comma) {
                            skip += 1; // skip trailing comma
                        }
                        skip += 1; // skip close paren
                        i += skip;
                        continue;
                    }
                }

                output.push(ch);
                indent_level += 1;
                output.push('\n');
                push_indent(&mut output, &indent_unit, indent_level);
            }
            Token::CloseBrace | Token::CloseBracket | Token::CloseParen => {
                let ch = match &tokens[i] {
                    Token::CloseBrace => '}',
                    Token::CloseBracket => ']',
                    Token::CloseParen => ')',
                    _ => unreachable!(),
                };
                // Add trailing comma if the previous token wasn't a comma
                if i > 0
                    && !matches!(
                        tokens[i - 1],
                        Token::Comma | Token::OpenBrace | Token::OpenBracket | Token::OpenParen
                    )
                {
                    output.push(',');
                }
                indent_level = indent_level.saturating_sub(1);
                output.push('\n');
                push_indent(&mut output, &indent_unit, indent_level);
                output.push(ch);
            }
            Token::Comma => {
                output.push(',');
                // Skip newline if next token is a close delimiter (input already has trailing comma)
                let next_is_close = i + 1 < len
                    && matches!(
                        tokens[i + 1],
                        Token::CloseBrace | Token::CloseBracket | Token::CloseParen
                    );
                if !next_is_close {
                    output.push('\n');
                    push_indent(&mut output, &indent_unit, indent_level);
                }
            }
            Token::Colon => {
                output.push_str(": ");
            }
            Token::Text(text) => {
                output.push_str(text);
                // Add space before { only (not before ( or [)
                if i + 1 < len && matches!(tokens[i + 1], Token::OpenBrace) {
                    output.push(' ');
                }
            }
        }
        i += 1;
    }

    output
}

/// Check if a paren group contains exactly one simple text value.
/// Returns the text if so, None otherwise.
/// Matches: `(value)` or `(value,)` where value is a single Text token.
fn single_paren_value(tokens: &[Token]) -> Option<String> {
    if tokens.len() < 3 {
        return None;
    }
    if !matches!(tokens[0], Token::OpenParen) {
        return None;
    }
    let value = match &tokens[1] {
        Token::Text(s) => s.clone(),
        _ => return None,
    };
    // (value) or (value,)
    match tokens.get(2) {
        Some(Token::CloseParen) => Some(value),
        Some(Token::Comma) => {
            if matches!(tokens.get(3), Some(Token::CloseParen)) {
                Some(value)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn push_indent(output: &mut String, indent_unit: &str, level: usize) {
    for _ in 0..level {
        output.push_str(indent_unit);
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::tokenize;

    use super::*;

    fn fmt(input: &str) -> String {
        let tokens = tokenize(input);
        format_tokens(&tokens, 2)
    }

    #[test]
    fn format_simple_struct() {
        assert_eq!(
            fmt("Foo { bar: 1, baz: 2 }"),
            "\
Foo {
  bar: 1,
  baz: 2,
}"
        );
    }

    #[test]
    fn format_nested_struct() {
        assert_eq!(
            fmt("Foo { bar: 1, inner: Bar { x: 2 } }"),
            "\
Foo {
  bar: 1,
  inner: Bar {
    x: 2,
  },
}"
        );
    }

    #[test]
    fn format_array() {
        assert_eq!(
            fmt("[1, 2, 3]"),
            "\
[
  1,
  2,
  3,
]"
        );
    }

    #[test]
    fn format_empty_struct() {
        assert_eq!(fmt("Foo {}"), "Foo {}");
    }

    #[test]
    fn format_empty_array() {
        assert_eq!(fmt("[]"), "[]");
    }

    #[test]
    fn format_enum_some() {
        // Single value in parens stays inline
        assert_eq!(fmt("Some(42)"), "Some(42)");
    }

    #[test]
    fn format_enum_some_with_trailing_comma() {
        // Single value with trailing comma stays inline
        assert_eq!(fmt("Some(42,)"), "Some(42)");
    }

    #[test]
    fn format_deeply_nested() {
        assert_eq!(
            fmt("A { b: B { c: C { d: 1 } } }"),
            "\
A {
  b: B {
    c: C {
      d: 1,
    },
  },
}"
        );
    }

    #[test]
    fn format_mixed_delimiters() {
        assert_eq!(
            fmt("Foo { items: [1, 2], pair: (3, 4) }"),
            "\
Foo {
  items: [
    1,
    2,
  ],
  pair: (
    3,
    4,
  ),
}"
        );
    }

    #[test]
    fn format_with_string_value() {
        assert_eq!(
            fmt(r#"Foo { name: "hello", count: 5 }"#),
            "\
Foo {
  name: \"hello\",
  count: 5,
}"
        );
    }

    #[test]
    fn format_single_value() {
        assert_eq!(fmt("42"), "42");
    }

    #[test]
    fn format_none() {
        assert_eq!(fmt("None"), "None");
    }

    #[test]
    fn format_trailing_comma_in_input() {
        // Single value with trailing comma stays inline
        assert_eq!(fmt("Foo(1,)"), "Foo(1)");
    }

    #[test]
    fn format_multi_value_paren() {
        // Multiple values in parens still expand
        assert_eq!(
            fmt("Foo(1, 2)"),
            "\
Foo(
  1,
  2,
)"
        );
    }

    #[test]
    fn format_hashmap() {
        assert_eq!(
            fmt("{1: \"a\", 2: \"b\"}"),
            "\
{
  1: \"a\",
  2: \"b\",
}"
        );
    }

    #[test]
    fn format_hashset() {
        assert_eq!(
            fmt("{1, 2, 3}"),
            "\
{
  1,
  2,
  3,
}"
        );
    }

    #[test]
    fn format_unicode() {
        assert_eq!(
            fmt("Foo { name: \"太郎\", emoji: \"🦀\" }"),
            "\
Foo {
  name: \"太郎\",
  emoji: \"🦀\",
}"
        );
    }

    #[test]
    fn format_indent_zero() {
        let tokens = tokenize("Foo { x: 1 }");
        assert_eq!(
            format_tokens(&tokens, 0),
            "\
Foo {
x: 1,
}"
        );
    }

    #[test]
    fn format_custom_indent() {
        let tokens = tokenize("Foo { x: 1 }");
        assert_eq!(
            format_tokens(&tokens, 4),
            "\
Foo {
    x: 1,
}"
        );
    }
}
