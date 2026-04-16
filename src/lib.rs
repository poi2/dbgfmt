mod formatter;
mod tokenizer;

pub fn format_debug(input: &str, indent_width: usize) -> String {
    let tokens = tokenizer::tokenize(input);
    formatter::format_tokens(&tokens, indent_width)
}
