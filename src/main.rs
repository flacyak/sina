mod encode;
mod mode;
pub use encode::{Encoding, encode};
use std::env;

struct Base {
    encoding: Encoding,
    details: String,
}

fn read_error() {
    println!("Invalid command");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let query: &String = &args[1];
    let size: &'static str = &args[2];
    let url_str: &String = &args[3];
    let encode_str: String = encode(url_str.to_string(), size);
    match query.as_str() {
        "gen" => {}
        _ => read_error(),
    }
}
