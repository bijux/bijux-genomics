# Output Formats

## Terminal Text
- Help output is deterministic and snapshotted.
- Human-readable text may improve wording, but command names, exit category, and required hints must
  remain reviewable.
- Help snapshot changes must be intentional and reviewed with command parser changes.
- Text examples should use canonical command names from [COMMANDS.md](COMMANDS.md).

## JSON
- JSON output must be stable, ordered when feasible, and schema-compatible unless the change is
  explicitly breaking.
- Report and plan JSON are delegated to API/runtime contracts.
- Operator-facing JSON should not include host-specific absolute paths unless the path is the
  requested output.

## Dry-Run Artifacts
- Dry-run manifests and graphs are planning evidence, not execution evidence.
- They must not include wall-clock timestamps or random identifiers unless the API contract
  normalizes them.

## Reports
- CLI report rendering delegates to `bijux-dna-api`.
- Report files are written only to requested paths or documented defaults.

## Operator Errors
- Parse errors, contract errors, tool errors, and infrastructure errors must keep distinct exit
  behavior.
- Refusal payloads should include what failed, why it failed, and how the operator can proceed.
- Minimal bug reports should include the command and flags, terminal or JSON error payload,
  `run_manifest.json` when present, and relevant declared artifacts.

## Verification
- `tests/contracts/cli_behavior.rs`
- `tests/snapshots/*.txt`
- `src/process_exit.rs` unit tests
