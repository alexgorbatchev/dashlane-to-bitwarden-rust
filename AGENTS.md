---
created_on: 2026-10-14 15:29
last_modified: 2026-10-14 15:29
status: current
---

# dashlane-to-bitwarden

Local Dashlane CSV/ZIP conversion to unencrypted Bitwarden JSON. Inherit [../AGENTS.md](../AGENTS.md).

## Commands

- Build: `just build`, producing `bin/dashlane-to-bitwarden`.
- Run: `just run export convert <input> <output>`; agent mode: `just run-ai export convert <input> <output>`.
- Test: `just test`; lint: `just lint`; full gate: `just check`; coverage: `just coverage`.

## Conversion and file contracts

- Use native csv readers, zip archive APIs, UUID generation, Serde serialization, and tempfile atomic persistence. Preserve recognized category mappings and folder references.
- Preserve credential and note values exactly, including whitespace; use null only for empty optional fields. Keep unknown ID type/number values in custom fields rather than losing them.
- Propagate source read/parse failures. Parse every category before staging output. Reject duplicate CSV columns and ambiguous recognized basenames across ZIP paths.
- Keep output staging in the destination directory, owner-only on Unix, and sync the file before atomic replacement. Never truncate an existing destination before successful conversion; reject destinations that resolve to source export files.
- Keep credentials out of logs, summaries, diagnostics, tests using real data, and debug representations. Use synthetic fixtures. Conversion has no network access or external subprocess dependencies.
- Keep one error-reporting boundary in `entry`; output-write failures are errors even after a completed file replacement. Display paths through the shared home-folding helper.
- Update `skills/dashlane-to-bitwarden/SKILL.md` and its `last_modified` in the same change whenever commands, arguments, flags, defaults, environments, formats, output, or side effects change. Verify against the live native command metadata.
- Runtime-result changes require corresponding behavioral test-file changes and at least 90% line coverage; scripts are excluded. Disable behavior to prove its regression test fails, restore the implementation, and rerun the gate.
- Keep temporary evidence under `.tmp/` and task progress outside agent instructions. Scope formatters to owned files.
- Never publish releases, tags, packages, or deployments without explicit authorization. The canonical Go repository URLs do not establish Rust release availability.
