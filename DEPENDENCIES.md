---
created_on: 2026-10-14 15:29
last_modified: 2026-10-14 15:29
status: current
---

# Dependency choices

| Go functionality | Native Rust choice |
| :--- | :--- |
| encoding/csv | csv 1.4.0 ReaderBuilder and UTF-8 StringRecord |
| archive/zip | zip 8.6.0 ZipArchive, without extracting archive paths |
| encoding/json | Serde models and serde_json serialization |
| google/uuid | uuid 1.26.1 v4 generation and native Serde support |
| flag and direct printing | cli-command-kit with native clap help and argument metadata |
| os.WriteFile | tempfile 3.27.0 destination-local private file, sync_all, atomic persist |

The Go MIT license and attribution are retained. Direct dependency licenses are MIT, MIT/Apache-2.0, or compatible options. APIs were checked in downloaded crate source and official [csv](https://docs.rs/csv/1.4.0/csv/struct.ReaderBuilder.html), [zip](https://docs.rs/zip/8.6.0/zip/read/struct.ZipArchive.html), and [tempfile](https://docs.rs/tempfile/3.27.0/tempfile/struct.NamedTempFile.html) documentation. Supported JSON fields were compared with the source converter and the official [Bitwarden exporter serialization](https://github.com/bitwarden/sdk-internal/blob/main/crates/bitwarden-exporters/src/json.rs). Its exporter is private to the SDK crate, so this local converter uses explicit Serde data models rather than an SDK wrapper.

# Corrected source behavior

- Go logs CSV failures and returns no items for the affected category, then reports successful output. Rust propagates the failure before destination replacement.
- Go trims passwords, usernames, notes, and other optional values. Rust preserves field contents exactly, including whitespace-only secrets.
- Go writes plaintext credentials with mode 0644 and directly truncates existing output. Rust stages privately with the native tempfile default 0600 on Unix, syncs, then atomically replaces the destination.
- Go silently accepts duplicate CSV columns and selects the first ZIP file with a matching basename. Rust rejects duplicate columns and recognized basenames found under multiple ZIP paths.
- Go can produce a blank identity for unknown ID types with a custom name. Rust retains unknown type and number in custom fields, using a hidden number field.
- Go accepts extra positionals and may silently produce an empty export from a nonexistent directory. Native clap rejects extras; filesystem metadata validation rejects missing input.

Missing categories, source category filters, item ordering, six-folder grouping, and known field mappings follow the Go converter. In particular, issuing_bank remains the source for card brand. CSV syntax follows the maintained csv crate rather than a custom reproduction of Go's parser. Conversion is local; successful Bitwarden account import and native macOS/Windows behavior require separate verification.
