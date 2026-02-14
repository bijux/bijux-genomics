#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="$ROOT_DIR/docs/20-science/DOMAIN_COVERAGE.generated.md"

usage() {
    cat <<'EOF'
Usage: scripts/tooling/generate-domain-coverage-doc.sh [--out <path>] [--help]
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --out)
            shift
            [[ $# -gt 0 ]] || { echo "missing value for --out" >&2; exit 2; }
            OUT="$1"
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        --*)
            echo "unknown flag: $1" >&2
            usage >&2
            exit 2
            ;;
        *)
            echo "unexpected positional argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
    shift
done

python3 - "$ROOT_DIR" "$OUT" <<'PY'
from pathlib import Path
import json
import sys

root = Path(sys.argv[1])
out = Path(sys.argv[2])
domain_root = root / "domain"

rows = []
for dom in sorted(p for p in domain_root.iterdir() if p.is_dir()):
    stages = len([p for p in (dom / "stages").glob("*.yaml") if p.name != "_schema.yaml"])
    tools = len([p for p in (dom / "tools").glob("*.yaml") if p.name != "_schema.yaml"])
    fixtures = len(list((dom / "fixtures").glob("*/*.txt")))
    rows.append({"domain": dom.name, "stages": stages, "tools": tools, "fixtures": fixtures})

lines = [
    "<!-- GENERATED FILE - DO NOT EDIT -->",
    "<!-- Regenerate with: scripts/tooling/generate-domain-coverage-doc.sh -->",
    "",
    "# DOMAIN_COVERAGE",
    "",
    "## Purpose",
    "Generated coverage table for domain stages/tools/fixtures.",
    "",
    "## Scope",
    "Derived from `domain/*/{stages,tools,fixtures}`.",
    "",
    "## Non-goals",
    "- Replacing per-domain scientific specifications.",
    "",
    "## Contracts",
    "- Generated-only document; manual edits are forbidden.",
    "- Counts must be deterministic for a fixed repository state.",
    "",
    "| Domain | Stage Count | Tool Count | Fixture Count |",
    "|---|---:|---:|---:|",
]
for r in rows:
    lines.append(f"| `{r['domain']}` | {r['stages']} | {r['tools']} | {r['fixtures']} |")

out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(f"generated {out}")
PY
