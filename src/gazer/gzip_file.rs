use std::io::prelude::*;
use std::io;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::fs::File;
use crate::gazer::chunk_provider::ChunkProvider;
use crate::gazer::chunk_consumer::ChunkConsumer;

impl ChunkProvider for GzDecoder<&mut File> {
    fn get_next_chunk(&mut self) -> Option<[u8; 512]> {
        let mut chunk = [0u8; 512];
        match self.read_exact(&mut chunk) {
            Ok(_) => Some(chunk),
            Err(_) => None
        }
    }
}

impl ChunkConsumer for GzEncoder<&mut File> {
    fn consume_next_chunk(&mut self, chunk: [u8; 512]) -> io::Result<()> {
        self.write_all(&chunk).map(|_| ())
    }

    fn done(&mut self) -> io::Result<()> {
        self.try_finish().map(|_| ())
    }
}
