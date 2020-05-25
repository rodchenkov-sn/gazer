use std::io;

pub trait ChunkConsumer {
    fn consume_next_chunk(&mut self, chunk: [u8; 512]) -> io::Result<()>;
    fn done(&mut self) -> io::Result<()>;
}
