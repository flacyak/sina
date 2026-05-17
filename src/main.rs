mod encode;
pub use encode::{Encoding, smallest_encode};
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
    let url_str: &String = &args[2];
    let base_type: Encoding = smallest_encode(url_str.to_string());
    match query.as_str() {
        "gen" => {}
        _ => read_error(),
    }
}
