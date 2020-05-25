use crate::gazer::chunk_provider::ChunkProvider;
use crate::gazer::chunk_consumer::ChunkConsumer;
use std::io::{Read, Write};
use std::io;

pub struct TarFile {
    archive: std::fs::File
}

impl TarFile {
    pub fn new(archive: std::fs::File) -> TarFile {
        TarFile{
            archive
        }
    }
}

impl ChunkProvider for TarFile {
    fn get_next_chunk(&mut self) -> Option<[u8; 512]> {
        let mut buf = [0u8; 512];
        if self.archive.read(&mut buf).is_ok() {
            Some(buf)
        } else {
            None
        }
    }
}

impl ChunkConsumer for TarFile {
    fn consume_next_chunk(&mut self, chunk: [u8; 512]) -> io::Result<()> {
        self.archive.write(&chunk)?;
        Ok(())
    }

    fn done(&mut self) -> io::Result<()> {
        self.archive.write(&[0u8; 1024])?;
        Ok(())
    }
}
