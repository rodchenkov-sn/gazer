use std::{io, fs};
use std::path::Path;
use std::io::{Write, Read};

use crate::gazer::chunk_provider::ChunkProvider;
use crate::gazer::tar_header::{is_chunk_empty, TarFileHeader};
use crate::gazer::chunk_consumer::ChunkConsumer;

pub fn untar<T: ChunkProvider>(target_path: &String, archive: &mut T) -> Result<(), String> {
    use std::fs::*;
    while let Some(chunk) = archive.get_next_chunk() {
        if is_chunk_empty(&chunk) {
            if let Some(next_chunk) = archive.get_next_chunk(){
                if is_chunk_empty(&next_chunk) {
                    return Ok(());
                }
            }
            return Err(String::from("archive is broken"));
        }
        let header = TarFileHeader::from_chunk(&chunk)?;
        if header.tpe == 53 {
            create_dir(format!("{}{}", target_path, header.name))
                .map_err(|_| String::from("could not create folder"))?;
        } else {
            let mut file = File::create(
                format!("{}{}", target_path, header.name)
            ).map_err(|_| String::from("creation error"))?;
            let mut bytes_left: usize = header.size as usize;
            for _ in 0..header.chunk_count() {
                if let Some(data) = archive.get_next_chunk() {
                    if bytes_left >= 512 {
                        bytes_left -= 512;
                        file.write(&data)
                            .map_err(|_| String::from("could not write in file"))?;
                    } else {
                        file.write(&data[0 .. bytes_left])
                            .map_err(|_| String::from("could not write in file"))?;
                    }
                } else {
                    return Err(String::from("archive is broken"));
                }
            }
        }
    }
    Ok(())
}

fn write_dir<T: ChunkConsumer>(path: &Path, prefix: String, consumer: &mut T) -> io::Result<()> {
    let name = format!(r"{}{}/", prefix, String::from(path.file_name().unwrap().to_str().unwrap()));
    let dir_header = TarFileHeader{
        name: name.clone(),
        mode: "0040777".to_string(),
        owner_id: "0000000".to_string(),
        group_id: "0000000".to_string(),
        size: 0,
        lmt: path.metadata().unwrap().modified().unwrap().elapsed().unwrap().as_millis(),
        tpe: 53,
        link: "".to_string(),
        version: [48, 48]
    };
    consumer.consume_next_chunk(dir_header.to_chunk())?;
    for file in fs::read_dir(path)? {
        let file = file?;
        if file.path().is_dir() {
            write_dir(&file.path(), name.clone(), consumer)?;
        } else {
            write_file(&file.path(), name.clone(), consumer)?;
        }
    }
    Ok(())
}

fn write_file<T: ChunkConsumer>(path: &Path, prefix: String, consumer: &mut T) -> io::Result<()> {
    let name = format!("{}{}", prefix, String::from(path.file_name().unwrap().to_str().unwrap()));
    let file_header = TarFileHeader{
        name,
        mode: "0100777".to_string(),
        owner_id: "0000000".to_string(),
        group_id: "0000000".to_string(),
        size: path.metadata().unwrap().len() as u128,
        lmt: path.metadata().unwrap().modified().unwrap().elapsed().unwrap().as_millis(),
        tpe: 48,
        link: "".to_string(),
        version: [48, 48]
    };
    let mut file = std::fs::File::open(path)?;
    consumer.consume_next_chunk(file_header.to_chunk())?;
    let mut chunk = [0u8; 512];
    for _ in 0 .. file_header.chunk_count() {
        file.read(&mut chunk)?;
        consumer.consume_next_chunk(chunk)?;
    }
    Ok(())
}

pub fn mktar<T: ChunkConsumer>(items: Vec<&Path>, consumer: &mut T) -> Result<(), String> {
    for path in items {
        if path.is_dir() {
            write_dir(path, "".to_string(), consumer)
                .map_err(|_| "bad dir writing".to_string())?;
        } else {
            write_file(path, "".to_string(), consumer)
                .map_err(|_| "bad file writing".to_string())?;
        }
    }
    consumer.done().map_err(|_| "write error".to_string())?;
    Ok(())
}
