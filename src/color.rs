const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";

use crate::tokenizer::Token;

pub fn format_tokens_colored(tokens: &[Token], indent_width: usize) -> String {
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
                    push_bracket(&mut output, ch);
                    push_bracket(&mut output, close_ch);
                    i += 2;
                    continue;
                }

                if matches!(&tokens[i], Token::OpenParen) {
                    if let Some(single) = single_paren_value(&tokens[i..]) {
                        push_bracket(&mut output, '(');
                        push_value(&mut output, &single);
                        push_bracket(&mut output, ')');
                        let mut skip = 2;
                        if i + skip < len && matches!(tokens[i + skip], Token::Comma) {
                            skip += 1;
                        }
                        skip += 1;
                        i += skip;
                        continue;
                    }
                }

                push_bracket(&mut output, ch);
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
                if i > 0
                    && !matches!(
                        tokens[i - 1],
                        Token::Comma | Token::OpenBrace | Token::OpenBracket | Token::OpenParen
                    )
                {
                    push_dim(&mut output, ",");
                }
                indent_level = indent_level.saturating_sub(1);
                output.push('\n');
                push_indent(&mut output, &indent_unit, indent_level);
                push_bracket(&mut output, ch);
            }
            Token::Comma => {
                push_dim(&mut output, ",");
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
                push_dim(&mut output, ": ");
            }
            Token::Text(text) => {
                // Determine role: type name, key, or value
                if i + 1 < len && matches!(tokens[i + 1], Token::OpenBrace) {
                    // Type name (before {)
                    push_type_name(&mut output, text);
                    output.push(' ');
                } else if i + 1 < len && matches!(tokens[i + 1], Token::Colon) {
                    // Key (before :)
                    push_key(&mut output, text);
                } else if i + 1 < len
                    && matches!(tokens[i + 1], Token::OpenParen | Token::OpenBracket)
                {
                    // Type name (before ( or [)
                    push_type_name(&mut output, text);
                } else {
                    // Value
                    push_value(&mut output, text);
                }
            }
        }
        i += 1;
    }

    output
}

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

fn push_bracket(output: &mut String, ch: char) {
    output.push_str(BOLD);
    output.push(ch);
    output.push_str(RESET);
}

fn push_type_name(output: &mut String, text: &str) {
    output.push_str(CYAN);
    output.push_str(text);
    output.push_str(RESET);
}

fn push_key(output: &mut String, text: &str) {
    output.push_str(GREEN);
    output.push_str(text);
    output.push_str(RESET);
}

fn push_value(output: &mut String, text: &str) {
    if text.starts_with('"') || text.starts_with('\'') {
        output.push_str(YELLOW);
        output.push_str(text);
        output.push_str(RESET);
    } else {
        output.push_str(text);
    }
}

fn push_dim(output: &mut String, text: &str) {
    output.push_str(DIM);
    output.push_str(text);
    output.push_str(RESET);
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::tokenize;

    use super::*;

    #[test]
    fn colored_output_contains_ansi_codes() {
        let tokens = tokenize("Foo { bar: 1 }");
        let output = format_tokens_colored(&tokens, 2);
        assert!(output.contains("\x1b["));
    }

    #[test]
    fn colored_simple_struct() {
        let tokens = tokenize("Foo { bar: 1 }");
        let output = format_tokens_colored(&tokens, 2);
        // Type name should be cyan
        assert!(output.contains(&format!("{CYAN}Foo{RESET}")));
        // Key should be green
        assert!(output.contains(&format!("{GREEN}bar{RESET}")));
    }

    #[test]
    fn colored_string_value() {
        let tokens = tokenize(r#"Foo { s: "hello" }"#);
        let output = format_tokens_colored(&tokens, 2);
        // String value should be yellow
        assert!(output.contains(&format!("{YELLOW}\"hello\"{RESET}")));
    }

    #[test]
    fn colored_non_string_value() {
        let tokens = tokenize("Foo { x: 42 }");
        let output = format_tokens_colored(&tokens, 2);
        // Numeric value should not have color
        assert!(output.contains("42"));
        assert!(!output.contains(&format!("{YELLOW}42{RESET}")));
    }
}
