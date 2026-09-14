use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub fn detect_file_type(path: &Path) -> String {
    if let Some(kind) = detect_magic(path) {
        return kind;
    }

    detect_extension(path).to_string()
}

fn detect_magic(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;

    let mut header = [0u8; 16];
    let read = file.read(&mut header).ok()?;

    if read >= 4 {
        if header.starts_with(b"\x7FELF") {
            return Some("ELF Binary".to_string());
        }

        if header.starts_with(b"PK\x03\x04") {
            return Some("ZIP Archive".to_string());
        }

        if header.starts_with(b"\x89PNG\r\n\x1A\n") {
            return Some("PNG Image".to_string());
        }

        if header.starts_with(b"%PDF") {
            return Some("PDF Document".to_string());
        }

        if header[0..3] == [0xFF, 0xD8, 0xFF] {
            return Some("JPEG Image".to_string());
        }

        if header.starts_with(b"GIF87a") || header.starts_with(b"GIF89a") {
            return Some("GIF Image".to_string());
        }

        if header.starts_with(b"\x1F\x8B") {
            return Some("GZip Archive".to_string());
        }

        if header.starts_with(b"7z\xBC\xAF\x27\x1C") {
            return Some("7-Zip Archive".to_string());
        }

        if header.starts_with(b"Rar!\x1A\x07") {
            return Some("RAR Archive".to_string());
        }

        if header.starts_with(b"MZ") {
            return detect_pe_type(&mut file).or_else(|| Some("Windows PE Binary".to_string()));
        }
    }

    None
}

fn detect_pe_type(file: &mut File) -> Option<String> {
    file.seek(SeekFrom::Start(0x3C)).ok()?;

    let mut offset_bytes = [0u8; 4];
    file.read_exact(&mut offset_bytes).ok()?;

    let pe_offset = u32::from_le_bytes(offset_bytes) as u64;

    file.seek(SeekFrom::Start(pe_offset)).ok()?;

    let mut signature = [0u8; 4];
    file.read_exact(&mut signature).ok()?;

    if signature != *b"PE\0\0" {
        return None;
    }

    /*
        PE layout after signature:

        +0x00  Signature
        +0x04  Machine
        +0x06  NumberOfSections
        ...
        +0x16  Characteristics
    */

    file.seek(SeekFrom::Start(pe_offset + 22)).ok()?;

    let mut characteristics_bytes = [0u8; 2];
    file.read_exact(&mut characteristics_bytes).ok()?;

    let characteristics = u16::from_le_bytes(characteristics_bytes);

    const IMAGE_FILE_DLL: u16 = 0x2000;
    const IMAGE_FILE_SYSTEM: u16 = 0x1000;

    if characteristics & IMAGE_FILE_DLL != 0 {
        return Some("Windows DLL".to_string());
    }

    if characteristics & IMAGE_FILE_SYSTEM != 0 {
        return Some("Windows System Binary".to_string());
    }

    Some("Windows EXE".to_string())
}

fn detect_extension(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "exe" => "Windows EXE",
        "dll" => "Windows DLL",
        "sys" => "Windows Driver",

        "so" => "Linux Shared Object",
        "elf" => "ELF Binary",

        "py" => "Python",
        "js" => "JavaScript",
        "ts" => "TypeScript",
        "lua" => "Lua",
        "ps1" => "PowerShell",
        "bat" | "cmd" => "Windows Batch",
        "sh" => "Shell Script",

        "json" => "JSON",
        "xml" => "XML",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "ini" => "INI",

        "png" => "PNG Image",
        "jpg" | "jpeg" => "JPEG Image",
        "gif" => "GIF Image",
        "webp" => "WebP Image",

        "zip" => "ZIP Archive",
        "7z" => "7-Zip Archive",
        "rar" => "RAR Archive",
        "tar" => "TAR Archive",
        "gz" => "GZip Archive",

        "pdf" => "PDF Document",

        "txt" | "md" => "Text",

        _ => "Binary / Other",
    }
}
