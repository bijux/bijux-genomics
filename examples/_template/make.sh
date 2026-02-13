#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)

usage() {
  cat <<'EOF'
Usage: examples/_template/make.sh <example-id>
Regenerates golden outputs for an example and refreshes corpus checksums.
EOF
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

[[ $# -eq 1 ]] || { usage >&2; exit 2; }
example_id="$1"

"$ROOT_DIR/bin/isolate" sh -ceu "
  ./scripts/examples/run.sh ${example_id}
"

example_dir="$(find "$ROOT_DIR/examples" -type f -name example.toml -print | while read -r f; do
  if rg -q \"^id\\s*=\\s*\\\"${example_id}\\\"\\s*$\" \"$f\"; then
    dirname \"$f\"
    break
  fi
done)"
[[ -n "$example_dir" ]] || { echo "unknown example id: $example_id" >&2; exit 1; }

iso_root="$(ISO_TAG="${ISO_TAG:-}" "$ROOT_DIR/bin/isolate" --print-root 2>/dev/null || true)"
if [[ -z "$iso_root" ]]; then
  iso_root="$ROOT_DIR/artifacts/isolates"
fi

art_dir="$(find "$ROOT_DIR/artifacts/isolates" -type d -path \"*/examples/${example_id}\" | sort | tail -n1)"
[[ -n "$art_dir" ]] || { echo "no artifacts found for $example_id" >&2; exit 1; }

cp -f "$art_dir/plan.json" "$example_dir/golden/plan.json"
cp -f "$art_dir/explain.json" "$example_dir/golden/explain.json"
cp -f "$art_dir/report.json" "$example_dir/golden/report.json"

for corpus in "$ROOT_DIR"/examples/data/corpus-*; do
  [[ -d "$corpus" ]] || continue
  (
    cd "$corpus"
    find raw normalized -type f 2>/dev/null | sort | while read -r f; do
      shasum -a 256 "$f"
    done > CHECKSUMS.sha256
  )
done

echo "example template refresh: updated ${example_dir#"$ROOT_DIR/"} golden files and corpus checksums"
