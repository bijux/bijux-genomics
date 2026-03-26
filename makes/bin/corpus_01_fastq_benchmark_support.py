#!/usr/bin/env python3
from __future__ import annotations

import json
from collections import defaultdict
from pathlib import Path

try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:
    tomllib = None


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def parse_simple_toml(path: Path) -> dict:
    root: dict = {}
    samples: list[dict] = []
    current_sample: dict | None = None
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.split("#", 1)[0].strip()
        if not line:
            continue
        if line == "[[samples]]":
            current_sample = {}
            samples.append(current_sample)
            continue
        key, raw_value = [part.strip() for part in line.split("=", 1)]
        if raw_value.startswith('"') and raw_value.endswith('"'):
            value = raw_value[1:-1]
        elif raw_value.isdigit():
            value = int(raw_value)
        else:
            raise SystemExit(f"unsupported TOML value in {path}: {raw_value}")
        target = current_sample if current_sample is not None else root
        target[key] = value
    root["samples"] = samples
    return root


def load_corpus_spec(repo_root: Path) -> dict:
    path = repo_root / "configs" / "runtime" / "corpora" / "corpus-01.toml"
    if tomllib is not None:
        with path.open("rb") as handle:
            return tomllib.load(handle)
    return parse_simple_toml(path)


def discover_normalized_samples(
    corpus_root: Path,
    *,
    expected_total: int = 20,
) -> list[dict]:
    normalized = corpus_root / "normalized"
    if not normalized.is_dir():
        raise SystemExit(f"missing normalized corpus directory: {normalized}")

    sample_ids: set[str] = set()
    for path in normalized.glob("sample_*_R1.fastq.gz"):
        sample_ids.add(path.name.removesuffix("_R1.fastq.gz"))
    for path in normalized.glob("sample_*_R2.fastq.gz"):
        sample_ids.add(path.name.removesuffix("_R2.fastq.gz"))

    samples: list[dict] = []
    for sample_id in sorted(sample_ids):
        r1 = normalized / f"{sample_id}_R1.fastq.gz"
        r2 = normalized / f"{sample_id}_R2.fastq.gz"
        if not r1.is_file():
            raise SystemExit(f"missing R1 for sample {sample_id}: {r1}")
        samples.append(
            {
                "sample_id": sample_id,
                "r1": r1,
                "r2": r2 if r2.is_file() else None,
                "layout": "pe" if r2.is_file() else "se",
            }
        )
    if len(samples) != expected_total:
        raise SystemExit(
            f"expected {expected_total} normalized samples for corpus-01, found {len(samples)}"
        )
    return samples


def sample_accession_map(corpus_root: Path, spec: dict) -> dict[str, dict]:
    manifest = load_json(corpus_root / "MANIFEST.json")
    manifest_files = manifest["files"]
    hash_to_accessions: dict[str, set[str]] = defaultdict(set)
    for relative_path, digest in manifest_files.items():
        parts = Path(relative_path).parts
        if len(parts) >= 2 and parts[0] == "raw":
            hash_to_accessions[digest].add(parts[1])

    spec_by_accession = {sample["accession"]: sample for sample in spec["samples"]}
    sample_map: dict[str, dict] = {}
    for relative_path, digest in manifest_files.items():
        path = Path(relative_path)
        if len(path.parts) != 2 or path.parts[0] != "normalized":
            continue
        filename = path.name
        if filename.endswith("_R1.fastq.gz"):
            sample_id = filename.removesuffix("_R1.fastq.gz")
        elif filename.endswith("_R2.fastq.gz"):
            sample_id = filename.removesuffix("_R2.fastq.gz")
        else:
            continue
        accessions = sorted(hash_to_accessions.get(digest, set()))
        if len(accessions) != 1:
            raise SystemExit(
                f"expected one accession for {relative_path}, found {accessions or 'none'}"
            )
        accession = accessions[0]
        sample_map.setdefault(sample_id, {}).update(spec_by_accession[accession])
        sample_map[sample_id]["accession"] = accession
    return sample_map


def expected_cohort_counts(spec: dict) -> dict[str, int]:
    return {
        "ancient_se": int(spec["target_ancient_se"]),
        "ancient_pe": int(spec["target_ancient_pe"]),
        "modern_se": int(spec["target_modern_se"]),
        "modern_pe": int(spec["target_modern_pe"]),
    }


def validate_corpus_contract(
    corpus_root: Path,
    spec: dict,
    samples: list[dict],
) -> dict[str, dict]:
    metadata_by_sample = sample_accession_map(corpus_root, spec)
    actual_counts: dict[str, int] = defaultdict(int)

    for sample in samples:
        sample_id = sample["sample_id"]
        metadata = metadata_by_sample.get(sample_id)
        if metadata is None:
            raise SystemExit(f"missing accession metadata for normalized sample {sample_id}")
        cohort = f"{metadata['era']}_{metadata['layout']}"
        actual_counts[cohort] += 1

    expected_counts = expected_cohort_counts(spec)
    if dict(sorted(actual_counts.items())) != dict(sorted(expected_counts.items())):
        raise SystemExit(
            "corpus-01 cohort contract drift: "
            f"expected {dict(sorted(expected_counts.items()))}, "
            f"found {dict(sorted(actual_counts.items()))}"
        )
    return metadata_by_sample
