use std::io::{self, Read};

fn main() {
    let mut input = Vec::<u8>::new();
    io::stdin().read_to_end(&mut input).unwrap();
    println!("{}", bs58::encode(input).into_string());
}
