from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path
from unittest import mock

ADAPTERS = Path(__file__).parents[1]


def load_adapter(name: str, path: Path):
    specification = importlib.util.spec_from_file_location(name, path)
    assert specification is not None and specification.loader is not None
    module = importlib.util.module_from_spec(specification)
    sys.modules[specification.name] = module
    specification.loader.exec_module(module)
    return module


materialization = load_adapter(
    "materialize_fastq_corpus_adapter", ADAPTERS / "materialize_fastq_corpus.py"
)
slurm = load_adapter(
    "corpus_materialization_slurm_adapter", ADAPTERS / "corpus_materialization_slurm.py"
)


class CorpusMaterializationAdapterTests(unittest.TestCase):
    def test_materialization_delegates_all_arguments(self) -> None:
        arguments = [
            "--plan",
            "/study/plan.json",
            "--data-root",
            "/study/data",
            "--admitted-at-utc",
            "2026-08-06T12:00:00Z",
        ]

        self.assertEqual(
            materialization.delegated_command(arguments, {}),
            ["bijux-dna", "bench", "materialize-fastq-corpus", *arguments],
        )

    def test_slurm_delegates_each_lifecycle_command(self) -> None:
        for lifecycle in ("prepare", "submit", "status", "collect"):
            with self.subTest(lifecycle=lifecycle):
                arguments = [
                    lifecycle,
                    "--specification",
                    "/study/materialization/specification.json",
                ]

                self.assertEqual(
                    slurm.delegated_command(arguments, {}),
                    ["bijux-dna", "bench", "corpus-materialization", *arguments],
                )

    def test_adapters_replace_process_with_configured_binary(self) -> None:
        environment = {"BIJUX_DNA_BIN": "/opt/bijux/bin/bijux-dna"}
        cases = (
            (materialization, ["--plan", "/study/plan.json"], "materialize-fastq-corpus"),
            (
                slurm,
                ["submit", "--specification", "/study/spec.json"],
                "corpus-materialization",
            ),
        )
        for adapter, arguments, command_name in cases:
            with self.subTest(command=command_name), mock.patch.object(
                adapter.os, "environ", environment
            ), mock.patch.object(adapter.os, "execvp") as execute:
                self.assertEqual(adapter.main(arguments), 0)
                expected = [
                    environment["BIJUX_DNA_BIN"],
                    "bench",
                    command_name,
                    *arguments,
                ]
                execute.assert_called_once_with(expected[0], expected)


if __name__ == "__main__":
    unittest.main()
