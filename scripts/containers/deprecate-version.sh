#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

tool=""
version=""
rationale=""
sunset_date=""
replacement_tool=""
replacement_version=""
compatibility_mode="allowed"
today="$(date -u +%Y-%m-%d)"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tool) tool="${2:-}"; shift ;;
    --version) version="${2:-}"; shift ;;
    --rationale) rationale="${2:-}"; shift ;;
    --sunset-date) sunset_date="${2:-}"; shift ;;
    --replacement-tool) replacement_tool="${2:-}"; shift ;;
    --replacement-version) replacement_version="${2:-}"; shift ;;
    --compatibility-mode) compatibility_mode="${2:-}"; shift ;;
    --help|-h)
      cat <<'EOF'
Usage: scripts/containers/deprecate-version.sh --tool <id> --version <semver> --rationale <text> --sunset-date <YYYY-MM-DD> --replacement-tool <id> --replacement-version <semver> [--compatibility-mode allowed|blocked]
EOF
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
  shift
done

[[ -n "$tool" && -n "$version" && -n "$rationale" && -n "$sunset_date" && -n "$replacement_tool" && -n "$replacement_version" ]] || { echo "missing required args" >&2; exit 2; }
[[ "$compatibility_mode" == "allowed" || "$compatibility_mode" == "blocked" ]] || { echo "--compatibility-mode must be allowed|blocked" >&2; exit 2; }

python3 - "$ROOT_DIR" "$tool" "$version" "$today" "$sunset_date" "$rationale" "$compatibility_mode" "$replacement_tool" "$replacement_version" <<'PY'
from pathlib import Path
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
tool, version, deprecated_since, sunset_date, rationale, mode, replacement_tool, replacement_version = sys.argv[2:10]
versions_path = root / "containers/versions/versions.toml"
versions = tomllib.loads(versions_path.read_text(encoding="utf-8"))
if tool not in versions:
    raise SystemExit(f"unknown tool in versions.toml: {tool}")
if replacement_tool not in versions:
    raise SystemExit(f"unknown replacement_tool in versions.toml: {replacement_tool}")

dep = root / "containers/versions/deprecations.toml"
if dep.exists():
    current = tomllib.loads(dep.read_text(encoding="utf-8"))
    for row in current.get("deprecation", []):
        if str(row.get("tool_id")) == tool and str(row.get("version")) == version:
            raise SystemExit(f"deprecation already exists for {tool}@{version}")
    text = dep.read_text(encoding="utf-8").rstrip() + "\n\n"
else:
    text = "# schema_version = 1\n# owner = bijux-dna-platform\n\n"

text += "[[deprecation]]\n"
text += f'tool_id = "{tool}"\n'
text += f'version = "{version}"\n'
text += f'deprecated_since = "{deprecated_since}"\n'
text += f'sunset_date = "{sunset_date}"\n'
text += f'replacement_tool = "{replacement_tool}"\n'
text += f'replacement_version = "{replacement_version}"\n'
text += f'rationale = "{rationale}"\n'
text += f'compatibility_mode = "{mode}"\n'
dep.write_text(text, encoding="utf-8")
PY

"$SCRIPT_DIR/generate-version-lock.sh"
echo "deprecated ${tool}@${version} (compatibility_mode=${compatibility_mode})"
