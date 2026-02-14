#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

out_dir="$ROOT_DIR/containers/licenses"
mkdir -p "$out_dir"

python3 - "$ROOT_DIR" "$out_dir" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
out_dir = Path(sys.argv[2])
for path in sorted((root / "containers/apptainer").rglob("*.def")):
    text = path.read_text(encoding="utf-8")
    tool = path.stem
    source = ""
    license_name = ""
    version = ""
    for line in text.splitlines():
        s = line.strip()
        if s.startswith("org.opencontainers.image.source "):
            source = s.split(" ", 1)[1].strip()
        elif s.startswith("org.opencontainers.image.licenses "):
            license_name = s.split(" ", 1)[1].strip()
        elif s.startswith("org.opencontainers.image.version "):
            version = s.split(" ", 1)[1].strip()
    if not source:
        source = "unknown"
    if not license_name:
        license_name = "unknown"
    if not version:
        version = "unknown"
    kind = "bijux" if "/bijux/" in str(path) else "non-bijux"
    out = out_dir / f"{tool}.license.toml"
    if source == "unknown":
        source = "https://example.invalid/unknown-source"
    spdx = license_name if license_name else "NOASSERTION"
    if spdx == "unknown":
        spdx = "NOASSERTION"
    out.write_text(
        "\n".join(
            [
                "# schema_version = 1",
                "# owner = bijux-dna-platform",
                f'tool_id = "{tool}"',
                f'container_kind = "{kind}"',
                f'spdx = "{spdx}"',
                f'upstream_url = "{source}"',
                'redistribution_note = "See upstream license terms and redistribution policy."',
                f'citation = "upstream:{source}"',
                f'version = "{version}"',
                "",
            ]
        ),
        encoding="utf-8",
    )
print(f"generated {out_dir}")
PY
