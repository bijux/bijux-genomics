#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

violations="$(
  find "$ROOT_DIR/containers" -type f \( -iname '*version*' -o -iname '*lock*' \) \
    | sed "s#^$ROOT_DIR/##" \
    | grep -vE '^containers/versions/(versions\.toml|lock\.json|LOCK\.md|index\.md)$' || true
)"
if [[ -n "$violations" ]]; then
  echo "non-canonical version/lock files found under containers/ (use containers/versions/* only):" >&2
  printf '%s\n' "$violations" >&2
  exit 1
fi

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import hashlib
import json
import sys

root = Path(sys.argv[1])
lock_path = root / "containers/versions/lock.json"
versions_path = root / "containers/versions/versions.toml"
lock = json.loads(lock_path.read_text(encoding="utf-8"))

errors = []
if lock.get("schema_version") != "bijux.container.version_lock.v1":
    errors.append("lock.json schema_version must be bijux.container.version_lock.v1")
if lock.get("source") != "containers/versions/versions.toml":
    errors.append("lock.json source must be containers/versions/versions.toml")
expected_sha = hashlib.sha256(versions_path.read_bytes()).hexdigest()
if lock.get("source_sha256") != expected_sha:
    errors.append("lock.json source_sha256 does not match versions.toml")

items = lock.get("items")
if not isinstance(items, list) or not items:
    errors.append("lock.json items must be a non-empty list")

if errors:
    print("version authority check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("version authority: OK")
PY
