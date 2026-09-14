use cwn_universal_packer::commands::pack;
use cwn_universal_packer::container;
use cwn_universal_packer::container::manifest::PackageMetadata;

use std::fs;

#[test]
fn custom_package_metadata_roundtrips() {
    let temp = tempfile::tempdir().unwrap();

    let input = temp.path().join("hello.txt");
    let output = temp.path().join("custom.CWN");

    fs::write(&input, b"hello from CWN").unwrap();

    let metadata = PackageMetadata::new("CWN Test Package", "2.4.1", "CWN Testing Division");

    pack::run_with_metadata(vec![input], output.clone(), 10, metadata).unwrap();

    let (_, manifest) = container::read_manifest(&output).unwrap();

    assert_eq!(manifest.package_name, "CWN Test Package");
    assert_eq!(manifest.package_version, "2.4.1");
    assert_eq!(manifest.publisher, "CWN Testing Division");

    assert_eq!(
        manifest.producer,
        format!("CWN Universal Packer {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn package_metadata_is_trimmed() {
    let metadata =
        PackageMetadata::new("  Example Package  ", "  1.0.0  ", "  Example Publisher  ")
            .validate()
            .unwrap();

    assert_eq!(metadata.name, "Example Package");
    assert_eq!(metadata.version, "1.0.0");
    assert_eq!(metadata.publisher, "Example Publisher");
}

#[test]
fn empty_package_name_is_rejected() {
    let error = PackageMetadata::new("   ", "1.0.0", "Community Watch Network")
        .validate()
        .unwrap_err();

    assert!(error.to_string().contains("package name cannot be empty"));
}

#[test]
fn oversized_metadata_is_rejected() {
    let error = PackageMetadata::new("A".repeat(129), "1.0.0", "Community Watch Network")
        .validate()
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("package name cannot exceed 128 characters")
    );
}

#[test]
fn control_characters_are_rejected() {
    let error = PackageMetadata::new("CWN\nPackage", "1.0.0", "Community Watch Network")
        .validate()
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("package name cannot contain control characters")
    );
}

#[test]
fn publisher_control_characters_are_rejected() {
    let error = PackageMetadata::new("CWN Package", "1.0.0", "Community\nWatch Network")
        .validate()
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("publisher cannot contain control characters")
    );
}
