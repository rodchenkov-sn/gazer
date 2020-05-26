mod gazer;

use std::fs::{File, OpenOptions};
use flate2::write::GzEncoder;
use std::path::{Path, PathBuf};
use flate2::{Compression, GzBuilder};

use gazer::tar_archive::*;
use flate2::read::GzDecoder;
use crate::gazer::tar_file::TarFile;
use std::io::{Read, Write, Seek, SeekFrom};

use clap::{App, load_yaml, Values};
use std::error::Error;

fn main() {
    let cli = load_yaml!("../res/cli.yml");
    let matches = App::from(cli).get_matches();
    let verbose = matches.is_present("verbose");
    let compress = matches.is_present("compress");

    let create = matches.values_of("create");
    let extract = matches.values_of("extract");
    let append = matches.values_of("append");
    let input = matches.values_of("INPUT");
    let list = matches.values_of("list");

    if let Some(output) = create {
        do_create(input, output, compress, verbose);
    } else if let Some(archive) = extract {
        do_extract(archive, verbose);
    } else if let Some(output) = append {
        do_append(input, output, verbose);
    } else if let Some(archive) = list {
        do_list(archive);
    } else {
        println!("Use gazer --help for help");
    }
}

fn do_create(mut input: Option<Values>, mut output: Values, compress: bool, verbose: bool) -> Option<()> {
    let mut input = input?;
    let mut input_paths: Vec<&Path> = Vec::new();
    let output_path = Path::new(output.next()?);
    let mut output_file = File::create(output_path).expect("bruh");
    while let Some(path) = input.next() {
        input_paths.push(Path::new(path));
    }
   if compress {
       let mut cons = GzEncoder::new(&mut output_file, Compression::default());
       match mktar::<GzEncoder<&mut File>>(input_paths, &mut cons, verbose) {
           Err(msg) => println!("Error: {}", msg),
           Ok(_) => {}
       }
   } else {
       let mut cons = TarFile::new(output_file);
       match mktar::<TarFile>(input_paths, &mut cons, verbose) {
           Err(msg) => println!("Error: {}", msg),
           Ok(_) => {}
       }
   }
    Some(())
}

fn do_extract(mut archive: Values, verbose: bool) -> Option<()> {
    let input = String::from(archive.next()?);
    let input_path = PathBuf::from(input.as_str());
    let mut output_path = input_path.clone();
    output_path.pop();
    let output = String::from(output_path.to_str()?) + "\\";
    let mut input_file =File::open(input.as_str()).expect("bruh");
    if input.ends_with("tar") {
        let mut prov = TarFile::new(input_file);
        match untar::<TarFile>(&output, &mut prov, verbose) {
            Err(msg) => println!("Error: {}", msg),
            Ok(_) => {}
        }
    } else {
        let mut prov = GzDecoder::new(&mut input_file);
        match untar::<GzDecoder<&mut File>>(&output, &mut prov, verbose) {
            Err(msg) => println!("Error: {}", msg),
            Ok(_) => {}
        }
    }
    Some(())
}

fn do_append(input: Option<Values>, mut output: Values, verbose: bool) -> Option<()> {
    let output_str = String::from(output.next()?);
    let mut output_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(output_str.as_str()).expect("bruh");
    let mut perm = output_file.metadata().unwrap().permissions();
    let file_len = output_file.metadata().unwrap().len();
    if file_len >= 1024 {
        match output_file.set_len(file_len - 1024) {
            Ok(_) => println!("ok"),
            Err(error) => println!("{}", error.to_string())
        }
    }
    output_file.seek(SeekFrom::End(0));
    let mut input = input?;
    let mut input_paths: Vec<&Path> = Vec::new();
    while let Some(path) = input.next() {
        input_paths.push(Path::new(path));
    }
    let mut cons = TarFile::new(output_file);
    match mktar::<TarFile>(input_paths, &mut cons, verbose) {
        Err(msg) => println!("Error: {}", msg),
        Ok(_) => {}
    }
    Some(())
}

fn do_list(mut archive: Values) -> Option<()> {
    let mut traversed = File::open(archive.next()?).expect("bruh");
    let mut tar_archive = TarFile::new(traversed);
    match traverse(&mut tar_archive) {
        Err(msg) => println!("Error: {}", msg),
        Ok(_) => {}
    }
    Some(())
}

fn summarize(result: Option<()>) {
    match result {
        Some(_) => println!("Done"),
        _ => println!("Aborted due to errors.")
    }
}
