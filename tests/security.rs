use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn rejects_completely_invalid_container() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("invalid.CWN");

    fs::write(&archive, b"NOT A CWN CONTAINER").unwrap();

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("test")
        .arg(&archive)
        .assert()
        .failure();
}

#[test]
fn rejects_truncated_container() {
    let temp = tempdir().unwrap();
    let archive = temp.path().join("truncated.CWN");

    // Correct magic but deliberately incomplete header.
    fs::write(&archive, b"CWN1\x01\x00").unwrap();

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("test")
        .arg(&archive)
        .assert()
        .failure();
}

#[test]
fn refuses_existing_output_archive() {
    let temp = tempdir().unwrap();

    let source = temp.path().join("source.txt");
    let archive = temp.path().join("existing.CWN");

    fs::write(&source, b"CWN test").unwrap();
    fs::write(&archive, b"existing data").unwrap();

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("pack")
        .arg(&source)
        .arg("-o")
        .arg(&archive)
        .assert()
        .failure();

    // Existing file must remain untouched.
    assert_eq!(fs::read(&archive).unwrap(), b"existing data");
}

#[test]
fn handles_empty_file() {
    let temp = tempdir().unwrap();

    let source = temp.path().join("empty.txt");
    let archive = temp.path().join("empty.CWN");
    let extracted = temp.path().join("out");

    fs::write(&source, b"").unwrap();

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("pack")
        .arg(&source)
        .arg("-o")
        .arg(&archive)
        .assert()
        .success();

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("verify")
        .arg(&archive)
        .assert()
        .success();

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("unpack")
        .arg(&archive)
        .arg("-o")
        .arg(&extracted)
        .assert()
        .success();

    assert_eq!(fs::read(extracted.join("empty.txt")).unwrap(), b"");
}
