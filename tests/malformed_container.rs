use assert_cmd::Command;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn build_container(path: &Path, payload: &[u8], entries: serde_json::Value) {
    const HEADER_SIZE: u64 = 32;

    let manifest = json!({
        "format": "CWN",
        "version": 1,
        "producer": "CWN Security Test",
        "package_name": "Malformed Test",
        "package_version": "test",
        "publisher": "Community Watch Network",
        "entries": entries
    });

    let manifest_bytes = serde_json::to_vec_pretty(&manifest).unwrap();

    let manifest_offset = HEADER_SIZE + payload.len() as u64;

    let entry_count = manifest["entries"].as_array().unwrap().len() as u64;

    let mut out = Vec::new();

    out.extend_from_slice(b"CWN1");
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&manifest_offset.to_le_bytes());
    out.extend_from_slice(&(manifest_bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(&entry_count.to_le_bytes());

    assert_eq!(out.len(), 32);

    out.extend_from_slice(payload);
    out.extend_from_slice(&manifest_bytes);

    fs::write(path, out).unwrap();
}

#[test]
fn rejects_duplicate_paths() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("duplicate.CWN");

    let payload = b"AB";

    build_container(
        &archive,
        payload,
        json!([
            {
                "path": "same.txt",
                "kind": "file",
                "file_type": "Text",
                "original_size": 1,
                "packed_size": 1,
                "data_offset": 32,
                "compression": "none",
                "sha256": sha256(b"A")
            },
            {
                "path": "same.txt",
                "kind": "file",
                "file_type": "Text",
                "original_size": 1,
                "packed_size": 1,
                "data_offset": 33,
                "compression": "none",
                "sha256": sha256(b"B")
            }
        ]),
    );

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("test")
        .arg(&archive)
        .assert()
        .failure();
}

#[test]
fn rejects_overlapping_payloads() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("overlap.CWN");

    let payload = b"ABCD";

    build_container(
        &archive,
        payload,
        json!([
            {
                "path": "one.bin",
                "kind": "file",
                "file_type": "Binary / Other",
                "original_size": 3,
                "packed_size": 3,
                "data_offset": 32,
                "compression": "none",
                "sha256": sha256(b"ABC")
            },
            {
                "path": "two.bin",
                "kind": "file",
                "file_type": "Binary / Other",
                "original_size": 2,
                "packed_size": 2,
                "data_offset": 34,
                "compression": "none",
                "sha256": sha256(b"CD")
            }
        ]),
    );

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("test")
        .arg(&archive)
        .assert()
        .failure();
}

#[test]
fn rejects_payload_inside_manifest() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("manifest-overlap.CWN");

    build_container(
        &archive,
        b"A",
        json!([
            {
                "path": "bad.bin",
                "kind": "file",
                "file_type": "Binary / Other",
                "original_size": 100,
                "packed_size": 100,
                "data_offset": 32,
                "compression": "none",
                "sha256": sha256(b"A")
            }
        ]),
    );

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("test")
        .arg(&archive)
        .assert()
        .failure();
}

#[test]
fn rejects_unknown_compression() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("compression.CWN");

    build_container(
        &archive,
        b"A",
        json!([
            {
                "path": "bad.bin",
                "kind": "file",
                "file_type": "Binary / Other",
                "original_size": 1,
                "packed_size": 1,
                "data_offset": 32,
                "compression": "evilzip9000",
                "sha256": sha256(b"A")
            }
        ]),
    );

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("test")
        .arg(&archive)
        .assert()
        .failure();
}

#[test]
fn rejects_parent_path_during_extraction() {
    let temp = tempdir().unwrap();

    let archive = temp.path().join("traversal.CWN");
    let output = temp.path().join("output");

    build_container(
        &archive,
        b"A",
        json!([
            {
                "path": "../escape.txt",
                "kind": "file",
                "file_type": "Text",
                "original_size": 1,
                "packed_size": 1,
                "data_offset": 32,
                "compression": "none",
                "sha256": sha256(b"A")
            }
        ]),
    );

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("unpack")
        .arg(&archive)
        .arg("-o")
        .arg(&output)
        .assert()
        .failure();

    assert!(!temp.path().join("escape.txt").exists());
}

#[test]
fn rejects_absolute_path_during_extraction() {
    let temp = tempdir().unwrap();

    let archive = temp.path().join("absolute.CWN");
    let output = temp.path().join("output");

    build_container(
        &archive,
        b"A",
        json!([
            {
                "path": "/tmp/cwn-escape-test.txt",
                "kind": "file",
                "file_type": "Text",
                "original_size": 1,
                "packed_size": 1,
                "data_offset": 32,
                "compression": "none",
                "sha256": sha256(b"A")
            }
        ]),
    );

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("unpack")
        .arg(&archive)
        .arg("-o")
        .arg(&output)
        .assert()
        .failure();
}

#[test]
fn bad_sha256_fails_verification() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("bad-hash.CWN");

    build_container(
        &archive,
        b"A",
        json!([
            {
                "path": "hello.txt",
                "kind": "file",
                "file_type": "Text",
                "original_size": 1,
                "packed_size": 1,
                "data_offset": 32,
                "compression": "none",
                "sha256": "0000000000000000000000000000000000000000000000000000000000000000"
            }
        ]),
    );

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("verify")
        .arg(&archive)
        .assert()
        .failure();
}

#[test]
fn rejects_malformed_sha256_metadata() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("bad-sha-format.CWN");

    build_container(
        &archive,
        b"A",
        json!([
            {
                "path": "file.txt",
                "kind": "file",
                "file_type": "Text",
                "original_size": 1,
                "packed_size": 1,
                "data_offset": 32,
                "compression": "none",
                "sha256": "definitely-not-a-sha256"
            }
        ]),
    );

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("test")
        .arg(&archive)
        .assert()
        .failure();
}

#[test]
fn corrupt_zstd_payload_fails_verification() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("bad-zstd.CWN");

    let payload = b"this is not a zstd stream";

    build_container(
        &archive,
        payload,
        json!([
            {
                "path": "compressed.txt",
                "kind": "file",
                "file_type": "Text",
                "original_size": 100,
                "packed_size": payload.len(),
                "data_offset": 32,
                "compression": "zstd",
                "sha256": sha256(b"expected decoded contents")
            }
        ]),
    );

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("verify")
        .arg(&archive)
        .assert()
        .failure();
}

#[test]
fn declared_size_mismatch_fails_verification() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("size-mismatch.CWN");

    let payload = b"ABCDE";

    build_container(
        &archive,
        payload,
        json!([
            {
                "path": "file.bin",
                "kind": "file",
                "file_type": "Binary / Other",
                "original_size": 2,
                "packed_size": 5,
                "data_offset": 32,
                "compression": "none",
                "sha256": sha256(payload)
            }
        ]),
    );

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("verify")
        .arg(&archive)
        .assert()
        .failure();
}
