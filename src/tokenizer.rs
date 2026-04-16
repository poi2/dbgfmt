#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    OpenParen,
    CloseParen,
    Comma,
    Colon,
    Text(String),
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            '{' => {
                tokens.push(Token::OpenBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::CloseBrace);
                chars.next();
            }
            '[' => {
                tokens.push(Token::OpenBracket);
                chars.next();
            }
            ']' => {
                tokens.push(Token::CloseBracket);
                chars.next();
            }
            '(' => {
                tokens.push(Token::OpenParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::CloseParen);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            ':' => {
                // Only treat `: ` (colon followed by space) as a field separator.
                // Other colons (e.g. 127.0.0.1:8080, a::b) are part of text.
                chars.next();
                if chars.peek() == Some(&' ') || chars.peek() == Some(&'\t') {
                    tokens.push(Token::Colon);
                    chars.next(); // consume the space after colon
                } else {
                    // Colon is part of a value — merge into adjacent Text token or create new one
                    let mut text = String::from(':');
                    while let Some(&c) = chars.peek() {
                        if c.is_whitespace()
                            || matches!(c, '{' | '}' | '[' | ']' | '(' | ')' | ',' | '"' | '\'')
                        {
                            break;
                        }
                        text.push(c);
                        chars.next();
                    }
                    // Merge with previous Text token if possible
                    if let Some(Token::Text(prev)) = tokens.last_mut() {
                        prev.push_str(&text);
                    } else {
                        tokens.push(Token::Text(text));
                    }
                }
            }
            '\'' => {
                let mut s = String::new();
                s.push('\'');
                chars.next();
                loop {
                    match chars.next() {
                        Some('\\') => {
                            s.push('\\');
                            if let Some(escaped) = chars.next() {
                                s.push(escaped);
                            }
                        }
                        Some('\'') => {
                            s.push('\'');
                            break;
                        }
                        Some(c) => s.push(c),
                        None => break,
                    }
                }
                tokens.push(Token::Text(s));
            }
            '"' => {
                let mut s = String::new();
                s.push('"');
                chars.next();
                loop {
                    match chars.next() {
                        Some('\\') => {
                            s.push('\\');
                            if let Some(escaped) = chars.next() {
                                s.push(escaped);
                            }
                        }
                        Some('"') => {
                            s.push('"');
                            break;
                        }
                        Some(c) => s.push(c),
                        None => break,
                    }
                }
                tokens.push(Token::Text(s));
            }
            c if c.is_whitespace() => {
                chars.next();
            }
            _ => {
                let mut text = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace()
                        || matches!(c, '{' | '}' | '[' | ']' | '(' | ')' | ',' | '"' | '\'')
                    {
                        break;
                    }
                    // Break on `:` only if followed by whitespace (field separator)
                    if c == ':' {
                        let mut lookahead = chars.clone();
                        lookahead.next();
                        if lookahead.peek() == Some(&' ') || lookahead.peek() == Some(&'\t') {
                            break;
                        }
                    }
                    text.push(c);
                    chars.next();
                }
                if !text.is_empty() {
                    tokens.push(Token::Text(text));
                }
            }
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_simple_struct() {
        let tokens = tokenize("Foo { bar: 1 }");
        assert_eq!(
            tokens,
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
    fn tokenize_string_value() {
        let tokens = tokenize(r#"Foo { s: "hello world" }"#);
        assert_eq!(
            tokens,
            vec![
                Token::Text("Foo".into()),
                Token::OpenBrace,
                Token::Text("s".into()),
                Token::Colon,
                Token::Text(r#""hello world""#.into()),
                Token::CloseBrace,
            ]
        );
    }

    #[test]
    fn tokenize_escaped_quote() {
        let tokens = tokenize(r#"Foo { s: "he said \"hi\"" }"#);
        assert_eq!(
            tokens,
            vec![
                Token::Text("Foo".into()),
                Token::OpenBrace,
                Token::Text("s".into()),
                Token::Colon,
                Token::Text(r#""he said \"hi\"""#.into()),
                Token::CloseBrace,
            ]
        );
    }

    #[test]
    fn tokenize_nested() {
        let tokens = tokenize("A { b: B { c: 1 } }");
        assert_eq!(
            tokens,
            vec![
                Token::Text("A".into()),
                Token::OpenBrace,
                Token::Text("b".into()),
                Token::Colon,
                Token::Text("B".into()),
                Token::OpenBrace,
                Token::Text("c".into()),
                Token::Colon,
                Token::Text("1".into()),
                Token::CloseBrace,
                Token::CloseBrace,
            ]
        );
    }

    #[test]
    fn tokenize_array() {
        let tokens = tokenize("[1, 2, 3]");
        assert_eq!(
            tokens,
            vec![
                Token::OpenBracket,
                Token::Text("1".into()),
                Token::Comma,
                Token::Text("2".into()),
                Token::Comma,
                Token::Text("3".into()),
                Token::CloseBracket,
            ]
        );
    }

    #[test]
    fn tokenize_tuple() {
        let tokens = tokenize("(1, 2)");
        assert_eq!(
            tokens,
            vec![
                Token::OpenParen,
                Token::Text("1".into()),
                Token::Comma,
                Token::Text("2".into()),
                Token::CloseParen,
            ]
        );
    }

    #[test]
    fn tokenize_enum_variant() {
        let tokens = tokenize("Some(42)");
        assert_eq!(
            tokens,
            vec![
                Token::Text("Some".into()),
                Token::OpenParen,
                Token::Text("42".into()),
                Token::CloseParen,
            ]
        );
    }

    #[test]
    fn tokenize_char_literal() {
        let tokens = tokenize("Foo { c: '{' }");
        assert_eq!(
            tokens,
            vec![
                Token::Text("Foo".into()),
                Token::OpenBrace,
                Token::Text("c".into()),
                Token::Colon,
                Token::Text("'{'".into()),
                Token::CloseBrace,
            ]
        );
    }

    #[test]
    fn tokenize_escaped_char_literal() {
        let tokens = tokenize(r"Foo { c: '\'' }");
        assert_eq!(
            tokens,
            vec![
                Token::Text("Foo".into()),
                Token::OpenBrace,
                Token::Text("c".into()),
                Token::Colon,
                Token::Text(r"'\''".into()),
                Token::CloseBrace,
            ]
        );
    }

    #[test]
    fn tokenize_socket_addr() {
        let tokens = tokenize("Foo { addr: 127.0.0.1:8080 }");
        assert_eq!(
            tokens,
            vec![
                Token::Text("Foo".into()),
                Token::OpenBrace,
                Token::Text("addr".into()),
                Token::Colon,
                Token::Text("127.0.0.1:8080".into()),
                Token::CloseBrace,
            ]
        );
    }

    #[test]
    fn tokenize_double_colon_path() {
        let tokens = tokenize("Foo { path: std::io::Error }");
        assert_eq!(
            tokens,
            vec![
                Token::Text("Foo".into()),
                Token::OpenBrace,
                Token::Text("path".into()),
                Token::Colon,
                Token::Text("std::io::Error".into()),
                Token::CloseBrace,
            ]
        );
    }

    #[test]
    fn tokenize_empty_input() {
        let tokens = tokenize("");
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn tokenize_negative_number() {
        let tokens = tokenize("Foo { x: -42 }");
        assert_eq!(
            tokens,
            vec![
                Token::Text("Foo".into()),
                Token::OpenBrace,
                Token::Text("x".into()),
                Token::Colon,
                Token::Text("-42".into()),
                Token::CloseBrace,
            ]
        );
    }

    #[test]
    fn tokenize_hashmap() {
        let tokens = tokenize("{1: \"a\", 2: \"b\"}");
        assert_eq!(
            tokens,
            vec![
                Token::OpenBrace,
                Token::Text("1".into()),
                Token::Colon,
                Token::Text("\"a\"".into()),
                Token::Comma,
                Token::Text("2".into()),
                Token::Colon,
                Token::Text("\"b\"".into()),
                Token::CloseBrace,
            ]
        );
    }

    #[test]
    fn tokenize_hashset() {
        let tokens = tokenize("{1, 2, 3}");
        assert_eq!(
            tokens,
            vec![
                Token::OpenBrace,
                Token::Text("1".into()),
                Token::Comma,
                Token::Text("2".into()),
                Token::Comma,
                Token::Text("3".into()),
                Token::CloseBrace,
            ]
        );
    }

    #[test]
    fn tokenize_unicode() {
        let tokens = tokenize("Foo { name: \"太郎\" }");
        assert_eq!(
            tokens,
            vec![
                Token::Text("Foo".into()),
                Token::OpenBrace,
                Token::Text("name".into()),
                Token::Colon,
                Token::Text("\"太郎\"".into()),
                Token::CloseBrace,
            ]
        );
    }
}
