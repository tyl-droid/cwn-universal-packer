use cwn_universal_packer::commands::{pack, unpack};
use cwn_universal_packer::engine::inspect::inspect_container;

use std::fs;
use tempfile::tempdir;

fn build_test_archive() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp = tempdir().unwrap();

    let source = temp.path().join("source");
    let archive = temp.path().join("selected-test.CWN");

    fs::create_dir_all(source.join("nested")).unwrap();

    fs::write(
        source.join("first.txt"),
        b"First CWN selected extraction file\n",
    )
    .unwrap();

    fs::write(
        source.join("second.txt"),
        b"Second CWN selected extraction file\n",
    )
    .unwrap();

    fs::write(
        source.join("nested/deep.txt"),
        b"Nested CWN extraction test\n",
    )
    .unwrap();

    pack::run(vec![source.clone()], archive.clone(), 10).unwrap();

    (temp, source, archive)
}

#[test]
fn extracts_only_selected_file() {
    let (temp, source, archive) = build_test_archive();

    let output = temp.path().join("selected-output");

    unpack::run_selected(archive, output.clone(), "source/first.txt".to_string()).unwrap();

    assert_eq!(
        fs::read(output.join("source/first.txt")).unwrap(),
        fs::read(source.join("first.txt")).unwrap()
    );

    assert!(!output.join("source/second.txt").exists());
    assert!(!output.join("source/nested/deep.txt").exists());
}

#[test]
fn extracts_selected_nested_file() {
    let (temp, source, archive) = build_test_archive();

    let output = temp.path().join("nested-output");

    unpack::run_selected(
        archive,
        output.clone(),
        "source/nested/deep.txt".to_string(),
    )
    .unwrap();

    assert_eq!(
        fs::read(output.join("source/nested/deep.txt")).unwrap(),
        fs::read(source.join("nested/deep.txt")).unwrap()
    );

    assert!(!output.join("source/first.txt").exists());
    assert!(!output.join("source/second.txt").exists());
}

#[test]
fn rejects_missing_selected_entry() {
    let (temp, _source, archive) = build_test_archive();

    let output = temp.path().join("missing-output");

    let result = unpack::run_selected(
        archive,
        output.clone(),
        "source/does-not-exist.txt".to_string(),
    );

    assert!(result.is_err());

    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("archive entry not found")
    );

    assert!(!output.join("source/does-not-exist.txt").exists());
}

#[test]
fn refuses_to_overwrite_selected_file() {
    let (temp, _source, archive) = build_test_archive();

    let output = temp.path().join("overwrite-output");
    let destination = output.join("source/first.txt");

    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    fs::write(&destination, b"DO NOT OVERWRITE").unwrap();

    let result = unpack::run_selected(archive, output, "source/first.txt".to_string());

    assert!(result.is_err());

    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("refusing to overwrite existing file")
    );

    assert_eq!(fs::read(&destination).unwrap(), b"DO NOT OVERWRITE");
}

#[test]
fn selected_extraction_rejects_unsafe_requested_path() {
    let (temp, _source, archive) = build_test_archive();

    let output = temp.path().join("unsafe-output");

    let result = unpack::run_selected(archive, output, "../escape.txt".to_string());

    assert!(result.is_err());

    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("parent traversal rejected")
    );
}

#[test]
fn selected_archive_entry_paths_match_inspection_results() {
    let (_temp, _source, archive) = build_test_archive();

    let info = inspect_container(&archive).unwrap();

    assert!(
        info.entries
            .iter()
            .any(|entry| entry.path == "source/first.txt")
    );

    assert!(
        info.entries
            .iter()
            .any(|entry| entry.path == "source/nested/deep.txt")
    );
}

#[test]
fn corrupted_selected_payload_fails_and_removes_partial_file() {
    use std::fs::OpenOptions;
    use std::io::{Read, Seek, SeekFrom, Write};

    let (temp, _source, archive) = build_test_archive();

    let (_, manifest) = cwn_universal_packer::container::read_manifest(&archive).unwrap();

    let entry = manifest
        .entries
        .iter()
        .find(|entry| entry.path == "source/first.txt")
        .unwrap();

    assert!(entry.packed_size > 0);

    let data_offset = entry.data_offset;
    let packed_size = entry.packed_size;

    /*
     * Keep the manifest intact and modify only bytes belonging to the
     * selected file's payload.
     */
    let corrupt_offset = data_offset + (packed_size / 2);

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&archive)
        .unwrap();

    file.seek(SeekFrom::Start(corrupt_offset)).unwrap();

    let mut original = [0u8; 1];
    file.read_exact(&mut original).unwrap();

    file.seek(SeekFrom::Start(corrupt_offset)).unwrap();
    file.write_all(&[original[0] ^ 0xff]).unwrap();
    file.flush().unwrap();

    drop(file);

    let output = temp.path().join("corrupt-output");
    let destination = output.join("source/first.txt");

    let result = unpack::run_selected(archive, output, "source/first.txt".to_string());

    assert!(
        result.is_err(),
        "corrupted selected payload unexpectedly extracted successfully"
    );

    assert!(
        !destination.exists(),
        "failed extraction left a partial destination file behind"
    );
}
