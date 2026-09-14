use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn pack_verify_unpack_roundtrip() {
    let temp = tempdir().unwrap();

    let source = temp.path().join("source");
    let archive = temp.path().join("test.CWN");
    let extracted = temp.path().join("extracted");

    fs::create_dir_all(source.join("scripts")).unwrap();

    fs::write(source.join("hello.txt"), b"Community Watch Network\n").unwrap();

    fs::write(source.join("scripts/test.py"), b"print('CWN test')\n").unwrap();

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
        .arg("test")
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

    let root_name = source.file_name().unwrap();

    let extracted_root = extracted.join(root_name);

    assert_eq!(
        fs::read(source.join("hello.txt")).unwrap(),
        fs::read(extracted_root.join("hello.txt")).unwrap()
    );

    assert_eq!(
        fs::read(source.join("scripts/test.py")).unwrap(),
        fs::read(extracted_root.join("scripts/test.py")).unwrap()
    );
}
