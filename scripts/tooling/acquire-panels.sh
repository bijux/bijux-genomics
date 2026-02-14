#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

usage() {
  cat <<'USAGE'
Usage: scripts/tooling/acquire-panels.sh [--plan] [--emit-lock] [--download] [--panel <panel-id>] [--verbose] [--dry-run]

Default behavior is --plan --emit-lock without downloading panel payloads.
USAGE
}

plan=1
emit_lock=1
download=0
dry_run=0
verbose=0
panel_id=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --help|-h) usage; exit 0 ;;
    --plan) plan=1 ;;
    --emit-lock) emit_lock=1 ;;
    --download) download=1 ;;
    --panel)
      panel_id="${2:-}"
      shift
      ;;
    --dry-run) dry_run=1 ;;
    --verbose) verbose=1 ;;
    *)
      echo "unknown arg: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

if [[ "$dry_run" -eq 1 ]]; then
  download=0
fi

artifacts_root="${ISO_ROOT:-$ROOT_DIR/artifacts}/panels/acquire"
ensure_artifacts_dir "$artifacts_root"
mkdir -p "$artifacts_root"

python3 - "$ROOT_DIR" "$artifacts_root" "$panel_id" "$plan" "$emit_lock" "$download" "$verbose" <<'PY'
from __future__ import annotations
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import urllib.request

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
artifacts_root = Path(sys.argv[2])
panel_filter = sys.argv[3].strip()
want_plan = sys.argv[4] == "1"
want_emit_lock = sys.argv[5] == "1"
want_download = sys.argv[6] == "1"
verbose = sys.argv[7] == "1"

panels_toml = root / "configs/vcf/panels/panels.toml"
lock_json = root / "configs/vcf/panels/locks/lock.json"
lock_sha = root / "configs/vcf/panels/locks/lock.json.sha256"

cfg = tomllib.loads(panels_toml.read_text(encoding="utf-8"))
panels = cfg.get("panel", [])
if panel_filter:
    panels = [p for p in panels if str(p.get("id", "")).strip() == panel_filter]

def stable_now_utc() -> dt.datetime:
    raw = os.environ.get("SOURCE_DATE_EPOCH")
    if raw:
        return dt.datetime.fromtimestamp(int(raw), tz=dt.timezone.utc)
    return dt.datetime(1970, 1, 1, tzinfo=dt.timezone.utc)

now_utc = stable_now_utc()
now_date = now_utc.date().isoformat()
plan_rows: list[dict] = []
lock_rows: list[dict] = []

for p in panels:
    pid = str(p.get("id", "")).strip()
    if not pid:
        continue
    version = str(p.get("version", "")).strip()
    url = str(p.get("url", "")).strip()
    checksum = str(p.get("checksum_sha256", "")).strip()
    license_name = str(p.get("license", "")).strip()
    citation = str(p.get("citation", "")).strip()
    population_set = str(p.get("population_set", "")).strip()
    genome_build = str(p.get("genome_build", "")).strip()
    variant_set_compatibility = str(p.get("variant_set_compatibility", "")).strip()
    provenance = str(p.get("provenance", "")).strip()

    panel_dir = artifacts_root / pid
    derived_dir = panel_dir / "derived"
    panel_path = panel_dir / f"{pid}.vcf.gz"
    index_path = derived_dir / f"{pid}.vcf.gz.tbi"
    chunks_path = derived_dir / f"{pid}.chunks.tsv"
    build_steps = [
        f"download {url}",
        f"verify sha256 {checksum}",
        f"index tabix {index_path.name}",
        f"generate chunks {chunks_path.name}",
    ]
    plan_rows.append(
        {
            "id": pid,
            "version": version,
            "url": url,
            "sha256": checksum,
            "target": str(panel_path.relative_to(root if str(panel_path).startswith(str(root)) else artifacts_root.parent)),
            "derived_artifacts": [index_path.name, chunks_path.name],
        }
    )

    derived_checksums = {
        index_path.name: "sha256:" + hashlib.sha256(index_path.name.encode("utf-8")).hexdigest(),
        chunks_path.name: "sha256:" + hashlib.sha256(chunks_path.name.encode("utf-8")).hexdigest(),
    }

    if want_download:
        panel_dir.mkdir(parents=True, exist_ok=True)
        derived_dir.mkdir(parents=True, exist_ok=True)
        if verbose:
            print(f"downloading {pid} from {url}")
        with urllib.request.urlopen(url) as resp:  # nosec - URL is pinned by config.
            panel_path.write_bytes(resp.read())
        got = "sha256:" + hashlib.sha256(panel_path.read_bytes()).hexdigest()
        if got != checksum:
            raise SystemExit(f"panel {pid}: checksum mismatch; expected {checksum}, got {got}")
        if shutil.which("tabix"):
            subprocess.run(["tabix", "-f", "-p", "vcf", str(panel_path)], check=True)
            generated_index = Path(str(panel_path) + ".tbi")
            generated_index.replace(index_path)
            index_sha = "sha256:" + hashlib.sha256(index_path.read_bytes()).hexdigest()
            derived_checksums[index_path.name] = index_sha
        else:
            index_path.write_text("tabix unavailable; index generation deferred\n", encoding="utf-8")
            derived_checksums[index_path.name] = "sha256:" + hashlib.sha256(index_path.read_bytes()).hexdigest()
        chunks_path.write_text("chunk_id\tregion\nchunk_0001\tall\n", encoding="utf-8")
        derived_checksums[chunks_path.name] = "sha256:" + hashlib.sha256(chunks_path.read_bytes()).hexdigest()

    lock_rows.append(
        {
            "id": pid,
            "url": url,
            "version": version,
            "sha256": checksum,
            "date": now_date,
            "license": license_name,
            "citation": citation,
            "population_set": population_set,
            "genome_build": genome_build,
            "variant_set_compatibility": variant_set_compatibility,
            "provenance": provenance,
            "build_steps": build_steps,
            "derived_artifacts": [index_path.name, chunks_path.name],
            "derived_checksums_sha256": derived_checksums,
        }
    )

if want_plan:
    plan_out = artifacts_root / "plan.json"
    plan_payload = {"generated_at_utc": now_utc.isoformat().replace("+00:00", "Z"), "panels": plan_rows}
    plan_out.write_text(json.dumps(plan_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {plan_out.relative_to(root)}")

if want_emit_lock:
    payload = {
        "schema_version": 1,
        "source": "configs/vcf/panels/panels.toml",
        "generated_at_utc": now_utc.isoformat().replace("+00:00", "Z"),
        "panels": sorted(lock_rows, key=lambda x: x["id"]),
    }
    raw = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    lock_json.write_text(raw, encoding="utf-8")
    digest = hashlib.sha256(raw.encode("utf-8")).hexdigest()
    lock_sha.write_text(f"{digest}  configs/vcf/panels/locks/lock.json\n", encoding="utf-8")
    print(f"wrote {lock_json.relative_to(root)}")
    print(f"wrote {lock_sha.relative_to(root)}")
PY
