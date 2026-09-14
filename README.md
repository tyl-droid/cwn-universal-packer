# CWN Universal Packer

**CWN Universal Packer** is an archive and container utility developed by the Community Watch Network (CWN).

It provides a custom `.CWN` container format for packaging files and directories with compression, integrity verification, package metadata, safe extraction, archive inspection, and desktop archive management.

> Current release: **v0.9.0-alpha**

## Features

### CWN Container Format

CWN Universal Packer uses its own `.CWN` container format with:

- Structured container headers
- JSON manifest metadata
- Multiple files and directories
- SHA-256 integrity information
- Stored and Zstandard-compressed payloads
- Package name, version, publisher, and producer metadata
- File type detection
- Per-entry size and compression information

### Smart Compression

Files can be compressed using Zstandard.

The packer compares the compressed representation against the original data and only stores the compressed version when doing so actually reduces its size.

Files that do not benefit from compression are stored directly.

### Integrity Verification

CWN containers support SHA-256 verification.

The verification engine checks archive structure, payload boundaries, declared sizes, compression information, and file hashes before data is trusted.

### Safe Extraction

Extraction includes protections against malformed or hostile archive paths, including:

- Parent directory traversal
- Absolute paths
- Duplicate archive paths
- Overlapping payload regions
- Payloads overlapping the manifest
- Invalid compression metadata
- Invalid SHA-256 metadata
- Declared size mismatches
- Oversized decompression output
- Existing destination file overwrites

Selected-file extraction also removes partial output when extraction, integrity verification, or output flushing fails.

### Package Metadata

Packages can contain:

- Package name
- Package version
- Publisher
- Producer

The producer field is controlled by CWN Universal Packer and identifies the version of the software that created the package.

Metadata fields are validated before a package is written.

### Archive Inspection

The inspection engine reports:

- Format version
- Package metadata
- File and directory counts
- Original size
- Packed payload size
- Container size
- Compression statistics
- Individual archive entries
- File types
- Per-entry compression methods

### Desktop Interface

The desktop application provides separate Package and Archive workspaces.

Archive management includes:

- Archive inspection
- Search
- Entry filtering
- Sortable columns
- Entry selection
- SHA-256 verification
- Structural testing
- Full extraction
- Selected-file extraction
- Compression statistics

The GUI is built with Rust and `eframe`/`egui`.

### Command Line Interface

The `cwnpack` CLI supports:

```text
pack
unpack
list
info
verify
test
Package creation also supports custom metadata through:
--package-name
--package-version
--publisher
Example
Create a package:
cwnpack pack ./example-folder \
  -o Example.CWN \
  --package-name "Example Package" \
  --package-version "1.0.0" \
  --publisher "Community Watch Network"
Inspect it:
cwnpack info Example.CWN
Verify it:
cwnpack verify Example.CWN
Extract it:
cwnpack unpack Example.CWN ./extracted
Development
Format the project:
cargo fmt --check
Check the project:
cargo check
Run the test suite:
cargo test
v0.9.0-alpha Validation
The v0.9.0-alpha release passed the local Rust validation suite on the project's Android/Termux development environment.
At release time:
cargo fmt --check passed
cargo check passed
29 integration tests passed
SHA-256 and malformed-container tests passed
Selected extraction tests passed
Package metadata tests passed
Working tree was clean at the release commit
Desktop Validation Notice
The desktop-only GUI code was not independently compiled on a desktop target before the v0.9.0-alpha tag was created.
A GitHub Actions desktop compile workflow exists, but its runner was prevented from starting because of an account billing restriction. No workflow build steps executed.
This is therefore an alpha release and desktop functionality should be considered pending independent desktop-target validation.
Project Status
CWN Universal Packer is under active development.
The .CWN format and application interfaces may change between alpha releases. Do not treat the current format as permanently stable until a stable format specification is published.
Security
CWN Universal Packer is designed as a general archive/container utility.
It is not intended for executable injection, malware packing, security-product evasion, or payload obfuscation.
Security-sensitive archive parsing should continue to be treated defensively, particularly while the project remains in alpha.
Developed By
Community Watch Network (CWN)
CWN Universal Packer is part of CWN's software and tooling projects.
