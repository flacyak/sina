//! sina: generate a QR code and print it to the terminal.
mod ecc;
mod encode;
mod matrix;
mod mode;

use matrix::Matrix;
use mode::EC;
use std::io::{self, Read};
use std::process::ExitCode;

/// Light margin around the symbol, in modules, as required by the spec.
const QUIET_ZONE: usize = 4;
const USAGE: &str = "Usage: sina [OPTIONS] [TEXT]

Generate a QR code and print it to the terminal.
TEXT is read from stdin when omitted.

Options:
  -e, --ec <l|m|q|h>  Error correction level (default: m)
  -p, --plain         Print without ANSI colors
  -h, --help          Show this help";

struct Options {
    text: Option<String>,
    ec: EC,
    plain: bool,
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Options, String> {
    let mut opts = Options { text: None, ec: EC::M, plain: false };
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            "-p" | "--plain" => opts.plain = true,
            "-e" | "--ec" => {
                let level = args.next().ok_or("missing value for --ec")?;
                opts.ec = level.parse()?;
            }
            _ if arg.starts_with('-') && arg.len() > 1 => {
                return Err(format!("unknown option '{arg}'"));
            }
            _ if opts.text.is_some() => return Err("expected a single TEXT argument".into()),
            _ => opts.text = Some(arg),
        }
    }
    Ok(opts)
}

fn read_stdin() -> Result<String, String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).map_err(|e| format!("failed to read stdin: {e}"))?;
    Ok(input.trim_end_matches(['\r', '\n']).to_string())
}

/// Renders two module rows per text line using half-block characters.
fn render(m: &Matrix, plain: bool) -> String {
    let span = m.size + 2 * QUIET_ZONE;
    let dark = |row: usize, col: usize| {
        let inside = |v: usize| (QUIET_ZONE..QUIET_ZONE + m.size).contains(&v);
        inside(row) && inside(col) && m.get(row - QUIET_ZONE, col - QUIET_ZONE)
    };
    let mut out = String::with_capacity(span * span);
    for top in (0..span).step_by(2) {
        if !plain {
            out.push_str("\x1b[30;47m");
        }
        for col in 0..span {
            out.push(match (dark(top, col), dark(top + 1, col)) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            });
        }
        if !plain {
            out.push_str("\x1b[0m");
        }
        out.push('\n');
    }
    out
}

fn run() -> Result<(), String> {
    let opts = parse_args(std::env::args().skip(1))?;
    let text = match opts.text {
        Some(text) => text,
        None => read_stdin()?,
    };
    if text.is_empty() {
        return Err("nothing to encode".into());
    }
    let encoded = encode::encode(&text, opts.ec)?;
    let codewords = ecc::add_error_correction(&encoded.codewords, encoded.version, encoded.ec);
    print!("{}", render(&Matrix::build(encoded.version, encoded.ec, &codewords), opts.plain));
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("sina: {message}");
            ExitCode::FAILURE
        }
    }
}
