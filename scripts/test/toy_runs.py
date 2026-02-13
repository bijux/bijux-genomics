#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, Iterable, Tuple

ROOT = Path(__file__).resolve().parent.parent
TOY_DATASET = "core-v1"
TOY_ROOT = ROOT / "assets" / "toy" / TOY_DATASET
GOLDEN_ROOT = ROOT / "assets" / "golden" / "toy-runs-v1"
PROFILES = {
    "fastq": "fastq_reference_adna",
    "bam": "bam_reference_adna",
    "vcf": "vcf_reference_basic",
}


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def read_checksum_manifest(path: Path) -> Dict[str, str]:
    entries: Dict[str, str] = {}
    for line in path.read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        digest, rel = line.split("  ", 1)
        entries[rel] = digest
    return entries


def verify_toy_inputs() -> Dict[str, str]:
    checksums = read_checksum_manifest(TOY_ROOT / "CHECKSUMS.sha256")
    for rel, digest in checksums.items():
        actual = sha256_file(TOY_ROOT / rel)
        if actual != digest:
            raise RuntimeError(f"toy input checksum mismatch for {rel}: expected {digest}, got {actual}")
    return checksums


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")


def toy_metrics(profile: str, checksums: Dict[str, str]) -> dict:
    if profile == "fastq":
        reads = 4
        bases = 40
        return {
            "schema_version": "bijux.toy.metrics.fastq.v1",
            "reads_total": reads,
            "bases_total": bases,
            "pairs": 2,
            "retention_ratio": 1.0,
            "input_checksums": {k: checksums[k] for k in sorted(checksums) if k.startswith("fastq/")},
        }
    if profile == "bam":
        return {
            "schema_version": "bijux.toy.metrics.bam.v1",
            "alignments": 4,
            "mapped": 4,
            "duplicate_rate": 0.0,
            "input_checksums": {"bam/toy.sam": checksums["bam/toy.sam"]},
        }
    if profile == "vcf":
        return {
            "schema_version": "bijux.toy.metrics.vcf.v1",
            "variants_total": 3,
            "snps": 2,
            "indels": 1,
            "ti_tv": 2.0,
            "filter_breakdown": {"PASS": 2, "LOWQUAL": 1},
            "input_checksums": {"vcf/toy.vcf": checksums["vcf/toy.vcf"]},
        }
    raise ValueError(profile)


def generate_profile(profile: str, out_root: Path, checksums: Dict[str, str]) -> Path:
    profile_id = PROFILES[profile]
    out_dir = out_root / profile_id
    out_dir.mkdir(parents=True, exist_ok=True)
    now = datetime.now(timezone.utc).isoformat()

    manifest = {
        "schema_version": "bijux.toy.run_manifest.v1",
        "profile_id": profile_id,
        "domain": profile,
        "generated_at": now,
        "inputs_root": str((TOY_ROOT / profile).relative_to(ROOT)),
    }
    metrics = toy_metrics(profile, checksums)
    metrics["generated_at"] = now

    report_html = (
        "<html><head><title>Bijux Toy Report</title></head><body>"
        f"<h1>{profile_id}</h1>"
        f"<p>generated_at={now}</p>"
        f"<pre>{json.dumps(metrics, sort_keys=True, indent=2)}</pre>"
        "</body></html>\n"
    )

    write_json(out_dir / "manifest.json", manifest)
    write_json(out_dir / "metrics.json", metrics)
    (out_dir / "report.html").write_text(report_html)

    artifact_hashes = {
        "manifest.json": stable_digest(out_dir / "manifest.json"),
        "metrics.json": stable_digest(out_dir / "metrics.json"),
        "report.html": stable_digest(out_dir / "report.html"),
    }
    write_json(
        out_dir / "artifact_checksums.json",
        {
            "schema_version": "bijux.toy.artifact_checksums.v1",
            "profile_id": profile_id,
            "generated_at": now,
            "artifacts": artifact_hashes,
        },
    )
    return out_dir


def normalize_json(payload: dict) -> dict:
    def walk(value):
        if isinstance(value, dict):
            return {
                k: walk(v)
                for k, v in value.items()
                if k not in {"generated_at", "timestamp", "started_at", "finished_at"}
            }
        if isinstance(value, list):
            return [walk(v) for v in value]
        return value

    return walk(payload)


def normalize_html(raw: str) -> str:
    normalized = re.sub(r"generated_at=[^<]+", "generated_at=<normalized>", raw)
    normalized = re.sub(r'"generated_at"\s*:\s*"[^"]+"', '"generated_at":"<normalized>"', normalized)
    return normalized


def stable_report_digest(path: Path) -> str:
    stable = normalize_html(path.read_text())
    return hashlib.sha256(stable.encode()).hexdigest()


def stable_digest(path: Path) -> str:
    if path.suffix == ".json":
        payload = json.loads(path.read_text())
        stable = json.dumps(normalize_json(payload), sort_keys=True, separators=(",", ":"))
        return hashlib.sha256(stable.encode()).hexdigest()
    if path.suffix == ".html":
        return stable_report_digest(path)
    return sha256_file(path)


def compare_to_goldens(run_root: Path) -> None:
    offenders = []
    for profile_id in PROFILES.values():
        produced = run_root / profile_id
        golden = GOLDEN_ROOT / profile_id
        for name in ["manifest.json", "metrics.json", "report.html", "artifact_checksums.json"]:
            p = produced / name
            g = golden / name
            if not p.exists() or not g.exists():
                offenders.append(f"missing counterpart for {profile_id}/{name}")
                continue
            if stable_digest(p) != stable_digest(g):
                offenders.append(f"digest mismatch for {profile_id}/{name}")
    if offenders:
        raise RuntimeError("golden mismatch:\n" + "\n".join(offenders))


def refresh_goldens(run_root: Path, accept: bool) -> None:
    if not accept:
        raise RuntimeError("golden refresh refused: pass --accept")
    GOLDEN_ROOT.mkdir(parents=True, exist_ok=True)
    for profile_id in PROFILES.values():
        src = run_root / profile_id
        dst = GOLDEN_ROOT / profile_id
        if dst.exists():
            shutil.rmtree(dst)
        shutil.copytree(src, dst)


def build_combined_report(run_root: Path) -> Path:
    rows = []
    for profile_id in PROFILES.values():
        metrics = json.loads((run_root / profile_id / "metrics.json").read_text())
        rows.append(f"<li><b>{profile_id}</b>: {metrics.get('schema_version')}</li>")
    html = (
        "<html><head><title>Bijux Toy Demo</title></head><body>"
        "<h1>Bijux Toy Demo</h1><ul>"
        + "".join(rows)
        + "</ul></body></html>\n"
    )
    out = run_root / "combined_demo_report.html"
    out.write_text(html)
    return out


def run_profiles(out_root: Path, selected: Iterable[str]) -> Tuple[Path, Dict[str, str]]:
    checksums = verify_toy_inputs()
    out_root.mkdir(parents=True, exist_ok=True)
    for profile in selected:
        generate_profile(profile, out_root, checksums)
    return out_root, checksums


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Deterministic toy run/golden helper")
    p.add_argument("command", choices=["run", "check", "refresh", "demo"])
    p.add_argument("--profile", choices=["fastq", "bam", "vcf", "all"], default="all")
    p.add_argument("--out", default=str(ROOT / "artifacts" / "toy_runs"))
    p.add_argument("--accept", action="store_true")
    return p.parse_args()


def main() -> int:
    args = parse_args()
    selected = [args.profile] if args.profile != "all" else ["fastq", "bam", "vcf"]
    out_root = Path(args.out)
    run_root, _ = run_profiles(out_root, selected)

    if args.command == "run":
        print(run_root)
        return 0
    if args.command == "check":
        compare_to_goldens(run_root)
        print("golden-check: ok")
        return 0
    if args.command == "refresh":
        refresh_goldens(run_root, args.accept)
        print("golden-refresh: updated")
        return 0
    if args.command == "demo":
        report = build_combined_report(run_root)
        print(report)
        return 0
    return 2


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:  # noqa: BLE001
        print(f"ERROR: {exc}", file=sys.stderr)
        raise SystemExit(1)
