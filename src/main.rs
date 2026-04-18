use std::io::{self, IsTerminal, Read};

struct Options {
    indent_width: usize,
    color: ColorMode,
    input: Option<String>,
}

enum ColorMode {
    Auto,
    Always,
    Never,
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

    let output = if use_color {
        dbgfmt::format_debug_colored(input.trim(), opts.indent_width)
    } else {
        dbgfmt::format_debug(input.trim(), opts.indent_width)
    };
    println!("{output}");
}

fn parse_args() -> Result<Options, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut indent_width: usize = 2;
    let mut color = ColorMode::Auto;
    let mut positional = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
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
                indent_width = val
                    .parse()
                    .map_err(|_| format!("invalid indent value: {val}"))?;
            }
            "--color" => {
                i += 1;
                let val = args.get(i).ok_or("--color requires a value")?;
                color = match val.as_str() {
                    "auto" => ColorMode::Auto,
                    "always" => ColorMode::Always,
                    "never" => ColorMode::Never,
                    _ => {
                        return Err(format!(
                            "invalid color value: {val} (expected auto, always, never)"
                        ));
                    }
                };
            }
            arg if arg.starts_with("--indent=") => {
                let val = &arg["--indent=".len()..];
                indent_width = val
                    .parse()
                    .map_err(|_| format!("invalid indent value: {val}"))?;
            }
            arg if arg.starts_with("--color=") => {
                let val = &arg["--color=".len()..];
                color = match val {
                    "auto" => ColorMode::Auto,
                    "always" => ColorMode::Always,
                    "never" => ColorMode::Never,
                    _ => {
                        return Err(format!(
                            "invalid color value: {val} (expected auto, always, never)"
                        ));
                    }
                };
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
        input,
    })
}

fn print_usage() {
    eprintln!(
        "\
Usage: dbgfmt [OPTIONS] [INPUT]

Arguments:
  [INPUT]    Rust Debug format string to pretty-print. If omitted, reads from stdin.

Options:
  -i, --indent <N>     Indent width (default: 2)
      --color <WHEN>    Color output: auto, always, never (default: auto)
  -h, --help           Print help
  -V, --version        Print version"
    );
}
