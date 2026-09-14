use std::path::Path;

pub fn detect_file_type(path: &Path) -> &'static str {
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

        "txt" | "md" => "Text",

        _ => "Binary / Other",
    }
}
