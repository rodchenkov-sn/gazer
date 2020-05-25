mod gazer;

use std::fs::File;
use flate2::write::GzEncoder;
use std::path::Path;
use flate2::{Compression, GzBuilder};

use gazer::tar_archive::*;
use flate2::read::GzDecoder;
use crate::gazer::tar_file::TarFile;
use std::io::{Read, Write};

fn print_help() {
    println!("usage: gazer [-c -x] ARCHIVE FILE...");
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() >= 4 {
        match args[1].as_str() {
            "-c" => {
                let mut dst_file = File::create(&args[2]).expect("bad input");
                let src = args
                    .iter()
                    .skip(3)
                    .map(|s| Path::new(&**s))
                    .collect::<Vec<&Path>>();
                if args[2].ends_with(".tar") {
                    let file = File::create(&args[2]).expect("bad input");
                    let mut cons = TarFile::new(file);
                    match mktar::<TarFile>(src, &mut cons) {
                        Ok(_) => println!("Done"),
                        Err(msg) => println!("Error: {}", msg)
                    }
                } else if args[2].ends_with(".tgz") || args[2].ends_with(".tar.gz") {
                    let mut cons = GzEncoder::new(&mut dst_file, Compression::fast());
                    match mktar::<GzEncoder<&mut File>>(src, &mut cons) {
                        Ok(_) => println!("Done"),
                        Err(msg) => println!("Error: {}", msg)
                    }
                } else {
                    println!("tarball expected as archive");
                }
                return;
            }
            "-x" => {
                let mut src_file = File::open(&args[2]).expect("bad input");
                let dst_folder = args[3].clone();
                if !dst_folder.ends_with("/") && !dst_folder.ends_with("\\") {
                    println!("folder expected as destination");
                }
                if args[2].ends_with(".tar") {
                    let mut src = TarFile::new(src_file);
                    match untar::<TarFile>(&dst_folder, &mut src) {
                        Ok(_) => println!("Done"),
                        Err(msg) => println!("Error: {}", msg)
                    }
                } else if args[2].ends_with(".tgz") || args[2].ends_with(".tar.gz") {
                    let mut src = GzDecoder::new(&mut src_file);
                    match untar::<GzDecoder<&mut File>>(&dst_folder, &mut src) {
                        Ok(_) => println!("Done"),
                        Err(msg) => println!("Error: {}", msg)
                    }
                } else {
                    println!("tarball expected as archive");
                }
                return;
            }
            _ => print_help()
        }
    }
    print_help();
}
