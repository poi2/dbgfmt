use crate::formatter::{Emitter, format_tokens_with_emitter};
use crate::tokenizer::Token;

const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";

pub struct ColorEmitter;

impl Emitter for ColorEmitter {
    fn emit_bracket(&mut self, output: &mut String, ch: char) {
        output.push_str(BOLD);
        output.push(ch);
        output.push_str(RESET);
    }

    fn emit_type_name(&mut self, output: &mut String, text: &str) {
        output.push_str(CYAN);
        output.push_str(text);
        output.push_str(RESET);
    }

    fn emit_key(&mut self, output: &mut String, text: &str) {
        output.push_str(GREEN);
        output.push_str(text);
        output.push_str(RESET);
    }

    fn emit_value(&mut self, output: &mut String, text: &str) {
        if text.starts_with('"') || text.starts_with('\'') {
            output.push_str(YELLOW);
            output.push_str(text);
            output.push_str(RESET);
        } else {
            output.push_str(text);
        }
    }

    fn emit_punctuation(&mut self, output: &mut String, text: &str) {
        output.push_str(DIM);
        output.push_str(text);
        output.push_str(RESET);
    }
}

pub fn format_tokens_colored(tokens: &[Token], indent_width: usize) -> String {
    format_tokens_with_emitter(tokens, indent_width, &mut ColorEmitter)
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
