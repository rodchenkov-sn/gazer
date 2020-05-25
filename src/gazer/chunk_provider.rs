pub trait ChunkProvider {
    fn get_next_chunk(&mut self) -> Option<[u8; 512]>;
}
