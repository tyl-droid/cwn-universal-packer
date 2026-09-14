use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn rejects_invalid_magic() {
    let temp = tempdir().unwrap();

    let archive = temp.path().join("fake.CWN");

    fs::write(&archive, b"THIS IS NOT A CWN FILE").unwrap();

    Command::cargo_bin("cwnpack")
        .unwrap()
        .arg("test")
        .arg(&archive)
        .assert()
        .failure();
}
