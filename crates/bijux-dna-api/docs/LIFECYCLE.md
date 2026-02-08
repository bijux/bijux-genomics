# LIFECYCLE

## Plan → Execute → Report → Index
1. Plan: returns graph + graph hash.
2. Execute: produces run_manifest.json and step artifacts.
3. Report: produces report.json + report.html + summary.tsv.
4. Index: run_index.jsonl lists completed runs.

## Integrity verification
- verify graph hash in manifest
- verify artifact list exists
- verify report bundle contains expected files
