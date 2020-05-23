mod header;
mod archive;

use std::env;

use archive::untar;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() != 3 {
        println!("bruh...")
    } else {
        let res = untar(&args[1], &args[2]);
        if res.is_err() {
            println!("{}", res.unwrap_err());
        } else {
            println!("done.");
        }
    }
}
