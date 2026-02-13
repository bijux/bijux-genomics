#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
tool_status = {}
for raw in (root / "containers/TOOL_IDS.txt").read_text(encoding="utf-8").splitlines():
    line = raw.strip()
    if not line or line.startswith("#"):
        continue
    tool_id, status = line.split("\t", 1)
    tool_status[tool_id] = status

required_labels = [
    "org.opencontainers.image.source",
    "org.opencontainers.image.revision",
    "org.opencontainers.image.created",
    "org.opencontainers.image.licenses",
    "org.opencontainers.image.version",
    "org.opencontainers.image.tool",
    "org.opencontainers.image.title",
]
errors = []
for path in sorted((root / "containers/apptainer").rglob("*.def")):
    rel = str(path.relative_to(root))
    tool_id = path.stem
    status = tool_status.get(tool_id, "unknown")
    text = path.read_text(encoding="utf-8")
    head = "\n".join(text.splitlines()[:24])

    for marker in [
        f"# Tool ID: {tool_id}",
        "# Version policy:",
        "# Upstream source:",
        "# Build date policy:",
    ]:
        if marker not in head:
            errors.append(f"{rel}: missing standard header marker '{marker}'")

    for key in required_labels:
        if key not in text:
            errors.append(f"{rel}: missing label {key}")
    # Label contract aliases:
    # tool, version, source, license_ref, build_date, git_sha.
    alias_sets = {
        "tool": ["org.opencontainers.image.tool", "tool"],
        "version": ["org.opencontainers.image.version", "version"],
        "source": ["org.opencontainers.image.source", "source"],
        "license_ref": ["org.opencontainers.image.licenses", "license_ref"],
        "build_date": ["org.opencontainers.image.created", "build_date"],
        "git_sha": ["org.opencontainers.image.revision", "git_sha"],
    }
    for alias, keys in alias_sets.items():
        if not any(k in text for k in keys):
            errors.append(f"{rel}: missing label contract key '{alias}'")

    if "%environment" not in text:
        errors.append(f"{rel}: missing %environment section")
    else:
        env = text.split("%environment", 1)[1].split("\n%", 1)[0]
        for env_line in ("PATH=", "LC_ALL=", "TZ="):
            if env_line not in env:
                errors.append(f"{rel}: %environment missing {env_line}")
        if "/Users/" in env or "/home/" in env:
            errors.append(f"{rel}: %environment contains user-specific path")

    if "%post" not in text:
        errors.append(f"{rel}: missing %post section")
    else:
        post = text.split("%post", 1)[1].split("\n%", 1)[0]
        first_non_empty = ""
        for ln in post.splitlines():
            if ln.strip():
                first_non_empty = ln.strip()
                break
        if "set -eux" not in first_non_empty:
            errors.append(f"{rel}: %post must start with set -eux")
        if not re.search(r"^\s*umask\s+0?22\s*$", post, re.MULTILINE):
            errors.append(f"{rel}: %post must set deterministic umask 022")
        if re.search(r"\b(read -p|select |dialog|whiptail)\b", post):
            errors.append(f"{rel}: %post contains interactive prompt constructs")

        if ("wget " in post or "curl " in post) and "NETWORK_SOURCE_VERIFIED_BY_LOCK" not in text and "sha256sum" not in post:
            errors.append(f"{rel}: network download without checksum policy marker")

        if "rm -rf /var/lib/apt/lists/*" not in post and "apt-get" in post:
            errors.append(f"{rel}: apt usage requires cleanup of /var/lib/apt/lists/*")

    m_version = re.search(r"org\.opencontainers\.image\.version\s+([^\s]+)", text)
    if m_version:
        v = m_version.group(1).strip().strip('"').lower()
        if status == "production" and v in {"latest", "latest-pinned", "main", "master", "head", "unknown", ""}:
            errors.append(f"{rel}: floating/unknown image.version '{v}' is forbidden for production tool")

    if "From:" in text:
        from_line = next((ln.strip() for ln in text.splitlines() if ln.strip().startswith("From:")), "")
        if "@sha256:" not in from_line:
            errors.append(f"{rel}: base image must be digest pinned")
        if not re.search(r"From:\s+(ubuntu|debian|python|quay\.io/)", from_line):
            errors.append(f"{rel}: base image repo must follow policy (ubuntu/debian/python/quay.io/*)")

    if "chmod 777" in text:
        errors.append(f"{rel}: chmod 777 forbidden for runtime UID safety")

    has_help_doc = "%help" in text and len(text.split("%help", 1)[1].strip()) > 0
    if "%runscript" in text:
        run = text.split("%runscript", 1)[1].split("\n%", 1)[0]
        if "--help" not in run and not has_help_doc:
            errors.append(f"{rel}: runscript/help must provide predictable --help behavior")
    else:
        errors.append(f"{rel}: missing %runscript section")

if errors:
    print("apptainer hardening: failed", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("apptainer hardening: OK")
PY
