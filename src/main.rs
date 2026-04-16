use std::io::{self, IsTerminal, Read};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return;
    }

    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("rust-dbg-fmt {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let input = if !args.is_empty() {
        args.join(" ")
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

    let output = rust_dbg_fmt::format_debug(input.trim(), 2);
    println!("{output}");
}

fn print_usage() {
    eprintln!(
        "\
Usage: rust-dbg-fmt [OPTIONS] [INPUT]

Arguments:
  [INPUT]    Rust Debug format string to pretty-print. If omitted, reads from stdin.

Options:
  -h, --help     Print help
  -V, --version  Print version"
    );
}
