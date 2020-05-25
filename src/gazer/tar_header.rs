use std::convert::TryInto;

pub struct TarFileHeader {
    pub name:       String,
    pub mode:       String,
    pub owner_id:   String,
    pub group_id:   String,
    pub size:       u128,
    pub lmt:        u128,
    pub tpe:        u8,
    pub link:       String,
    pub version:    [u8; 2],
}

pub fn is_chunk_empty(chunk: &[u8; 512]) -> bool {
    chunk.iter().all(|&x| x == 0)
}

fn oct_to_dec(oct: &[u8]) -> u128 {
    let mut val = 0;
    for &digit in oct {
        if digit >= 48 {
            val = (digit as u128 - 48) + val * 8;
        }
    }
    val
}

fn dec_to_oct(dec: u128) -> [u8; 12] {
    let mut dec = dec;
    let mut oct = [0u8; 12];
    for i in (0..10).rev() {
        oct[i] = (dec % 8 + 48).try_into().expect("bruh...");
        dec /= 8;
    }
    oct
}

fn utf_to_str(utf: &[u8]) -> Result<String, String> {
    std::str::from_utf8(
        &utf.iter().filter(|&&i| i != 0).cloned().collect::<Vec<u8>>())
        .map(|s| String::from(s))
        .map_err(|_| String::from("Utf-8 error")
        )
}

impl TarFileHeader {

    pub fn from_chunk(chunk: &[u8; 512]) -> Result<TarFileHeader, String> {
        if utf_to_str(&chunk[257 .. 262])? != String::from("ustar") {
            return Err(String::from("bad magic..."));
        }
        let required_checksum =
            chunk[0 .. 148].iter().fold(0, |i, &c| i + c as u128)
                + 8 * 32
                + chunk[156 .. 512].iter().fold(0, |i, &c| i + c as u128);
        let actual_checksum = oct_to_dec(&chunk[148 .. 156]);
        if required_checksum != actual_checksum {
            return Err(String::from("Checksum error"));
        }
        let file_name = format!("{}{}",
                                utf_to_str(&chunk[345 .. 500])?,
                                utf_to_str(&chunk[0 .. 100])?);
        Ok(TarFileHeader{
            name:       file_name,
            mode:       utf_to_str(&chunk[100 .. 108])?,
            owner_id:   utf_to_str(&chunk[108 .. 116])?,
            group_id:   utf_to_str(&chunk[116 .. 124])?,
            size:       oct_to_dec(&chunk[124 .. 136]),
            lmt:        oct_to_dec(&chunk[136 .. 148]),
            tpe:        chunk[156],
            link:       String::from(""),
            version:    chunk[263 .. 265].try_into().unwrap(),
        })
    }

    pub fn chunk_count(&self) -> u128 {
        self.size / 512 + if self.size % 512 == 0 { 0 } else { 1 }
    }

    pub fn to_chunk(&self) -> [u8; 512] {
        let mut chunk = vec![0u8; 512];
        let name_utf = self.name.clone().into_bytes();
        if name_utf.len() <= 100 {
            chunk.splice(0 .. 100, name_utf);
        } else {
            let prefix_len = name_utf.len() - 100;
            chunk.splice(0 .. 100, name_utf.iter().skip(prefix_len).cloned().collect::<Vec<u8>>());
            chunk.splice(345 .. 500, name_utf.iter().take(prefix_len).cloned().collect::<Vec<u8>>());
        }
        chunk.splice(100 .. 108, self.mode.clone().into_bytes().iter().cloned().collect::<Vec<u8>>());
        chunk.splice(108 .. 116, self.owner_id.clone().into_bytes().iter().cloned().collect::<Vec<u8>>());
        chunk.splice(116 .. 124, self.group_id.clone().into_bytes().iter().cloned().collect::<Vec<u8>>());
        chunk.splice(124 .. 136, dec_to_oct(self.size).iter().cloned().collect::<Vec<u8>>());
        chunk.splice(136 .. 148, dec_to_oct(self.lmt).iter().cloned().collect::<Vec<u8>>());
        chunk[156] = self.tpe;
        chunk.splice(257 .. 262, String::from("ustar").into_bytes().iter().cloned().collect::<Vec<u8>>());
        chunk.splice(263 .. 265, [48; 2].iter().cloned().collect::<Vec<u8>>());

        let checksum = chunk.iter().fold(0, |s, &c| s + c as u128) + 32 * 8;
        chunk.splice(148 .. 156, dec_to_oct(checksum).iter().skip(4).cloned().collect::<Vec<u8>>());
        let mut mem = [0u8; 512];
        for i in 0..chunk.len() {
            mem[i] = chunk[i];
        }
        mem
    }

}