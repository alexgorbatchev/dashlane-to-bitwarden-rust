`dashlane-to-bitwarden` converts local Dashlane CSV exports into a Bitwarden JSON import file. It accepts an export directory or ZIP archive and keeps credentials on your computer.

# What It Does

- **Convert six categories:** Export logins, cards, bank accounts, identities, IDs, and secure notes into separate folders.
- **Preserve values:** Retain password and note whitespace, extra usernames, bank details, and ID dates.
- **Protect output:** Parse every category before replacing the destination with a private, completed JSON file.

# How It Works

- Export your Dashlane data to a CSV directory or ZIP archive.
- Run `dashlane-to-bitwarden export convert <input> <output>` with your export and a destination JSON path.
- Import the resulting file into Bitwarden using its Bitwarden JSON format.

# How it Really Works

- The converter reads credentials.csv, payments.csv, personalInfo.csv, ids.csv, and securenotes.csv. ZIP files may contain these files under nested paths. Missing and empty files contribute no items.
- Invalid UTF-8, mismatched CSV row widths, duplicate columns, ambiguous recognized ZIP basenames, and read failures stop conversion before replacing an existing output.
- All records are processed in memory. Conversion makes no network requests and leaves source files intact. Output contains unencrypted credentials; Unix files use mode 0600, subject to the process umask. Windows access control follows the operating system's temporary-file defaults.
- Each run generates fresh item and folder IDs. Repeated imports can create duplicates. Other payment types and personal-information types besides `name` are skipped. Unknown ID types retain their type and number as custom fields.
- Human stdout reports the destination and category counts. `AGENT=1` prints one compact JSON summary. Displayed home paths use `~`. Errors go once to stderr; success exits 0, and argument, conversion, or I/O failures exit 1. A summary-write failure can occur after a completed output file replacement.

# Installation

This Rust implementation has no published binary release. The original repository's Go release artifacts are separate from this project.

# Quick Start

```sh
dashlane-to-bitwarden export convert ~/Downloads/dashlane-export.zip bitwarden-export.json
```

Sample Output for an export containing one login:

```text
Created: bitwarden-export.json
Logins: 1
Cards: 0
Identities: 0
Secure Notes: 0
```

```sh
AGENT=1 dashlane-to-bitwarden ~/Downloads/dashlane-export bitwarden-export.json
dashlane-to-bitwarden --version
```

# Options & Flags

| Flag | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--help` | `-h` | false | Print command help |
| `--version` | `-v` | false | Print the raw version |

`dashlane-to-bitwarden <input> <output>` performs the same conversion as `export convert`. `dashlane-to-bitwarden skill` prints the canonical agent-skill installation pointer. Input and output paths are required for conversion; the command creates missing destination parent directories.

# License

[MIT](LICENSE).
