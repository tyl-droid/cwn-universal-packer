use anyhow::Result;
use sha2::{Digest, Sha256};
use std::io::Read;

pub fn sha256_reader<R: Read>(reader: &mut R) -> Result<String> {
    let (hash, _) = sha256_reader_counted(reader)?;
    Ok(hash)
}

pub fn sha256_reader_counted<R: Read>(reader: &mut R) -> Result<(String, u64)> {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut total = 0u64;

    loop {
        let read = reader.read(&mut buffer)?;

        if read == 0 {
            break;
        }

        hasher.update(&buffer[..read]);
        total += read as u64;
    }

    Ok((hex::encode(hasher.finalize()), total))
}
