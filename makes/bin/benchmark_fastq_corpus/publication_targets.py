from __future__ import annotations

from benchmark_fastq_corpus.support import (
    CORPUS_01_PUBLICATION_CONTRACTS,
    corpus_01_make_report_target,
    corpus_01_make_run_target,
)


def resolve_targets(kind: str) -> list[str]:
    target_builder = (
        corpus_01_make_report_target if kind == "report" else corpus_01_make_run_target
    )
    return [target_builder(contract.stage_id) for contract in CORPUS_01_PUBLICATION_CONTRACTS]
