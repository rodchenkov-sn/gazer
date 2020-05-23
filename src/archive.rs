use std::io::{Write, Read};
use crate::header::{is_chunk_empty, FileHeader};

trait ChunkSequence {
    fn get_next_chunk(&mut self) -> Option<[u8; 512]>;
}

struct TarArchive {
    archive: std::fs::File
}

impl TarArchive {
    fn new(archive: std::fs::File) -> TarArchive {
        TarArchive{
            archive
        }
    }
}

impl ChunkSequence for TarArchive {
    fn get_next_chunk(&mut self) -> Option<[u8; 512]> {
        let mut buf = [0u8; 512];
        if self.archive.read(&mut buf).is_ok() {
            Some(buf)
        } else {
            None
        }
    }
}

pub fn untar(archive_path: &String, target_path: &String) -> Result<(), String> {
    use std::fs::*;
    let mut archive =
        TarArchive::new(File::open(archive_path).map_err(|_| String::from("File opening error"))?);
    while let Some(chunk) = archive.get_next_chunk() {
        if is_chunk_empty(&chunk) {
            return Ok(());
        }
        let header = FileHeader::from_chunk(&chunk)?;
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
