use anyhow::{Context, Result, bail};
use std::io::{Read, Seek, SeekFrom, Write};

pub const CWN_MAGIC: [u8; 4] = *b"CWN1";
pub const CWN_VERSION: u16 = 1;

#[derive(Debug, Clone)]
pub struct CwnHeader {
    pub version: u16,
    pub flags: u16,
    pub manifest_offset: u64,
    pub manifest_size: u64,
    pub entry_count: u64,
}

impl CwnHeader {
    pub fn placeholder() -> Self {
        Self {
            version: CWN_VERSION,
            flags: 0,
            manifest_offset: 0,
            manifest_size: 0,
            entry_count: 0,
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&CWN_MAGIC)?;
        writer.write_all(&self.version.to_le_bytes())?;
        writer.write_all(&self.flags.to_le_bytes())?;
        writer.write_all(&self.manifest_offset.to_le_bytes())?;
        writer.write_all(&self.manifest_size.to_le_bytes())?;
        writer.write_all(&self.entry_count.to_le_bytes())?;

        Ok(())
    }

    pub fn rewrite<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        writer.seek(SeekFrom::Start(0))?;
        self.write(writer)?;
        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic = [0u8; 4];
        reader
            .read_exact(&mut magic)
            .context("failed to read CWN magic")?;

        if magic != CWN_MAGIC {
            bail!("not a valid CWN container: invalid magic");
        }

        let version = read_u16(reader)?;
        let flags = read_u16(reader)?;
        let manifest_offset = read_u64(reader)?;
        let manifest_size = read_u64(reader)?;
        let entry_count = read_u64(reader)?;

        if version != CWN_VERSION {
            bail!(
                "unsupported CWN format version {} (supported: {})",
                version,
                CWN_VERSION
            );
        }

        Ok(Self {
            version,
            flags,
            manifest_offset,
            manifest_size,
            entry_count,
        })
    }
}

fn read_u16<R: Read>(reader: &mut R) -> Result<u16> {
    let mut bytes = [0u8; 2];
    reader.read_exact(&mut bytes)?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_u64<R: Read>(reader: &mut R) -> Result<u64> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}
