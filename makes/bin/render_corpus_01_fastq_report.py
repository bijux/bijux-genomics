#!/usr/bin/env python3
from __future__ import annotations

import argparse
import importlib
import sys


def parse_args() -> tuple[str, list[str]]:
    parser = argparse.ArgumentParser(
        description="Dispatch a corpus-01 FASTQ benchmark report renderer by stage id."
    )
    parser.add_argument("--stage", required=True)
    args, remaining = parser.parse_known_args()
    return args.stage, remaining


def module_name_for_stage(stage_id: str) -> str:
    return f"render_{stage_id.replace('.', '_')}_corpus_01_report"


def main() -> int:
    stage_id, remaining = parse_args()
    module = importlib.import_module(module_name_for_stage(stage_id))
    sys.argv = [module.__file__ or module.__name__, *remaining]
    return int(module.main())


if __name__ == "__main__":
    raise SystemExit(main())
