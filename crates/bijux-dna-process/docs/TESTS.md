# Capture contracts

`tests/contracts.rs` exercises raw binary bytes and nonzero status, a shared stdout/stderr
budget including its exact limit, continuous-output cancellation, inherited pipes after
leader exit, closed pipes with a live child, and rejection before launching an invalid
deadline. Shell snippets are generated inline; no downloaded fixtures are required.

Runner retains its original command and backend contracts. Container admission verifies
that excessive version output cannot publish an admission record. These Unix tests do not
qualify other platforms or descendants that escape the owned process group.

From the repository root, use `cargo nextest run -p bijux-dna-process` with the governed
Cargo target directory. Full integration also runs runner and container contract suites.
