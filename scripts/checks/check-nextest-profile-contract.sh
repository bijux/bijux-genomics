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
cfg = root / "configs/nextest/nextest.toml"
mk = root / "makefiles/cargo.mk"
ci_test = root / "scripts/tooling/ci-test.sh"
ci_cov = root / "scripts/tooling/ci-coverage.sh"

profiles = set(re.findall(r"^\[profile\.([a-zA-Z0-9_-]+)\]\s*$", cfg.read_text(encoding="utf-8"), flags=re.MULTILINE))
if not profiles:
    raise SystemExit("nextest-profile-contract: no [profile.*] sections found")

errors = []
mk_text = mk.read_text(encoding="utf-8")
if 'NEXTEST_PROFILE ?= ci' not in mk_text:
    errors.append("makefiles/cargo.mk: must set default `NEXTEST_PROFILE ?= ci`")
if '--profile ${nextest_profile}' not in ci_test.read_text(encoding="utf-8"):
    errors.append("scripts/tooling/ci-test.sh: cargo nextest must use --profile ${nextest_profile}")
if '--profile ${nextest_profile}' not in ci_cov.read_text(encoding="utf-8"):
    errors.append("scripts/tooling/ci-coverage.sh: cargo llvm-cov nextest must use --profile ${nextest_profile}")

if errors:
    print("nextest-profile-contract: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("nextest-profile-contract: OK")
PY
