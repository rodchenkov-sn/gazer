use std::convert::TryInto;

pub struct FileHeader {
    pub name:       String,
    pub mode:       String,
    pub owner_id:   String,
    pub group_id:   String,
    pub size:       u64,
    pub lmt:        u64,
    pub sum:        u64,
    pub tpe:        u8,
    pub link:       String,
    pub version:    [u8; 2],
    pub owner_name: String,
    pub group_name: String,
    pub dev_major:  u64,
    pub dev_minor:  u64,
}

pub fn is_chunk_empty(chunk: &[u8; 512]) -> bool {
    chunk.iter().all(|&x| x == 0)
}

fn oct_to_dec(oct: &[u8]) -> u64 {
    let mut val = 0u64;
    for &digit in oct {
        if digit >= 48 {
            val = (digit as u64 - 48) + val * 8;
        }
    }
    val
}

fn utf_to_str(utf: &[u8]) -> Result<String, String> {
    std::str::from_utf8(
        &utf.iter().filter(|&&i| i != 0).cloned().collect::<Vec<u8>>())
        .map(|s| String::from(s))
        .map_err(|_| String::from("Utf-8 error")
        )
}

impl FileHeader {

    pub fn from_chunk(chunk: &[u8; 512]) -> Result<FileHeader, String> {
        if utf_to_str(&chunk[257 .. 262])? != String::from("ustar") {
            return Err(String::from("bad magic..."));
        }
        let required_checksum =
            chunk[0 .. 148].iter().fold(0u64, |i, &c| i + c as u64)
                + 8 * 32
                + chunk[156 .. 512].iter().fold(0u64, |i, &c| i + c as u64);
        let actual_checksum = oct_to_dec(&chunk[148 .. 156]);
        if required_checksum != actual_checksum {
            return Err(String::from("Checksum error"));
        }
        let file_name = format!("{}{}",
                                utf_to_str(&chunk[345 .. 500])?,
                                utf_to_str(&chunk[0 .. 100])?);
        Ok(FileHeader{
            name:       file_name,
            mode:       utf_to_str(&chunk[100 .. 108])?,
            owner_id:   utf_to_str(&chunk[108 .. 116])?,
            group_id:   utf_to_str(&chunk[116 .. 124])?,
            size:       oct_to_dec(&chunk[124 .. 136]),
            lmt:        oct_to_dec(&chunk[136 .. 148]),
            sum:        actual_checksum,
            tpe:        chunk[156],
            link:       String::from(""),
            version:    chunk[263 .. 265].try_into().expect("bruh..."),
            owner_name: utf_to_str(&chunk[265 .. 297])?,
            group_name: utf_to_str(&chunk[297 .. 329])?,
            dev_major:  oct_to_dec(&chunk[329 .. 337]),
            dev_minor:  oct_to_dec(&chunk[329 .. 337]),
        })
    }

    pub fn chunk_count(&self) -> u64 {
        self.size / 512 + if self.size % 512 == 0 { 0 } else { 1 }
    }

}