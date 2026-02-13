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
docker_dir = root / "containers/docker/arm64"
exceptions_doc = root / "containers/docker/NONROOT_EXCEPTIONS.md"
entrypoint_ex_doc = root / "containers/docker/ENTRYPOINT_EXCEPTIONS.md"

if not exceptions_doc.exists():
    print("missing containers/docker/NONROOT_EXCEPTIONS.md", file=sys.stderr)
    raise SystemExit(1)
if not entrypoint_ex_doc.exists():
    print("missing containers/docker/ENTRYPOINT_EXCEPTIONS.md", file=sys.stderr)
    raise SystemExit(1)

exceptions_text = exceptions_doc.read_text(encoding="utf-8")
entrypoint_ex_text = entrypoint_ex_doc.read_text(encoding="utf-8")
allowed = set()
entrypoint_allowed = set()
for m in re.finditer(r"\|\s*`([^`]+)`\s*\|", exceptions_text):
    allowed.add(m.group(1))
for m in re.finditer(r"\|\s*`([^`]+)`\s*\|", entrypoint_ex_text):
    entrypoint_allowed.add(m.group(1))

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
for path in sorted(docker_dir.glob("Dockerfile.*")):
    tool_id = path.name.split("Dockerfile.", 1)[1]
    text = path.read_text(encoding="utf-8")
    rel = path.relative_to(root)

    for label in required_labels:
        if label not in text:
            errors.append(f"{rel}: missing label {label}")

    if re.search(r"curl\s+[^|\n]*\|\s*(bash|sh)\b", text) or re.search(r"wget\s+[^|\n]*\|\s*(bash|sh)\b", text):
        errors.append(f"{rel}: forbidden curl|bash or wget|sh pattern")

    first_from = next((ln.strip() for ln in text.splitlines() if ln.strip().startswith("FROM ")), "")
    if "@sha256:" not in first_from:
        errors.append(f"{rel}: FROM must be digest-pinned")

    # ENTRYPOINT/CMD contract
    has_entrypoint = bool(re.search(r"^ENTRYPOINT\s+\[", text, flags=re.M))
    has_cmd = bool(re.search(r"^CMD\s+\[", text, flags=re.M))
    entrypoint_exempt = tool_id in entrypoint_allowed or "*" in entrypoint_allowed
    if not has_cmd and not entrypoint_exempt:
        errors.append(f"{rel}: missing JSON-form CMD")
    elif has_cmd:
        cmd_line = re.search(r"^CMD\s+\[(.+)\]\s*$", text, flags=re.M)
        cmd_txt = cmd_line.group(1).lower() if cmd_line else ""
        if not any(x in cmd_txt for x in ("--help", "-h", "--version")) and not entrypoint_exempt:
            errors.append(f"{rel}: CMD should default to --help/-h/--version")
    if has_entrypoint and not entrypoint_exempt:
        errors.append(f"{rel}: ENTRYPOINT is forbidden unless listed in ENTRYPOINT_EXCEPTIONS.md")
    if re.search(r'^ENTRYPOINT\s+\["/bin/sh",\s*"-c"', text, flags=re.M) and not entrypoint_exempt:
        errors.append(f"{rel}: ENTRYPOINT must not use /bin/sh -c wrapper")

    # Non-root policy
    user_lines = re.findall(r"^USER\s+(.+)$", text, flags=re.M)
    nonroot = any(u.strip() not in ("root", "0") for u in user_lines)
    if not nonroot:
        if tool_id not in allowed and "*" not in allowed:
            errors.append(f"{rel}: no non-root USER and not listed in NONROOT_EXCEPTIONS.md")

    # Healthcheck policy
    if "HEALTHCHECK" in text:
        m = re.search(r"^HEALTHCHECK\s+(.+)$", text, flags=re.M)
        line = m.group(1) if m else ""
        if "--interval=" not in line or "--timeout=" not in line:
            errors.append(f"{rel}: HEALTHCHECK must define --interval and --timeout")
        if "--version" not in text and "healthcheck" not in text.lower():
            errors.append(f"{rel}: HEALTHCHECK should verify tool --version or explicit health check")

print("docker hardening: OK" if not errors else "docker hardening: failed", file=sys.stderr if errors else sys.stdout)
if errors:
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
PY
