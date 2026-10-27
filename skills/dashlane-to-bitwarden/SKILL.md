---
name: dashlane-to-bitwarden
description: Convert a local Dashlane CSV directory or ZIP archive into a Bitwarden JSON import file.
metadata:
  created_on: 2026-10-14 15:28
  last_modified: 2026-10-14 15:28
  status: current
---

# Usage

`dashlane-to-bitwarden <INPUT> <OUTPUT>` and `dashlane-to-bitwarden export convert <INPUT> <OUTPUT>` convert the same files. INPUT and OUTPUT are required native filesystem paths; INPUT is an existing directory or a regular `.zip` file, case-insensitive extension. OUTPUT is the JSON destination; parent directories are created when needed, and an existing file is replaced atomically after conversion succeeds.

The converter reads credentials.csv, payments.csv, personalInfo.csv, ids.csv, and securenotes.csv. ZIP entries may have nested paths; multiple entries sharing a recognized basename cause an ambiguity error. Missing or empty categories contribute no items. Invalid UTF-8, unequal CSV row widths, duplicate column names, and read failures abort conversion before output replacement. UTF-8 BOM headers and quoted multiline fields are accepted. Conversion reads all records into memory and makes no network requests.

Output contains six folders: Logins, Cards, Bank Accounts, Identities, IDs, and Secure Notes. Credentials become logins; payment_card payments become cards; bank payments become secure notes with account fields; name personalInfo rows and all IDs become identities; securenotes rows become secure notes. Other payment and personalInfo row types are skipped. Source field values, including password and note whitespace, are preserved. Empty optional values become JSON null. Bank routing/account numbers use hidden custom fields. Extra usernames and ID issue/expiration dates become custom fields. IDs use passport, license, and social_security mappings; other ID types retain their type as Document Type and number as a hidden Document Number custom field. Each folder and item receives a fresh UUID. Repeated imports can create duplicates.

The JSON file is unencrypted and uses owner-only file mode 0600 on Unix, subject to the process umask. Existing source export files cannot be selected as OUTPUT. Windows access control follows the native temporary-file implementation; Unix permissions are not a Windows ACL guarantee. No source file is modified or deleted. A stdout failure after destination replacement exits unsuccessfully while leaving the completed file present.

`dashlane-to-bitwarden --version` or `dashlane-to-bitwarden -v` prints the raw version and newline. `--version` has type bool, default false, and is exclusive with other options and positionals.

`dashlane-to-bitwarden --help` or `dashlane-to-bitwarden -h` prints root help. `--help` has type bool, default false, and is available on every command. `dashlane-to-bitwarden export --help` and `dashlane-to-bitwarden export convert --help` print their help. `dashlane-to-bitwarden skill --help` prints skill-command help. Generated commands `dashlane-to-bitwarden help`, `dashlane-to-bitwarden help export`, `dashlane-to-bitwarden help export convert`, `dashlane-to-bitwarden help skill`, and `dashlane-to-bitwarden help help` print corresponding help without conversion. There are no completion commands.

`dashlane-to-bitwarden export` requires its convert subcommand. `dashlane-to-bitwarden skill` accepts no positional arguments or command-specific flags and prints the fixed Skills CLI installation command and canonical skill URL without reading files or credentials. That pointer identifies the original repository and does not establish Rust release publication.

`dashlane-to-bitwarden export help`, `dashlane-to-bitwarden export help convert`, and `dashlane-to-bitwarden export help help` are generated help paths for the export group, conversion command, and its generated help command.

`AGENT=1`, `AGENT=true`, or `AGENT=yes` selects agent mode. Agent conversion stdout is one compact JSON summary with output, items, logins, cards, identities, and secure_notes. Human stdout prints Created, Logins, Cards, Identities, and Secure Notes lines. Displayed paths fold the home directory to `~`. Output summaries contain counts and paths; credentials appear only in the JSON file. Errors go once to stderr with `ERR:` in agent mode or `[ERROR]` in human mode. Help starts with the skill-reading alert in agent mode. Skill and raw version bytes are identical in both modes. Success exits 0; invalid arguments, conversion errors, and I/O failures exit 1. Runtime failures print no usage screen.

```sh
dashlane-to-bitwarden export convert ~/Downloads/dashlane-export.zip bitwarden-export.json
AGENT=1 dashlane-to-bitwarden ~/Downloads/dashlane-export bitwarden-export.json
```
