from __future__ import annotations

import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]
BIN_DIR = ROOT / "makes" / "bin"
if str(BIN_DIR) not in sys.path:
    sys.path.insert(0, str(BIN_DIR))

import corpus_01_fastq_benchmark_support as support
import audit_corpus_01_fastq_benchmark_docs as benchmark_docs_audit
import audit_published_corpus_01_fastq_results as published_results_audit
import run_fastq_correct_errors_corpus_01 as correct_errors_runner
import run_fastq_deplete_host_corpus_01 as deplete_host_runner
import run_fastq_deplete_reference_contaminants_corpus_01 as deplete_reference_contaminants_runner
import run_fastq_deplete_rrna_corpus_01 as deplete_rrna_runner
import run_fastq_extract_umis_corpus_01 as extract_umis_runner
import run_fastq_screen_taxonomy_corpus_01 as screen_taxonomy_runner
import run_fastq_filter_reads_corpus_01 as filter_reads_runner
import run_fastq_filter_low_complexity_corpus_01 as filter_low_complexity_runner
import run_fastq_normalize_primers_corpus_01 as normalize_primers_runner
import run_fastq_remove_duplicates_corpus_01 as remove_duplicates_runner
import run_fastq_merge_pairs_corpus_01 as merge_runner
import run_fastq_trim_reads_corpus_01 as trim_reads_runner
import run_fastq_trim_terminal_damage_corpus_01 as terminal_damage_runner
import render_fastq_detect_adapters_corpus_01_briefing as detect_adapters_briefing
import render_fastq_detect_adapters_corpus_01_report as detect_adapters_report
import render_fastq_correct_errors_corpus_01_briefing as correct_errors_briefing
import render_fastq_correct_errors_corpus_01_report as correct_errors_report
import render_fastq_deplete_host_corpus_01_briefing as deplete_host_briefing
import render_fastq_deplete_host_corpus_01_report as deplete_host_report
import render_fastq_deplete_reference_contaminants_corpus_01_briefing as deplete_reference_contaminants_briefing
import render_fastq_deplete_reference_contaminants_corpus_01_report as deplete_reference_contaminants_report
import render_fastq_deplete_rrna_corpus_01_briefing as deplete_rrna_briefing
import render_fastq_deplete_rrna_corpus_01_report as deplete_rrna_report
import render_fastq_extract_umis_corpus_01_briefing as extract_umis_briefing
import render_fastq_extract_umis_corpus_01_report as extract_umis_report
import render_fastq_screen_taxonomy_corpus_01_briefing as screen_taxonomy_briefing
import render_fastq_screen_taxonomy_corpus_01_report as screen_taxonomy_report
import render_fastq_filter_reads_corpus_01_briefing as filter_reads_briefing
import render_fastq_filter_reads_corpus_01_report as filter_reads_report
import render_fastq_filter_low_complexity_corpus_01_briefing as filter_low_complexity_briefing
import render_fastq_filter_low_complexity_corpus_01_report as filter_low_complexity_report
import render_fastq_normalize_primers_corpus_01_briefing as normalize_primers_briefing
import render_fastq_normalize_primers_corpus_01_report as normalize_primers_report
import render_fastq_remove_duplicates_corpus_01_briefing as remove_duplicates_briefing
import render_fastq_remove_duplicates_corpus_01_report as remove_duplicates_report
import render_fastq_merge_pairs_corpus_01_briefing as merge_briefing
import render_fastq_merge_pairs_corpus_01_report as merge_report
import render_fastq_profile_overrepresented_sequences_corpus_01_briefing as overrepresented_briefing
import render_fastq_profile_overrepresented_sequences_corpus_01_report as overrepresented_report
import render_fastq_report_qc_corpus_01_briefing as report_qc_briefing
import render_fastq_report_qc_corpus_01_report as report_qc_report
import render_fastq_profile_read_lengths_corpus_01_briefing as profile_read_lengths_briefing
import render_fastq_profile_read_lengths_corpus_01_report as profile_read_lengths_report
import render_fastq_profile_reads_corpus_01_briefing as profile_reads_briefing
import render_fastq_profile_reads_corpus_01_report as profile_reads_report
import render_fastq_trim_reads_corpus_01_briefing as trim_reads_briefing
import render_fastq_trim_reads_corpus_01_report as trim_reads_report
import render_fastq_trim_terminal_damage_corpus_01_briefing as terminal_damage_briefing
import render_fastq_trim_terminal_damage_corpus_01_report as terminal_damage_report
import render_fastq_trim_polyg_tails_corpus_01_briefing as trim_polyg_briefing
import render_fastq_trim_polyg_tails_corpus_01_report as trim_polyg_report
import render_fastq_validate_reads_corpus_01_report as validate_reads_report
import normalize_lunarc_results_mirror as normalize_results_mirror
import repair_corpus_01_fastq_result_manifests as repair_results_manifests
import bootstrap_fastq_screen_taxonomy_database as taxonomy_db_bootstrap


class CorpusBenchmarkSupportTests(unittest.TestCase):
    def test_trim_reads_defaults_match_governed_suite(self) -> None:
        defaults = support.trim_reads_benchmark_defaults()
        self.assertIsNone(defaults["min_length"])
        self.assertIsNone(defaults["quality_cutoff"])
        self.assertEqual(defaults["n_policy"], "retain")
        self.assertEqual(defaults["adapter_policy"], "none")
        self.assertEqual(defaults["polyx_policy"], "none")
        self.assertEqual(defaults["contaminant_policy"], "none")

    def test_resolve_benchmark_tool_roster_falls_back_without_registry_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tools, error = support.resolve_benchmark_tool_roster(
                Path(tmpdir),
                "fastq.trim_reads",
                "trim_fairness",
                ["fastp"],
            )

        self.assertEqual(tools, ["fastp"])
        self.assertIsNone(error)

    def test_resolve_stage_toolset_falls_back_without_registry_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tools, error = support.resolve_stage_toolset(
                Path(tmpdir),
                "fastq.trim_reads",
                ["fastp"],
            )

        self.assertEqual(tools, ["fastp"])
        self.assertIsNone(error)

    def test_filter_reads_defaults_match_governed_suite(self) -> None:
        defaults = support.filter_reads_benchmark_defaults()
        self.assertEqual(defaults["threads"], 8)
        self.assertEqual(defaults["max_n"], 0)
        self.assertIsNone(defaults["max_n_fraction"])
        self.assertEqual(defaults["max_n_count"], 3)
        self.assertEqual(defaults["low_complexity_threshold"], 20.0)
        self.assertEqual(defaults["entropy_threshold"], 18.0)
        self.assertIsNone(defaults["kmer_ref"])
        self.assertEqual(defaults["polyx_policy"], "trim")

    def test_filter_low_complexity_defaults_match_governed_suite(self) -> None:
        defaults = support.filter_low_complexity_benchmark_defaults()
        self.assertEqual(defaults["entropy_threshold"], 0.55)
        self.assertIsNone(defaults["polyx_threshold"])

    def test_remove_duplicates_defaults_match_governed_suite(self) -> None:
        defaults = support.remove_duplicates_benchmark_defaults()
        self.assertEqual(defaults["dedup_mode"], "exact")
        self.assertTrue(defaults["keep_order"])

    def test_normalize_primers_defaults_match_governed_suite(self) -> None:
        defaults = support.normalize_primers_benchmark_defaults()
        self.assertEqual(defaults["primer_set_id"], "16S_universal_v1")
        self.assertEqual(
            defaults["orientation_policy"], "normalize_to_forward_primer"
        )
        self.assertAlmostEqual(defaults["max_mismatch_rate"], 0.10)
        self.assertEqual(defaults["min_overlap_bp"], 10)
        self.assertTrue(defaults["strict_5p_anchor"])
        self.assertTrue(defaults["allow_iupac_codes"])

    def test_deplete_rrna_defaults_match_governed_suite(self) -> None:
        defaults = support.deplete_rrna_benchmark_defaults()
        self.assertEqual(defaults["threads"], 4)
        self.assertEqual(defaults["rrna_bundle_id"], "sortmerna_v4_3_default_db")
        self.assertAlmostEqual(defaults["min_identity"], 0.95)

    def test_deplete_host_defaults_match_governed_suite(self) -> None:
        defaults = support.deplete_host_benchmark_defaults()
        self.assertEqual(defaults["threads"], 8)
        self.assertEqual(defaults["reference_catalog_id"], "host_reference")
        self.assertEqual(defaults["reference_index_backend"], "bowtie2_build")
        self.assertAlmostEqual(defaults["host_identity_threshold"], 0.95)
        self.assertTrue(defaults["retain_unmapped_only"])

    def test_benchmark_remote_corpus_root_prefers_workspace_config(self) -> None:
        with mock.patch.object(
            support,
            "load_benchmark_workspace_config",
            return_value={"remote": {"corpus_root": "/srv/bench/corpus_01"}},
        ):
            self.assertEqual(
                support.benchmark_remote_corpus_root(),
                Path("/srv/bench/corpus_01"),
            )

    def test_load_benchmark_workspace_config_reads_committed_contract(self) -> None:
        support.load_benchmark_workspace_config.cache_clear()
        try:
            config = support.load_benchmark_workspace_config()
        finally:
            support.load_benchmark_workspace_config.cache_clear()

        self.assertEqual(
            config.get("remote", {}).get("corpus_root"),
            "/home/bijan/lu2024-12-24/.cache/corpus_01",
        )
        self.assertEqual(
            support.benchmark_remote_corpus_root(),
            Path("/home/bijan/lu2024-12-24/.cache/corpus_01"),
        )

    def test_fastq_report_parsers_default_corpus_root_from_workspace_contract(self) -> None:
        original_argv = sys.argv
        try:
            sys.argv = ["render"]
            expected = str(support.benchmark_remote_corpus_root())
            for module in [
                correct_errors_report,
                filter_reads_report,
                merge_report,
                trim_reads_report,
                validate_reads_report,
            ]:
                self.assertEqual(module.parse_args().corpus_root, expected)
        finally:
            sys.argv = original_argv

    def test_default_host_reference_index_root_prefers_cache_extra_data(self) -> None:
        out_root = Path(
            "/home/bijan/lu2024-12-24/.cache/bijux-dna-results/corpus_01/fastq.deplete_host/lunarc"
        )
        self.assertEqual(
            support.default_host_reference_index_root(
                out_root,
                reference_catalog_id="host_reference",
                reference_index_backend="bowtie2_build",
            ).resolve(),
            Path(
                "/home/bijan/lu2024-12-24/.cache/extra-data/benchmark/fastq.deplete_host/host_reference/bowtie2_build/index"
            ).resolve(),
        )

    def test_benchmark_runtime_env_overrides_stale_cache_layout(self) -> None:
        out_root = Path(
            "/home/bijan/lu2024-12-24/.cache/bijux-dna-results/corpus_01/fastq.trim_reads/lunarc"
        )
        stale_cache_root = os.environ.get("BIJUX_CACHE_ROOT")
        stale_xdg_cache = os.environ.get("XDG_CACHE_HOME")
        stale_hpc_root = os.environ.get("BIJUX_HPC_ROOT")
        try:
            os.environ["BIJUX_CACHE_ROOT"] = "/tmp/stale-cache-root"
            os.environ["XDG_CACHE_HOME"] = "/tmp/stale-xdg-cache"
            os.environ["BIJUX_HPC_ROOT"] = "/tmp/stale-hpc-root"

            env = support.benchmark_runtime_env(out_root)
            expected_cache_root = support.infer_cache_root(out_root)
            self.assertIsNotNone(expected_cache_root)
            expected_cache_root = expected_cache_root.resolve()
            expected_hpc_root = expected_cache_root.parent

            self.assertEqual(Path(env["BIJUX_CACHE_ROOT"]).resolve(), expected_cache_root)
            self.assertEqual(Path(env["XDG_CACHE_HOME"]).resolve(), expected_cache_root)
            self.assertEqual(Path(env["BIJUX_HPC_ROOT"]).resolve(), expected_hpc_root)
        finally:
            if stale_cache_root is None:
                os.environ.pop("BIJUX_CACHE_ROOT", None)
            else:
                os.environ["BIJUX_CACHE_ROOT"] = stale_cache_root
            if stale_xdg_cache is None:
                os.environ.pop("XDG_CACHE_HOME", None)
            else:
                os.environ["XDG_CACHE_HOME"] = stale_xdg_cache
            if stale_hpc_root is None:
                os.environ.pop("BIJUX_HPC_ROOT", None)
            else:
                os.environ["BIJUX_HPC_ROOT"] = stale_hpc_root

    def test_localize_results_path_supports_cache_results_root(self) -> None:
        localized = support.localize_results_path(
            "/lunarc/nobackup/projects/snic2019-34-3/.cache/bijux-dna-results/corpus_01/fastq.extract_umis/lunarc/bench/extract_umis/sample_0001/report.json",
            Path("/tmp/local-results"),
        )

        self.assertEqual(
            localized,
            Path("/tmp/local-results")
            / "corpus_01/fastq.extract_umis/lunarc/bench/extract_umis/sample_0001/report.json",
        )

    def test_preferred_report_run_root_falls_back_to_legacy_local_archive(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_root = Path(tmpdir)
            corpus_root = Path("/home/bijan/lu2024-12-24/.cache/corpus_01")
            legacy_root = tmp_root / "legacy-results"
            cache_mirror_root = tmp_root / "cache-mirror"
            remote_root = tmp_root / "remote-results"
            stage_id = "fastq.report_qc"
            (legacy_root / "corpus_01" / stage_id / "lunarc").mkdir(parents=True)
            (remote_root / "corpus_01" / stage_id / "lunarc").mkdir(parents=True)

            with mock.patch.object(
                support,
                "benchmark_local_results_root",
                return_value=legacy_root,
            ), mock.patch.object(
                support,
                "benchmark_local_cache_mirror_root",
                return_value=cache_mirror_root,
            ), mock.patch.object(
                support,
                "default_results_stage_root",
                return_value=remote_root / "corpus_01" / stage_id / "lunarc",
            ):
                self.assertEqual(
                    support.preferred_report_run_root(corpus_root, stage_id),
                    legacy_root / "corpus_01" / stage_id / "lunarc",
                )

    def test_deplete_reference_contaminants_defaults_match_governed_suite(self) -> None:
        defaults = support.deplete_reference_contaminants_benchmark_defaults()
        self.assertEqual(defaults["threads"], 8)
        self.assertEqual(defaults["reference_catalog_id"], "contaminant_reference")
        self.assertEqual(defaults["reference_index_backend"], "bowtie2_build")
        self.assertEqual(defaults["decoy_mode"], "phix_and_spikeins")

    def test_screen_taxonomy_defaults_match_governed_suite(self) -> None:
        defaults = support.screen_taxonomy_benchmark_defaults()
        self.assertEqual(defaults["threads"], 8)
        self.assertEqual(defaults["database_catalog_id"], "taxonomy_reference")
        self.assertEqual(defaults["database_artifact_id"], "taxonomy_db")
        self.assertEqual(defaults["database_namespace"], "read_screening")
        self.assertEqual(defaults["database_scope"], "read_screening")
        self.assertIsNone(defaults["minimum_confidence"])
        self.assertTrue(defaults["emit_unclassified"])

    def test_screen_taxonomy_bootstrap_builds_lineage_payload(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            database_root = Path(tmpdir) / "taxonomy_db"
            (database_root / "source").mkdir(parents=True)
            (database_root / "kraken2").mkdir()
            (database_root / "krakenuniq").mkdir()
            (database_root / "centrifuge").mkdir()
            (database_root / "kaiju").mkdir()
            (database_root / "taxonomy").mkdir()
            (database_root / "kraken2" / "hash.k2d").write_text("a", encoding="utf-8")
            (database_root / "krakenuniq" / "database.kdb").write_text("b", encoding="utf-8")
            (database_root / "centrifuge" / "reference.1.cf").write_text("c", encoding="utf-8")
            (database_root / "kaiju" / "nodes.dmp").write_text("d", encoding="utf-8")
            (database_root / "taxonomy" / "names.dmp").write_text("e", encoding="utf-8")
            source_manifest = database_root / "source" / "panel_manifest.json"
            source_manifest.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "accession": "NC_000913.3",
                                "taxid": 562,
                                "display_name": "Escherichia coli K-12 MG1655",
                            }
                        ]
                    }
                ),
                encoding="utf-8",
            )

            payload = taxonomy_db_bootstrap.build_lineage_payload(
                database_root=database_root,
                source_manifest=source_manifest,
                bootstrap_report=None,
                database_catalog_id="taxonomy_reference",
                database_artifact_id="taxonomy_db",
                database_namespace="read_screening",
                database_scope="read_screening",
            )

        self.assertEqual(
            payload["schema_version"],
            "bijux.fastq.screen_taxonomy.database_lineage.v1",
        )
        self.assertEqual(payload["source_record_count"], 1)
        self.assertEqual(
            [row["backend"] for row in payload["backend_roots"]],
            ["kraken2", "krakenuniq", "centrifuge", "kaiju", "taxonomy"],
        )
        self.assertIsNotNone(payload["database_digest"])

    def test_screen_taxonomy_bootstrap_resolves_database_root_from_results_root(self) -> None:
        args = SimpleNamespace(
            database_root="",
            results_root="/tmp/local-results",
            cache_root="",
            database_namespace="read_screening",
            database_scope="read_screening",
            database_artifact_id="taxonomy_db",
        )

        self.assertEqual(
            taxonomy_db_bootstrap.resolve_database_root(args),
            Path(
                "/tmp/local-results/extra-data/benchmark/fastq.screen_taxonomy/"
                "read_screening/read_screening/taxonomy_db"
            ).resolve(),
        )

    def test_screen_taxonomy_bootstrap_defaults_to_workspace_results_root(self) -> None:
        args = SimpleNamespace(
            database_root="",
            results_root="",
            cache_root="",
            database_namespace="read_screening",
            database_scope="read_screening",
            database_artifact_id="taxonomy_db",
        )

        with mock.patch.object(
            taxonomy_db_bootstrap,
            "benchmark_local_results_root",
            return_value=Path("/srv/benchmark-results"),
        ):
            self.assertEqual(
                taxonomy_db_bootstrap.resolve_database_root(args),
                Path(
                    "/srv/benchmark-results/extra-data/benchmark/fastq.screen_taxonomy/"
                    "read_screening/read_screening/taxonomy_db"
                ).resolve(),
            )

    def test_screen_taxonomy_bootstrap_requires_all_backend_dirs(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            database_root = Path(tmpdir) / "taxonomy_db"
            (database_root / "source").mkdir(parents=True)
            source_manifest = database_root / "source" / "panel_manifest.json"
            source_manifest.write_text(
                json.dumps({"records": [{"accession": "NC_000913.3"}]}),
                encoding="utf-8",
            )
            with self.assertRaises(SystemExit):
                taxonomy_db_bootstrap.build_lineage_payload(
                    database_root=database_root,
                    source_manifest=source_manifest,
                    bootstrap_report=None,
                    database_catalog_id="taxonomy_reference",
                    database_artifact_id="taxonomy_db",
                    database_namespace="read_screening",
                    database_scope="read_screening",
                )

    def test_default_screen_taxonomy_database_root_prefers_cache_extra_data(
        self,
    ) -> None:
        out_root = Path(
            "/home/bijan/lu2024-12-24/.cache/bijux-dna-results/corpus_01/fastq.screen_taxonomy/lunarc"
        )
        self.assertEqual(
            support.default_screen_taxonomy_database_root(
                out_root,
                database_namespace="read_screening",
                database_scope="read_screening",
                database_artifact_id="taxonomy_db",
            ).resolve(),
            Path(
                "/home/bijan/lu2024-12-24/.cache/extra-data/benchmark/fastq.screen_taxonomy/read_screening/read_screening/taxonomy_db"
            ).resolve(),
        )

    def test_default_screen_taxonomy_database_root_uses_local_archive_extra_data(self) -> None:
        out_root = Path("/tmp/local-results/corpus_01/fastq.screen_taxonomy/lunarc")
        self.assertEqual(
            support.default_screen_taxonomy_database_root(
                out_root,
                database_namespace="read_screening",
                database_scope="read_screening",
                database_artifact_id="taxonomy_db",
            ).resolve(),
            Path(
                "/tmp/local-results/extra-data/benchmark/fastq.screen_taxonomy/read_screening/read_screening/taxonomy_db"
            ).resolve(),
        )

    def test_resolve_bowtie2_index_prefix_maps_directory_to_unique_prefix(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            index_root = Path(tmpdir) / "index"
            index_root.mkdir()
            (index_root / "reference.1.bt2").write_text("a", encoding="utf-8")
            (index_root / "reference.2.bt2").write_text("b", encoding="utf-8")
            (index_root / "reference.rev.1.bt2").write_text("c", encoding="utf-8")
            self.assertEqual(
                support.resolve_bowtie2_index_prefix(index_root).resolve(),
                (index_root / "reference").resolve(),
            )

    def test_resolve_bowtie2_index_prefix_rejects_empty_directory(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            index_root = Path(tmpdir) / "index"
            index_root.mkdir()
            with self.assertRaises(FileNotFoundError):
                support.resolve_bowtie2_index_prefix(index_root)

    def test_resolve_artifact_lineage_json_prefers_directory_lineage(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            bundle_root = Path(tmpdir) / "taxonomy_db"
            bundle_root.mkdir()
            lineage_path = bundle_root / "lineage.json"
            lineage_path.write_text("{}", encoding="utf-8")

            self.assertEqual(
                support.resolve_artifact_lineage_json(bundle_root),
                lineage_path.resolve(),
            )

    def test_resolve_artifact_lineage_json_supports_prefix_parent(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            index_root = Path(tmpdir) / "index"
            index_root.mkdir()
            prefix = index_root / "reference"
            (index_root / "reference.1.bt2").write_text("a", encoding="utf-8")
            lineage_path = index_root / "lineage.json"
            lineage_path.write_text("{}", encoding="utf-8")

            self.assertEqual(
                support.resolve_artifact_lineage_json(prefix),
                lineage_path.resolve(),
            )

    def test_correct_errors_defaults_match_governed_suite(self) -> None:
        defaults = support.correct_errors_benchmark_defaults()
        self.assertEqual(defaults["threads"], 8)
        self.assertEqual(defaults["quality_encoding"], "phred33")
        self.assertIsNone(defaults["kmer_size"])
        self.assertEqual(defaults["musket_kmer_budget"], 536_870_912)
        self.assertEqual(defaults["genome_size"], 3_200_000_000)
        self.assertIsNone(defaults["max_memory_gb"])
        self.assertIsNone(defaults["trusted_kmer_artifact"])
        self.assertFalse(defaults["conservative_mode"])
        self.assertEqual(
            support.CORRECT_ERRORS_BENCHMARK_CONTRACT.tools,
            ["bayeshammer", "lighter", "musket", "rcorrector"],
        )
        self.assertEqual(support.CORRECT_ERRORS_BENCHMARK_CONTRACT.sample_scope, "paired")

    def test_extract_umis_defaults_match_governed_suite(self) -> None:
        defaults = support.extract_umis_benchmark_defaults()
        self.assertEqual(defaults["threads"], 4)
        self.assertEqual(defaults["umi_pattern"], "NNNNNNNN")
        self.assertTrue(defaults["allow_missing_umi_headers"])

    def test_validate_corpus_contract_accepts_balanced_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            corpus_root = Path(tmpdir)
            normalized = corpus_root / "normalized"
            normalized.mkdir(parents=True)

            spec = {
                "target_ancient_se": 1,
                "target_ancient_pe": 1,
                "target_modern_se": 1,
                "target_modern_pe": 1,
                "samples": [
                    {
                        "accession": "ACC_ANCIENT_SE",
                        "era": "ancient",
                        "layout": "se",
                        "study_accession": "PRJ1",
                        "size_band": "under_100mb",
                    },
                    {
                        "accession": "ACC_ANCIENT_PE",
                        "era": "ancient",
                        "layout": "pe",
                        "study_accession": "PRJ2",
                        "size_band": "under_100mb",
                    },
                    {
                        "accession": "ACC_MODERN_SE",
                        "era": "modern",
                        "layout": "se",
                        "study_accession": "PRJ3",
                        "size_band": "under_500mb",
                    },
                    {
                        "accession": "ACC_MODERN_PE",
                        "era": "modern",
                        "layout": "pe",
                        "study_accession": "PRJ4",
                        "size_band": "under_500mb",
                    },
                ],
            }

            files = {
                "raw/ACC_ANCIENT_SE/reads.fastq.gz": "digest-ancient-se",
                "normalized/sample_0001_R1.fastq.gz": "digest-ancient-se",
                "raw/ACC_ANCIENT_PE/R1.fastq.gz": "digest-ancient-pe-r1",
                "raw/ACC_ANCIENT_PE/R2.fastq.gz": "digest-ancient-pe-r2",
                "normalized/sample_0002_R1.fastq.gz": "digest-ancient-pe-r1",
                "normalized/sample_0002_R2.fastq.gz": "digest-ancient-pe-r2",
                "raw/ACC_MODERN_SE/reads.fastq.gz": "digest-modern-se",
                "normalized/sample_0003_R1.fastq.gz": "digest-modern-se",
                "raw/ACC_MODERN_PE/R1.fastq.gz": "digest-modern-pe-r1",
                "raw/ACC_MODERN_PE/R2.fastq.gz": "digest-modern-pe-r2",
                "normalized/sample_0004_R1.fastq.gz": "digest-modern-pe-r1",
                "normalized/sample_0004_R2.fastq.gz": "digest-modern-pe-r2",
            }
            (corpus_root / "MANIFEST.json").write_text(
                json.dumps({"files": files}, indent=2) + "\n",
                encoding="utf-8",
            )

            for relative_path in [
                "normalized/sample_0001_R1.fastq.gz",
                "normalized/sample_0002_R1.fastq.gz",
                "normalized/sample_0002_R2.fastq.gz",
                "normalized/sample_0003_R1.fastq.gz",
                "normalized/sample_0004_R1.fastq.gz",
                "normalized/sample_0004_R2.fastq.gz",
            ]:
                (corpus_root / relative_path).write_bytes(b"test\n")

            samples = support.discover_normalized_samples(corpus_root, expected_total=4)
            metadata = support.validate_corpus_contract(corpus_root, spec, samples)

            self.assertEqual(metadata["sample_0001"]["accession"], "ACC_ANCIENT_SE")
            self.assertEqual(metadata["sample_0004"]["layout"], "pe")

    def test_require_exact_tool_roster_rejects_missing_tool(self) -> None:
        with self.assertRaises(SystemExit):
            support.require_exact_tool_roster(
                "fastq.trim_polyg_tails",
                ["fastp"],
                ["fastp", "bbduk"],
            )

    def test_select_paired_samples_rejects_unbalanced_subset(self) -> None:
        spec = {
            "target_ancient_pe": 1,
            "target_modern_pe": 1,
        }
        samples = [
            {"sample_id": "sample_0001"},
            {"sample_id": "sample_0002"},
        ]
        metadata_by_sample = {
            "sample_0001": {"era": "ancient", "layout": "pe"},
            "sample_0002": {"era": "ancient", "layout": "pe"},
        }

        with self.assertRaises(SystemExit):
            support.select_paired_samples(spec, samples, metadata_by_sample)

    def test_resolve_corpus_metadata_falls_back_to_published_docs(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            docs_root = repo_root / "docs" / "benchmark" / "fastq.validate_reads" / "corpus-01"
            docs_root.mkdir(parents=True)
            (docs_root / "sample_results.csv").write_text(
                "\n".join(
                    [
                        "sample_id,accession,era,layout,study_accession,size_band,tool",
                        "sample_0001,ACC_ANCIENT_SE,ancient,se,PRJ1,under_100mb,fastqvalidator",
                        "sample_0002,ACC_ANCIENT_PE,ancient,pe,PRJ2,under_100mb,fastqvalidator",
                        "sample_0003,ACC_MODERN_SE,modern,se,PRJ3,under_500mb,fastqvalidator",
                        "sample_0004,ACC_MODERN_PE,modern,pe,PRJ4,under_500mb,fastqvalidator",
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            corpus_root = repo_root / "missing-corpus"
            spec = {
                "target_ancient_se": 1,
                "target_ancient_pe": 1,
                "target_modern_se": 1,
                "target_modern_pe": 1,
            }

            metadata = support.resolve_corpus_metadata(
                repo_root,
                corpus_root,
                spec,
                expected_sample_ids=[
                    "sample_0001",
                    "sample_0002",
                    "sample_0003",
                    "sample_0004",
                ],
            )

            self.assertEqual(metadata["sample_0001"]["accession"], "ACC_ANCIENT_SE")
            self.assertEqual(metadata["sample_0004"]["layout"], "pe")

    def test_resolve_corpus_metadata_accepts_paired_subset_from_full_docs(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            docs_root = repo_root / "docs" / "benchmark" / "fastq.validate_reads" / "corpus-01"
            docs_root.mkdir(parents=True)
            (docs_root / "sample_results.csv").write_text(
                "\n".join(
                    [
                        "sample_id,accession,era,layout,study_accession,size_band,tool",
                        "sample_0001,ACC_ANCIENT_SE,ancient,se,PRJ1,under_100mb,fastqvalidator",
                        "sample_0002,ACC_ANCIENT_PE,ancient,pe,PRJ2,under_100mb,fastqvalidator",
                        "sample_0003,ACC_MODERN_SE,modern,se,PRJ3,under_500mb,fastqvalidator",
                        "sample_0004,ACC_MODERN_PE,modern,pe,PRJ4,under_500mb,fastqvalidator",
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            corpus_root = repo_root / "missing-corpus"
            spec = {
                "target_ancient_se": 1,
                "target_ancient_pe": 1,
                "target_modern_se": 1,
                "target_modern_pe": 1,
            }

            metadata = support.resolve_corpus_metadata(
                repo_root,
                corpus_root,
                spec,
                expected_sample_ids=["sample_0002", "sample_0004"],
            )

            self.assertEqual(sorted(metadata), ["sample_0001", "sample_0002", "sample_0003", "sample_0004"])
            self.assertEqual(metadata["sample_0002"]["layout"], "pe")
            self.assertEqual(metadata["sample_0004"]["era"], "modern")

    def test_filter_low_complexity_runner_parse_args_supports_sample_jobs(self) -> None:
        argv = [
            "run_fastq_filter_low_complexity_corpus_01.py",
            "--sample-jobs",
            "3",
            "--entropy-threshold",
            "0.6",
        ]
        with mock.patch.object(sys, "argv", argv):
            args = filter_low_complexity_runner.parse_args()
        self.assertEqual(args.sample_jobs, 3)
        self.assertEqual(args.entropy_threshold, 0.6)

    def test_filter_reads_runner_parse_args_supports_filter_overrides(self) -> None:
        argv = [
            "run_fastq_filter_reads_corpus_01.py",
            "--sample-jobs",
            "2",
            "--threads",
            "6",
            "--max-n",
            "0",
            "--max-n-count",
            "5",
            "--low-complexity-threshold",
            "19.5",
            "--entropy-threshold",
            "17.5",
            "--polyx-policy",
            "trim",
        ]
        with mock.patch.object(sys, "argv", argv):
            args = filter_reads_runner.parse_args()
        self.assertEqual(args.sample_jobs, 2)
        self.assertEqual(args.threads, 6)
        self.assertEqual(args.max_n, 0)
        self.assertEqual(args.max_n_count, 5)
        self.assertEqual(args.low_complexity_threshold, 19.5)
        self.assertEqual(args.entropy_threshold, 17.5)
        self.assertEqual(args.polyx_policy, "trim")

    def test_filter_reads_runner_resume_requires_successful_sample_report(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            report_path = Path(tmpdir) / "report.json"
            report_path.write_text(
                json.dumps(
                    {
                        "failures": [],
                        "gate": {"passes": True},
                        "records": [{"context": {"tool": "fastp"}}],
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            self.assertTrue(
                filter_reads_runner.sample_report_is_resume_ready(report_path)
            )

            report_path.write_text(
                json.dumps(
                    {
                        "failures": [
                            {
                                "kind": "tool_exit",
                                "reason": "tool `fastp` failed with status 143",
                            }
                        ],
                        "gate": {"passes": False},
                        "records": [],
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            self.assertFalse(
                filter_reads_runner.sample_report_is_resume_ready(report_path)
            )

    def test_filter_reads_runner_reruns_stale_resume_report(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir) / "repo"
            repo_root.mkdir()
            corpus_root = Path(tmpdir) / "corpus_01"
            normalized_root = corpus_root / "normalized"
            normalized_root.mkdir(parents=True)
            r1_path = normalized_root / "sample_0001_R1.fastq.gz"
            r1_path.write_text("reads", encoding="utf-8")
            out_root = Path(tmpdir) / "results"
            stale_sample_root = out_root / "bench" / "filter" / "sample_0001"
            stale_sample_root.mkdir(parents=True)
            stale_marker = stale_sample_root / "stale.marker"
            stale_marker.write_text("old", encoding="utf-8")
            stale_report = stale_sample_root / "report.json"
            stale_report.write_text(
                json.dumps(
                    {
                        "failures": [
                            {
                                "kind": "tool_exit",
                                "reason": "tool `fastp` failed with status 143",
                            }
                        ],
                        "gate": {"passes": False},
                        "records": [],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            def fake_run(command: list[str], cwd: Path, check: bool = False):
                self.assertEqual(Path(cwd).resolve(), repo_root.resolve())
                self.assertFalse(stale_marker.exists())
                fresh_report = out_root / "bench" / "filter" / "sample_0001" / "report.json"
                fresh_report.parent.mkdir(parents=True, exist_ok=True)
                fresh_report.write_text(
                    json.dumps(
                        {
                            "failures": [],
                            "gate": {"passes": True},
                            "records": [{"context": {"tool": "fastp"}}],
                            "semantic_metrics": [],
                        }
                    )
                    + "\n",
                    encoding="utf-8",
                )
                return mock.Mock(returncode=0)

            argv = [
                "run_fastq_filter_reads_corpus_01.py",
                "--repo-root",
                str(repo_root),
                "--corpus-root",
                str(corpus_root),
                "--out-root",
                str(out_root),
            ]
            with mock.patch.object(sys, "argv", argv):
                with mock.patch.object(
                    filter_reads_runner,
                    "load_corpus_spec",
                    return_value={"preferred_root": str(corpus_root)},
                ):
                    with mock.patch.object(
                        filter_reads_runner,
                        "discover_normalized_samples",
                        return_value=[
                            {
                                "sample_id": "sample_0001",
                                "r1": r1_path,
                                "r2": None,
                                "layout": "se",
                            }
                        ],
                    ):
                        with mock.patch.object(
                            filter_reads_runner,
                            "validate_benchmark_layout",
                        ):
                            with mock.patch.object(
                                filter_reads_runner,
                                "validate_corpus_contract",
                            ):
                                with mock.patch.object(
                                    filter_reads_runner,
                                    "require_canonical_tool_roster",
                                    return_value=["fastp"],
                                ):
                                    with mock.patch.object(
                                        filter_reads_runner.subprocess,
                                        "run",
                                        side_effect=fake_run,
                                    ) as run_mock:
                                        exit_code = filter_reads_runner.main()

            self.assertEqual(exit_code, 0)
            self.assertEqual(run_mock.call_count, 1)
            self.assertFalse(stale_marker.exists())

    def test_filter_reads_runner_resets_orphaned_sample_payload_before_resume(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir) / "repo"
            repo_root.mkdir()
            corpus_root = Path(tmpdir) / "corpus_01"
            normalized_root = corpus_root / "normalized"
            normalized_root.mkdir(parents=True)
            r1_path = normalized_root / "sample_0001_R1.fastq.gz"
            r1_path.write_text("reads", encoding="utf-8")
            out_root = Path(tmpdir) / "results"
            orphaned_sample_root = out_root / "bench" / "filter" / "sample_0001"
            orphaned_sample_root.mkdir(parents=True)
            stale_marker = orphaned_sample_root / "stale.marker"
            stale_marker.write_text("old", encoding="utf-8")

            def fake_run(command: list[str], cwd: Path, check: bool = False):
                self.assertEqual(Path(cwd).resolve(), repo_root.resolve())
                self.assertFalse(stale_marker.exists())
                fresh_report = out_root / "bench" / "filter" / "sample_0001" / "report.json"
                fresh_report.parent.mkdir(parents=True, exist_ok=True)
                fresh_report.write_text(
                    json.dumps(
                        {
                            "failures": [],
                            "gate": {"passes": True},
                            "records": [{"context": {"tool": "fastp"}}],
                            "semantic_metrics": [],
                        }
                    )
                    + "\n",
                    encoding="utf-8",
                )
                return mock.Mock(returncode=0)

            argv = [
                "run_fastq_filter_reads_corpus_01.py",
                "--repo-root",
                str(repo_root),
                "--corpus-root",
                str(corpus_root),
                "--out-root",
                str(out_root),
            ]
            with mock.patch.object(sys, "argv", argv):
                with mock.patch.object(
                    filter_reads_runner,
                    "load_corpus_spec",
                    return_value={"preferred_root": str(corpus_root)},
                ):
                    with mock.patch.object(
                        filter_reads_runner,
                        "discover_normalized_samples",
                        return_value=[
                            {
                                "sample_id": "sample_0001",
                                "r1": r1_path,
                                "r2": None,
                                "layout": "se",
                            }
                        ],
                    ):
                        with mock.patch.object(
                            filter_reads_runner,
                            "validate_benchmark_layout",
                        ):
                            with mock.patch.object(
                                filter_reads_runner,
                                "validate_corpus_contract",
                            ):
                                with mock.patch.object(
                                    filter_reads_runner,
                                    "require_canonical_tool_roster",
                                    return_value=["fastp"],
                                ):
                                    with mock.patch.object(
                                        filter_reads_runner.subprocess,
                                        "run",
                                        side_effect=fake_run,
                                    ) as run_mock:
                                        exit_code = filter_reads_runner.main()

            self.assertEqual(exit_code, 0)
            self.assertEqual(run_mock.call_count, 1)
            self.assertFalse(stale_marker.exists())

    def test_remove_duplicates_runner_parse_args_supports_sample_jobs(self) -> None:
        argv = [
            "run_fastq_remove_duplicates_corpus_01.py",
            "--sample-jobs",
            "2",
            "--dedup-mode",
            "exact",
            "--keep-order",
            "false",
        ]
        with mock.patch.object(sys, "argv", argv):
            args = remove_duplicates_runner.parse_args()
        self.assertEqual(args.sample_jobs, 2)
        self.assertEqual(args.dedup_mode, "exact")
        self.assertFalse(args.keep_order)

    def test_normalize_primers_runner_parse_args_supports_policy_overrides(self) -> None:
        argv = [
            "run_fastq_normalize_primers_corpus_01.py",
            "--sample-jobs",
            "2",
            "--primer-set-id",
            "16S_universal_v1",
            "--max-mismatch-rate",
            "0.05",
            "--strict-5p-anchor",
            "false",
        ]
        with mock.patch.object(sys, "argv", argv):
            args = normalize_primers_runner.parse_args()
        self.assertEqual(args.sample_jobs, 2)
        self.assertEqual(args.primer_set_id, "16S_universal_v1")
        self.assertAlmostEqual(args.max_mismatch_rate, 0.05)
        self.assertFalse(args.strict_5p_anchor)

    def test_deplete_rrna_runner_parse_args_supports_bundle_overrides(self) -> None:
        argv = [
            "run_fastq_deplete_rrna_corpus_01.py",
            "--sample-jobs",
            "3",
            "--threads",
            "6",
            "--rrna-db",
            "/refs/sortmerna_v4_3_default_db.fasta",
            "--rrna-bundle-id",
            "sortmerna_v4_3_default_db",
            "--min-identity",
            "0.95",
        ]
        with mock.patch.object(sys, "argv", argv):
            args = deplete_rrna_runner.parse_args()
        self.assertEqual(args.sample_jobs, 3)
        self.assertEqual(args.threads, 6)
        self.assertEqual(args.rrna_db, "/refs/sortmerna_v4_3_default_db.fasta")
        self.assertEqual(args.rrna_bundle_id, "sortmerna_v4_3_default_db")
        self.assertAlmostEqual(args.min_identity, 0.95)

    def test_deplete_host_runner_parse_args_supports_reference_overrides(self) -> None:
        argv = [
            "run_fastq_deplete_host_corpus_01.py",
            "--sample-jobs",
            "2",
            "--threads",
            "6",
            "--reference-index",
            "/refs/host_index",
            "--host-identity-threshold",
            "0.99",
            "--retain-unmapped-only",
            "false",
        ]
        with mock.patch.object(sys, "argv", argv):
            args = deplete_host_runner.parse_args()
        self.assertEqual(args.sample_jobs, 2)
        self.assertEqual(args.threads, 6)
        self.assertEqual(args.reference_index, "/refs/host_index")
        self.assertAlmostEqual(args.host_identity_threshold, 0.99)
        self.assertFalse(args.retain_unmapped_only)

    def test_deplete_host_runner_resolves_directory_reference_index_to_prefix(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            out_root = Path(tmpdir) / "results"
            out_root.mkdir()
            index_root = Path(tmpdir) / "index"
            index_root.mkdir()
            (index_root / "reference.1.bt2").write_text("a", encoding="utf-8")
            (index_root / "reference.2.bt2").write_text("b", encoding="utf-8")
            args = SimpleNamespace(
                reference_index=str(index_root),
                reference_catalog_id="host_reference",
                reference_index_backend="bowtie2_build",
            )
            self.assertEqual(
                deplete_host_runner.resolve_reference_index(args, out_root).resolve(),
                (index_root / "reference").resolve(),
            )

    def test_deplete_reference_contaminants_runner_parse_args_supports_policy_overrides(
        self,
    ) -> None:
        argv = [
            "run_fastq_deplete_reference_contaminants_corpus_01.py",
            "--sample-jobs",
            "2",
            "--threads",
            "6",
            "--reference-index",
            "/refs/contaminants",
            "--decoy-mode",
            "phix_and_spikeins",
        ]
        with mock.patch.object(sys, "argv", argv):
            args = deplete_reference_contaminants_runner.parse_args()
        self.assertEqual(args.sample_jobs, 2)
        self.assertEqual(args.threads, 6)
        self.assertEqual(args.reference_index, "/refs/contaminants")
        self.assertEqual(args.decoy_mode, "phix_and_spikeins")

    def test_deplete_reference_contaminants_runner_resolves_directory_reference_index_to_prefix(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            index_root = Path(tmpdir) / "index"
            index_root.mkdir()
            (index_root / "reference.1.bt2").write_text("a", encoding="utf-8")
            (index_root / "reference.2.bt2").write_text("b", encoding="utf-8")
            args = SimpleNamespace(reference_index=str(index_root))
            self.assertEqual(
                deplete_reference_contaminants_runner.resolve_reference_index(
                    args
                ).resolve(),
                (index_root / "reference").resolve(),
            )

    def test_screen_taxonomy_runner_parse_args_supports_database_overrides(
        self,
    ) -> None:
        argv = [
            "run_fastq_screen_taxonomy_corpus_01.py",
            "--sample-jobs",
            "2",
            "--threads",
            "6",
            "--database-root",
            "/refs/taxonomy",
            "--database-catalog-id",
            "taxonomy_reference_v2",
            "--database-artifact-id",
            "taxonomy_db_2026_03",
        ]
        with mock.patch.object(sys, "argv", argv):
            args = screen_taxonomy_runner.parse_args()
        self.assertEqual(args.sample_jobs, 2)
        self.assertEqual(args.threads, 6)
        self.assertEqual(args.database_root, "/refs/taxonomy")
        self.assertEqual(args.database_catalog_id, "taxonomy_reference_v2")
        self.assertEqual(args.database_artifact_id, "taxonomy_db_2026_03")

    def test_screen_taxonomy_runner_build_command_includes_database_root(self) -> None:
        command = screen_taxonomy_runner.build_command(
            out_root=Path("/tmp/out"),
            platform="lunarc-apptainer",
            tools="kraken2,kaiju",
            database_root=Path("/refs/taxonomy"),
            threads=8,
            jobs=1,
            sample={
                "sample_id": "sample_0001",
                "r1": Path("/corpus/sample_R1.fastq.gz"),
                "r2": Path("/corpus/sample_R2.fastq.gz"),
                "layout": "paired",
            },
        )

        self.assertIn("--database-root", command)
        self.assertIn("/refs/taxonomy", command)

    def test_correct_errors_runner_parse_args_supports_policy_overrides(self) -> None:
        argv = [
            "run_fastq_correct_errors_corpus_01.py",
            "--sample-jobs",
            "2",
            "--threads",
            "6",
            "--quality-encoding",
            "phred33",
            "--kmer-size",
            "31",
            "--musket-kmer-budget",
            "536870912",
            "--genome-size",
            "2800000",
            "--max-memory-gb",
            "16",
            "--trusted-kmer-artifact",
            "/refs/trusted.kmers",
            "--conservative-mode",
            "false",
        ]
        with mock.patch.object(sys, "argv", argv):
            args = correct_errors_runner.parse_args()
        self.assertEqual(args.sample_jobs, 2)
        self.assertEqual(args.threads, 6)
        self.assertEqual(args.quality_encoding, "phred33")
        self.assertEqual(args.kmer_size, 31)
        self.assertEqual(args.musket_kmer_budget, 536870912)
        self.assertEqual(args.genome_size, 2800000)
        self.assertEqual(args.max_memory_gb, 16)
        self.assertEqual(args.trusted_kmer_artifact, "/refs/trusted.kmers")
        self.assertFalse(args.conservative_mode)

    def test_extract_umis_runner_parse_args_supports_pattern_overrides(self) -> None:
        argv = [
            "run_fastq_extract_umis_corpus_01.py",
            "--sample-jobs",
            "2",
            "--threads",
            "6",
            "--umi-pattern",
            "NNNNNNNNNN",
            "--no-allow-missing-umi-headers",
        ]
        with mock.patch.object(sys, "argv", argv):
            args = extract_umis_runner.parse_args()
        self.assertEqual(args.sample_jobs, 2)
        self.assertEqual(args.threads, 6)
        self.assertEqual(args.umi_pattern, "NNNNNNNNNN")
        self.assertFalse(args.allow_missing_umi_headers)

    def test_extract_umis_runner_sets_missing_header_bypass_env(self) -> None:
        sample = {
            "sample_id": "sample_0002",
            "r1": Path("/tmp/sample_0002_R1.fastq.gz"),
            "r2": Path("/tmp/sample_0002_R2.fastq.gz"),
            "layout": "pe",
        }
        with mock.patch.object(
            extract_umis_runner.subprocess,
            "run",
            return_value=mock.Mock(returncode=0),
        ) as run_mock:
            run = extract_umis_runner.run_sample_command(
                repo_root=Path("/tmp/repo"),
                sample=sample,
                command=["cargo", "run"],
                sample_report=Path("/tmp/report.json"),
                allow_missing_umi_headers=True,
            )
        self.assertEqual(run.exit_code, 0)
        self.assertEqual(run_mock.call_args.kwargs["env"]["BIJUX_ALLOW_NO_UMI"], "1")

    def test_correct_errors_runner_dry_run_selects_paired_subset_only(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir) / "repo"
            corpus_root = Path(tmpdir) / "corpus"
            out_root = Path(tmpdir) / "results"
            repo_root.mkdir()
            corpus_root.mkdir()

            args = mock.Mock(
                repo_root=str(repo_root),
                corpus_root=str(corpus_root),
                out_root=str(out_root),
                platform="lunarc-apptainer",
                tools="",
                threads=8,
                jobs=1,
                sample_jobs=1,
                sample_limit=0,
                quality_encoding="phred33",
                kmer_size=None,
                musket_kmer_budget=536_870_912,
                genome_size=3_200_000_000,
                max_memory_gb=None,
                trusted_kmer_artifact="",
                conservative_mode=False,
                resume=True,
                dry_run=True,
            )
            all_samples = [
                {
                    "sample_id": "sample_0001",
                    "r1": corpus_root / "sample_0001_R1.fastq.gz",
                    "r2": None,
                    "layout": "se",
                },
                {
                    "sample_id": "sample_0002",
                    "r1": corpus_root / "sample_0002_R1.fastq.gz",
                    "r2": corpus_root / "sample_0002_R2.fastq.gz",
                    "layout": "pe",
                },
                {
                    "sample_id": "sample_0003",
                    "r1": corpus_root / "sample_0003_R1.fastq.gz",
                    "r2": corpus_root / "sample_0003_R2.fastq.gz",
                    "layout": "pe",
                },
            ]
            metadata_by_sample = {
                "sample_0001": {"layout": "se", "era": "ancient"},
                "sample_0002": {"layout": "pe", "era": "ancient"},
                "sample_0003": {"layout": "pe", "era": "modern"},
            }

            with mock.patch.object(correct_errors_runner, "parse_args", return_value=args):
                with mock.patch.object(
                    correct_errors_runner,
                    "load_corpus_spec",
                    return_value={
                        "preferred_root": str(corpus_root),
                        "target_ancient_pe": 1,
                        "target_modern_pe": 1,
                    },
                ):
                    with mock.patch.object(
                        correct_errors_runner,
                        "validate_benchmark_layout",
                    ):
                        with mock.patch.object(
                            correct_errors_runner,
                            "discover_normalized_samples",
                            return_value=all_samples,
                        ):
                            with mock.patch.object(
                                correct_errors_runner,
                                "validate_corpus_contract",
                                return_value=metadata_by_sample,
                            ):
                                with mock.patch.object(
                                    correct_errors_runner,
                                    "require_canonical_tool_roster",
                                    return_value=["lighter", "musket", "rcorrector"],
                                ):
                                    exit_code = correct_errors_runner.main()

            self.assertEqual(exit_code, 0)
            manifest = json.loads((out_root / "run_manifest.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["samples_total"], 2)
            self.assertEqual(
                [run["sample_id"] for run in manifest["runs"]],
                ["sample_0002", "sample_0003"],
            )

    def test_deplete_rrna_runner_shared_index_layout_is_stable(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            out_root = Path(tmpdir) / "results"
            shared_idx_dir = deplete_rrna_runner.sortmerna_shared_index_dir(
                out_root,
                "sortmerna_v4_3_default_db",
            )
            prepared = deplete_rrna_runner.prepare_sortmerna_sample_workdir(
                out_root,
                "sample_0001",
                "sortmerna_v4_3_default_db",
            )

            sample_idx_dir = (
                out_root
                / "bench"
                / "deplete_rrna"
                / "sample_0001"
                / "tools"
                / "sortmerna"
                / "sortmerna_workdir"
                / "idx"
            )
            self.assertEqual(prepared, shared_idx_dir)
            self.assertTrue(sample_idx_dir.is_dir())
            self.assertFalse(any(sample_idx_dir.iterdir()))

    def test_deplete_rrna_runner_lunarc_container_input_path_rewrites_bind_root(
        self,
    ) -> None:
        with mock.patch.object(
            deplete_rrna_runner.Path,
            "home",
            return_value=Path("/home/tester"),
        ):
            self.assertEqual(
                deplete_rrna_runner.lunarc_container_input_path(
                    Path("/home/tester/bijux/reference/rrna/db.fasta")
                ),
                "/data/input/reference/rrna/db.fasta",
            )

    def test_deplete_rrna_runner_promotes_sample_index_into_shared_cache(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            out_root = Path(tmpdir) / "results"
            local_idx_dir = (
                out_root
                / "bench"
                / "deplete_rrna"
                / "sample_0001"
                / "tools"
                / "sortmerna"
                / "sortmerna_workdir"
                / "idx"
            )
            local_idx_dir.mkdir(parents=True)
            (local_idx_dir / "reference.stats").write_text("seed", encoding="utf-8")

            shared_idx_dir = deplete_rrna_runner.promote_sortmerna_sample_index_cache(
                out_root,
                "sample_0001",
                "sortmerna_v4_3_default_db",
            )

            self.assertTrue((shared_idx_dir / "reference.stats").is_file())

    def test_deplete_rrna_runner_clones_seeded_cache_into_sample_workdir(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            out_root = Path(tmpdir) / "results"
            shared_idx_dir = deplete_rrna_runner.sortmerna_shared_index_dir(
                out_root,
                "sortmerna_v4_3_default_db",
            )
            shared_idx_dir.mkdir(parents=True)
            (shared_idx_dir / "reference.stats").write_text("seed", encoding="utf-8")

            deplete_rrna_runner.prepare_sortmerna_sample_workdir(
                out_root,
                "sample_0002",
                "sortmerna_v4_3_default_db",
            )
            sample_idx_dir = (
                out_root
                / "bench"
                / "deplete_rrna"
                / "sample_0002"
                / "tools"
                / "sortmerna"
                / "sortmerna_workdir"
                / "idx"
            )

            self.assertTrue((sample_idx_dir / "reference.stats").is_file())
            self.assertEqual(
                os.stat(sample_idx_dir / "reference.stats").st_nlink,
                os.stat(shared_idx_dir / "reference.stats").st_nlink,
            )

    def test_deplete_rrna_runner_prunes_sample_workdir_after_capture(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            out_root = Path(tmpdir) / "results"
            sample_workdir = deplete_rrna_runner.sortmerna_sample_workdir(
                out_root,
                "sample_0003",
            )
            (sample_workdir / "idx").mkdir(parents=True)
            (sample_workdir / "idx" / "reference.stats").write_text(
                "seed",
                encoding="utf-8",
            )

            deplete_rrna_runner.prune_sortmerna_sample_payload(
                out_root,
                "sample_0003",
            )

            self.assertFalse(sample_workdir.exists())

    def test_deplete_rrna_runner_resume_requires_successful_sample_report(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            report_path = Path(tmpdir) / "report.json"
            report_path.write_text(
                json.dumps(
                    {
                        "failures": [],
                        "gate": {"passes": True},
                        "records": [{"context": {"tool": "sortmerna"}}],
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            self.assertTrue(
                deplete_rrna_runner.sample_report_is_resume_ready(report_path)
            )

            report_path.write_text(
                json.dumps(
                    {
                        "failures": [
                            {
                                "kind": "tool_exit",
                                "reason": "tool `sortmerna` failed with status 143",
                            }
                        ],
                        "gate": {"passes": False},
                        "records": [],
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            self.assertFalse(
                deplete_rrna_runner.sample_report_is_resume_ready(report_path)
            )

    def test_deplete_rrna_runner_reruns_stale_resume_report(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir) / "repo"
            repo_root.mkdir()
            corpus_root = Path(tmpdir) / "corpus_01"
            normalized_root = corpus_root / "normalized"
            normalized_root.mkdir(parents=True)
            r1_path = normalized_root / "sample_0001_R1.fastq.gz"
            r1_path.write_text("reads", encoding="utf-8")
            rrna_db = Path(tmpdir) / "sortmerna_v4_3_default_db.fasta"
            rrna_db.write_text(">rrna\nACGT\n", encoding="utf-8")
            out_root = Path(tmpdir) / "results"
            stale_sample_root = out_root / "bench" / "deplete_rrna" / "sample_0001"
            stale_sample_root.mkdir(parents=True)
            stale_marker = stale_sample_root / "stale.marker"
            stale_marker.write_text("old", encoding="utf-8")
            stale_report = stale_sample_root / "report.json"
            stale_report.write_text(
                json.dumps(
                    {
                        "failures": [
                            {
                                "kind": "tool_exit",
                                "reason": "tool `sortmerna` failed with status 143",
                            }
                        ],
                        "gate": {"passes": False},
                        "records": [],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            def fake_run(command: list[str], cwd: Path, check: bool = False):
                self.assertEqual(Path(cwd).resolve(), repo_root.resolve())
                self.assertFalse(stale_marker.exists())
                fresh_report = out_root / "bench" / "deplete_rrna" / "sample_0001" / "report.json"
                fresh_report.parent.mkdir(parents=True, exist_ok=True)
                fresh_report.write_text(
                    json.dumps(
                        {
                            "failures": [],
                            "gate": {"passes": True},
                            "records": [{"context": {"tool": "sortmerna"}}],
                            "semantic_metrics": [],
                        }
                    )
                    + "\n",
                    encoding="utf-8",
                )
                return mock.Mock(returncode=0)

            argv = [
                "run_fastq_deplete_rrna_corpus_01.py",
                "--repo-root",
                str(repo_root),
                "--corpus-root",
                str(corpus_root),
                "--out-root",
                str(out_root),
                "--rrna-db",
                str(rrna_db),
            ]
            with mock.patch.object(sys, "argv", argv):
                with mock.patch.object(
                    deplete_rrna_runner,
                    "load_corpus_spec",
                    return_value={"preferred_root": str(corpus_root)},
                ):
                    with mock.patch.object(
                        deplete_rrna_runner,
                        "discover_normalized_samples",
                        return_value=[
                            {
                                "sample_id": "sample_0001",
                                "r1": r1_path,
                                "r2": None,
                                "layout": "se",
                            }
                        ],
                    ):
                        with mock.patch.object(
                            deplete_rrna_runner,
                            "validate_benchmark_layout",
                        ):
                            with mock.patch.object(
                                deplete_rrna_runner,
                                "validate_corpus_contract",
                            ):
                                with mock.patch.object(
                                    deplete_rrna_runner,
                                    "require_canonical_tool_roster",
                                    return_value=["sortmerna"],
                                ):
                                    with mock.patch.object(
                                        deplete_rrna_runner,
                                        "warm_sortmerna_shared_index_cache",
                                    ):
                                        with mock.patch.object(
                                            deplete_rrna_runner,
                                            "sortmerna_shared_index_seeded",
                                            return_value=True,
                                        ):
                                            with mock.patch.object(
                                                deplete_rrna_runner.subprocess,
                                                "run",
                                                side_effect=fake_run,
                                            ) as run_mock:
                                                exit_code = deplete_rrna_runner.main()

            self.assertEqual(exit_code, 0)
            self.assertEqual(run_mock.call_count, 1)
            self.assertFalse(stale_marker.exists())
            manifest = json.loads(
                (out_root / "run_manifest.json").read_text(encoding="utf-8")
            )
            self.assertEqual(manifest["samples_failed"], 0)
            self.assertEqual(manifest["runs"][0]["status"], "completed")

    def test_deplete_rrna_runner_resets_orphaned_sample_payload_before_resume(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir) / "repo"
            repo_root.mkdir()
            corpus_root = Path(tmpdir) / "corpus_01"
            normalized_root = corpus_root / "normalized"
            normalized_root.mkdir(parents=True)
            r1_path = normalized_root / "sample_0001_R1.fastq.gz"
            r1_path.write_text("reads", encoding="utf-8")
            rrna_db = Path(tmpdir) / "sortmerna_v4_3_default_db.fasta"
            rrna_db.write_text(">rrna\nACGT\n", encoding="utf-8")
            out_root = Path(tmpdir) / "results"
            orphaned_sample_root = out_root / "bench" / "deplete_rrna" / "sample_0001"
            orphaned_sample_root.mkdir(parents=True)
            stale_marker = orphaned_sample_root / "stale.marker"
            stale_marker.write_text("old", encoding="utf-8")

            def fake_run(command: list[str], cwd: Path, check: bool = False):
                self.assertEqual(Path(cwd).resolve(), repo_root.resolve())
                self.assertFalse(stale_marker.exists())
                fresh_report = out_root / "bench" / "deplete_rrna" / "sample_0001" / "report.json"
                fresh_report.parent.mkdir(parents=True, exist_ok=True)
                fresh_report.write_text(
                    json.dumps(
                        {
                            "failures": [],
                            "gate": {"passes": True},
                            "records": [{"context": {"tool": "sortmerna"}}],
                            "semantic_metrics": [],
                        }
                    )
                    + "\n",
                    encoding="utf-8",
                )
                return mock.Mock(returncode=0)

            argv = [
                "run_fastq_deplete_rrna_corpus_01.py",
                "--repo-root",
                str(repo_root),
                "--corpus-root",
                str(corpus_root),
                "--out-root",
                str(out_root),
                "--rrna-db",
                str(rrna_db),
            ]
            with mock.patch.object(sys, "argv", argv):
                with mock.patch.object(
                    deplete_rrna_runner,
                    "load_corpus_spec",
                    return_value={"preferred_root": str(corpus_root)},
                ):
                    with mock.patch.object(
                        deplete_rrna_runner,
                        "discover_normalized_samples",
                        return_value=[
                            {
                                "sample_id": "sample_0001",
                                "r1": r1_path,
                                "r2": None,
                                "layout": "se",
                            }
                        ],
                    ):
                        with mock.patch.object(
                            deplete_rrna_runner,
                            "validate_benchmark_layout",
                        ):
                            with mock.patch.object(
                                deplete_rrna_runner,
                                "validate_corpus_contract",
                            ):
                                with mock.patch.object(
                                    deplete_rrna_runner,
                                    "require_canonical_tool_roster",
                                    return_value=["sortmerna"],
                                ):
                                    with mock.patch.object(
                                        deplete_rrna_runner,
                                        "warm_sortmerna_shared_index_cache",
                                    ):
                                        with mock.patch.object(
                                            deplete_rrna_runner,
                                            "sortmerna_shared_index_seeded",
                                            return_value=True,
                                        ):
                                            with mock.patch.object(
                                                deplete_rrna_runner.subprocess,
                                                "run",
                                                side_effect=fake_run,
                                            ) as run_mock:
                                                exit_code = deplete_rrna_runner.main()

            self.assertEqual(exit_code, 0)
            self.assertEqual(run_mock.call_count, 1)
            self.assertFalse(stale_marker.exists())

    def test_normalize_primers_report_contract_rejects_policy_drift(self) -> None:
        run_manifest = {
            "tools": ["cutadapt"],
            "primer_set_id": "16S_universal_v1",
            "orientation_policy": "normalize_to_forward_primer",
            "max_mismatch_rate": 0.10,
            "min_overlap_bp": 10,
            "strict_5p_anchor": True,
            "allow_iupac_codes": True,
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "cutadapt",
                "primer_set_id": "16S_universal_v1",
                "orientation_policy": "normalize_to_forward_primer",
                "max_mismatch_rate": 0.10,
                "min_overlap_bp": 10,
                "strict_5p_anchor": False,
                "allow_iupac_codes": True,
                "raw_backend_report_format": "cutadapt_json",
                "reads_in": 100,
                "reads_out": 100,
            }
        ]
        with self.assertRaises(SystemExit):
            normalize_primers_report.validate_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0001"],
            )

    def test_normalize_primers_summary_preserves_configured_corpus_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            run_root = (
                repo_root / "results" / "corpus_01" / "fastq.normalize_primers" / "lunarc"
            )
            docs_root = (
                repo_root
                / "docs"
                / "benchmark"
                / "fastq.normalize_primers"
                / "corpus-01"
            )
            sample_report = (
                run_root / "bench" / "normalize_primers" / "sample_0001" / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {
                                    "tool": "cutadapt",
                                    "parameters": {
                                        "primer_set_id": "16S_universal_v1",
                                        "orientation_policy": "normalize_to_forward_primer",
                                        "max_mismatch_rate": 0.10,
                                        "min_overlap_bp": 10,
                                        "strict_5p_anchor": True,
                                        "allow_iupac_codes": True,
                                        "raw_backend_report_format": "cutadapt_json",
                                    },
                                },
                                "execution": {"runtime_s": 1.5, "exit_code": 0},
                                "metrics": {
                                    "reads_in": 100,
                                    "reads_out": 100,
                                    "primer_trimmed_fraction": 0.05,
                                    "orientation_forward_fraction": 1.0,
                                },
                            }
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "platform": "lunarc-apptainer",
                        "corpus_root": "/home/bijan/bijux/corpus_01",
                        "stage_id": "fastq.normalize_primers",
                        "scenario_id": "primer_normalization_fairness",
                        "tools": ["cutadapt"],
                        "primer_set_id": "16S_universal_v1",
                        "orientation_policy": "normalize_to_forward_primer",
                        "max_mismatch_rate": 0.10,
                        "min_overlap_bp": 10,
                        "strict_5p_anchor": True,
                        "allow_iupac_codes": True,
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "report_json": str(sample_report),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            original_argv = sys.argv
            try:
                sys.argv = [
                    "render_fastq_normalize_primers_corpus_01_report.py",
                    "--repo-root",
                    str(repo_root),
                    "--corpus-root",
                    "/home/bijan/lu2024-12-24/.cache/corpus_01",
                    "--run-root",
                    str(run_root),
                    "--docs-root",
                    str(docs_root.relative_to(repo_root)),
                ]
                with mock.patch.object(
                    normalize_primers_report,
                    "load_corpus_spec",
                    return_value={"corpus_id": "corpus-01"},
                ), mock.patch.object(
                    normalize_primers_report,
                    "resolve_corpus_metadata",
                    return_value={
                        "sample_0001": {
                            "accession": "ACC1",
                            "era": "ancient",
                            "layout": "se",
                            "study_accession": "PRJ1",
                            "size_band": "under_100mb",
                        }
                    },
                ):
                    self.assertEqual(normalize_primers_report.main(), 0)
            finally:
                sys.argv = original_argv

            summary = json.loads((docs_root / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(
                summary["corpus_root"],
                "/home/bijan/lu2024-12-24/.cache/corpus_01",
            )
            self.assertEqual(summary["sample_scope"], "paired")
            self.assertEqual(summary["sample_scope"], "paired")

    def test_normalize_primers_briefing_summarizes_orientation_fraction(self) -> None:
        rows = [
            {
                "tool": "cutadapt",
                "runtime_s": "2.0",
                "read_retention": "1.0",
                "primer_trimmed_fraction": "0.04",
                "orientation_forward_fraction": "0.93",
                "exit_code": "0",
            },
            {
                "tool": "cutadapt",
                "runtime_s": "4.0",
                "read_retention": "1.0",
                "primer_trimmed_fraction": "0.06",
                "orientation_forward_fraction": "0.97",
                "exit_code": "0",
            },
        ]
        summary_rows = normalize_primers_briefing.tool_runtime_summary(rows)
        self.assertEqual(summary_rows[0]["tool"], "cutadapt")
        self.assertAlmostEqual(
            summary_rows[0]["median_orientation_forward_fraction"], 0.95
        )

    def test_deplete_rrna_report_contract_rejects_bundle_drift(self) -> None:
        run_manifest = {
            "tools": ["sortmerna"],
            "rrna_bundle_id": "sortmerna_v4_3_default_db",
            "min_identity": 0.95,
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "sortmerna",
                "database_artifact_id": "legacy_rrna_db",
                "min_identity": 0.95,
                "reads_in": 100,
                "reads_out": 90,
                "bases_in": 1000,
                "bases_out": 900,
            }
        ]
        with self.assertRaises(SystemExit):
            deplete_rrna_report.validate_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0001"],
            )

    def test_deplete_rrna_summary_preserves_configured_corpus_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            run_root = repo_root / "results" / "corpus_01" / "fastq.deplete_rrna" / "lunarc"
            docs_root = (
                repo_root / "docs" / "benchmark" / "fastq.deplete_rrna" / "corpus-01"
            )
            sample_report = (
                run_root / "bench" / "deplete_rrna" / "sample_0001" / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {"tool": "sortmerna"},
                                "execution": {"runtime_s": 1.5, "exit_code": 0},
                                "metrics": {
                                    "reads_in": 100,
                                    "reads_out": 98,
                                    "bases_in": 1000,
                                    "bases_out": 980,
                                    "rrna_fraction_removed": 0.02,
                                    "depletion_summary": {
                                        "reads_removed": 2,
                                        "bases_removed": 20,
                                        "database_artifact_id": "sortmerna_v4_3_default_db",
                                        "screening_engine": "sortmerna",
                                        "report_tsv": "rrna.tsv",
                                        "report_json": "rrna.json",
                                    },
                                },
                            }
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "platform": "lunarc-apptainer",
                        "corpus_root": "/home/bijan/bijux/corpus_01",
                        "stage_id": "fastq.deplete_rrna",
                        "scenario_id": "rrna_depletion_fairness",
                        "tools": ["sortmerna"],
                        "rrna_db": "/refs/sortmerna.fasta",
                        "rrna_bundle_digest": "sha256:test",
                        "rrna_bundle_size_bytes": 1234,
                        "rrna_bundle_id": "sortmerna_v4_3_default_db",
                        "min_identity": 0.95,
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "report_json": str(sample_report),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            original_argv = sys.argv
            try:
                sys.argv = [
                    "render_fastq_deplete_rrna_corpus_01_report.py",
                    "--repo-root",
                    str(repo_root),
                    "--corpus-root",
                    "/home/bijan/lu2024-12-24/.cache/corpus_01",
                    "--run-root",
                    str(run_root),
                    "--docs-root",
                    str(docs_root.relative_to(repo_root)),
                ]
                with mock.patch.object(
                    deplete_rrna_report,
                    "load_corpus_spec",
                    return_value={"corpus_id": "corpus-01"},
                ), mock.patch.object(
                    deplete_rrna_report,
                    "resolve_corpus_metadata",
                    return_value={
                        "sample_0001": {
                            "accession": "ACC1",
                            "era": "ancient",
                            "layout": "se",
                            "study_accession": "PRJ1",
                            "size_band": "under_100mb",
                        }
                    },
                ):
                    self.assertEqual(deplete_rrna_report.main(), 0)
            finally:
                sys.argv = original_argv

            summary = json.loads((docs_root / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(
                summary["corpus_root"],
                "/home/bijan/lu2024-12-24/.cache/corpus_01",
            )

    def test_deplete_host_report_contract_rejects_reference_drift(self) -> None:
        run_manifest = {
            "tools": ["bowtie2"],
            "reference_catalog_id": "host_reference",
            "reference_index_backend": "bowtie2_build",
            "host_identity_threshold": 0.95,
            "retain_unmapped_only": True,
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "bowtie2",
                "reference_catalog_id": "host_reference",
                "reference_index_backend": "bowtie2_build",
                "host_identity_threshold": 0.90,
                "retain_unmapped_only": True,
                "raw_backend_report_format": "bowtie2_met_file",
                "reads_in": 100,
                "reads_out": 90,
                "bases_in": 1000,
                "bases_out": 900,
            }
        ]
        with self.assertRaises(SystemExit):
            deplete_host_report.validate_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0001"],
            )

    def test_deplete_host_report_contract_requires_reference_lineage(self) -> None:
        with self.assertRaises(SystemExit):
            deplete_host_report.validate_run_manifest_contract(
                {
                    "dry_run": False,
                    "sample_limit": None,
                    "stage_id": support.DEPLETE_HOST_BENCHMARK_CONTRACT.stage_id,
                    "scenario_id": support.DEPLETE_HOST_BENCHMARK_CONTRACT.scenario_id,
                    "tool_kind": "benchmark",
                    "reference_index": "/refs/host/reference",
                    "reference_index_digest": "sha256:index",
                }
            )

    def test_deplete_host_summary_preserves_configured_corpus_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            run_root = repo_root / "results" / "corpus_01" / "fastq.deplete_host" / "lunarc"
            docs_root = (
                repo_root / "docs" / "benchmark" / "fastq.deplete_host" / "corpus-01"
            )
            sample_report = (
                run_root / "bench" / "deplete_host" / "sample_0001" / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {
                                    "tool": "bowtie2",
                                    "parameters": {
                                        "reference_catalog_id": "host_reference",
                                        "reference_index_backend": "bowtie2_build",
                                        "host_identity_threshold": 0.95,
                                        "retain_unmapped_only": True,
                                        "raw_backend_report_format": "bowtie2_met_file",
                                    },
                                },
                                "execution": {"runtime_s": 1.5, "exit_code": 0},
                                "metrics": {
                                    "reads_in": 100,
                                    "reads_out": 98,
                                    "bases_in": 1000,
                                    "bases_out": 980,
                                    "host_fraction_removed": 0.02,
                                    "pairs_in": 50,
                                    "pairs_out": 49,
                                },
                            }
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "platform": "lunarc-apptainer",
                        "corpus_root": "/home/bijan/bijux/corpus_01",
                        "stage_id": "fastq.deplete_host",
                        "scenario_id": "host_depletion_fairness",
                        "tool_kind": "benchmark",
                        "tools": ["bowtie2"],
                        "samples_failed": 0,
                        "reference_index": "/refs/host/reference",
                        "reference_index_digest": "sha256:index",
                        "reference_index_size_bytes": 1234,
                        "reference_index_lineage_json": "/refs/host/lineage.json",
                        "reference_index_lineage_digest": "sha256:lineage",
                        "reference_catalog_id": "host_reference",
                        "reference_index_backend": "bowtie2_build",
                        "host_identity_threshold": 0.95,
                        "retain_unmapped_only": True,
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "report_json": str(sample_report),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            original_argv = sys.argv
            try:
                sys.argv = [
                    "render_fastq_deplete_host_corpus_01_report.py",
                    "--repo-root",
                    str(repo_root),
                    "--corpus-root",
                    "/home/bijan/lu2024-12-24/.cache/corpus_01",
                    "--run-root",
                    str(run_root),
                    "--docs-root",
                    str(docs_root.relative_to(repo_root)),
                ]
                with mock.patch.object(
                    deplete_host_report,
                    "load_corpus_spec",
                    return_value={"corpus_id": "corpus-01"},
                ), mock.patch.object(
                    deplete_host_report,
                    "resolve_corpus_metadata",
                    return_value={
                        "sample_0001": {
                            "accession": "ACC1",
                            "era": "ancient",
                            "layout": "se",
                            "study_accession": "PRJ1",
                            "size_band": "under_100mb",
                        }
                    },
                ):
                    self.assertEqual(deplete_host_report.main(), 0)
            finally:
                sys.argv = original_argv

            summary = json.loads((docs_root / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(
                summary["corpus_root"],
                "/home/bijan/lu2024-12-24/.cache/corpus_01",
            )

    def test_deplete_reference_contaminants_report_contract_rejects_policy_drift(
        self,
    ) -> None:
        run_manifest = {
            "tools": ["bowtie2"],
            "reference_catalog_id": "contaminant_reference",
            "reference_index_backend": "bowtie2_build",
            "decoy_mode": "phix_and_spikeins",
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "bowtie2",
                "reference_catalog_id": "contaminant_reference",
                "reference_index_backend": "bowtie2_build",
                "decoy_mode": "adapter_dimers",
                "raw_backend_report_format": "bowtie2_met_file",
                "reads_in": 100,
                "reads_out": 90,
                "bases_in": 1000,
                "bases_out": 900,
            }
        ]
        with self.assertRaises(SystemExit):
            deplete_reference_contaminants_report.validate_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0001"],
            )

    def test_deplete_reference_contaminants_summary_preserves_configured_corpus_root(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            run_root = (
                repo_root
                / "results"
                / "corpus_01"
                / "fastq.deplete_reference_contaminants"
                / "lunarc"
            )
            docs_root = (
                repo_root
                / "docs"
                / "benchmark"
                / "fastq.deplete_reference_contaminants"
                / "corpus-01"
            )
            sample_report = (
                run_root
                / "bench"
                / "deplete_reference_contaminants"
                / "sample_0001"
                / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {
                                    "tool": "bowtie2",
                                    "parameters": {
                                        "reference_catalog_id": "contaminant_reference",
                                        "reference_index_backend": "bowtie2_build",
                                        "decoy_mode": "phix_and_spikeins",
                                        "raw_backend_report_format": "bowtie2_met_file",
                                    },
                                },
                                "execution": {"runtime_s": 1.5, "exit_code": 0},
                                "metrics": {
                                    "reads_in": 100,
                                    "reads_out": 97,
                                    "bases_in": 1000,
                                    "bases_out": 970,
                                    "contaminant_fraction_removed": 0.03,
                                },
                            }
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "platform": "lunarc-apptainer",
                        "corpus_root": "/home/bijan/bijux/corpus_01",
                        "stage_id": "fastq.deplete_reference_contaminants",
                        "scenario_id": "contaminant_depletion_fairness",
                        "tool_kind": "benchmark",
                        "tools": ["bowtie2"],
                        "samples_failed": 0,
                        "reference_index": "/refs/contaminants/reference",
                        "reference_index_digest": "sha256:index",
                        "reference_index_size_bytes": 1234,
                        "reference_catalog_id": "contaminant_reference",
                        "reference_index_backend": "bowtie2_build",
                        "decoy_mode": "phix_and_spikeins",
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "report_json": str(sample_report),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            original_argv = sys.argv
            try:
                sys.argv = [
                    "render_fastq_deplete_reference_contaminants_corpus_01_report.py",
                    "--repo-root",
                    str(repo_root),
                    "--corpus-root",
                    "/home/bijan/lu2024-12-24/.cache/corpus_01",
                    "--run-root",
                    str(run_root),
                    "--docs-root",
                    str(docs_root.relative_to(repo_root)),
                ]
                with mock.patch.object(
                    deplete_reference_contaminants_report,
                    "load_corpus_spec",
                    return_value={"corpus_id": "corpus-01"},
                ), mock.patch.object(
                    deplete_reference_contaminants_report,
                    "resolve_corpus_metadata",
                    return_value={
                        "sample_0001": {
                            "accession": "ACC1",
                            "era": "ancient",
                            "layout": "se",
                            "study_accession": "PRJ1",
                            "size_band": "under_100mb",
                        }
                    },
                ):
                    self.assertEqual(deplete_reference_contaminants_report.main(), 0)
            finally:
                sys.argv = original_argv

            summary = json.loads((docs_root / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(
                summary["corpus_root"],
                "/home/bijan/lu2024-12-24/.cache/corpus_01",
            )

    def test_screen_taxonomy_report_contract_rejects_database_lineage_drift(
        self,
    ) -> None:
        run_manifest = {
            "tools": ["kraken2"],
            "database_catalog_id": "taxonomy_reference",
            "database_artifact_id": "taxonomy_db",
            "database_namespace": "read_screening",
            "database_scope": "read_screening",
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "kraken2",
                "database_catalog_id": "taxonomy_reference",
                "database_artifact_id": "taxonomy_db_legacy",
                "database_namespace": "read_screening",
                "database_scope": "read_screening",
                "reads_in": 100,
                "reads_out": 100,
                "bases_in": 1000,
                "bases_out": 1000,
            }
        ]
        with self.assertRaises(SystemExit):
            screen_taxonomy_report.validate_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0001"],
            )

    def test_screen_taxonomy_report_contract_requires_database_lineage(self) -> None:
        with self.assertRaises(SystemExit):
            screen_taxonomy_report.validate_run_manifest_contract(
                {
                    "dry_run": False,
                    "sample_limit": None,
                    "stage_id": support.SCREEN_TAXONOMY_BENCHMARK_CONTRACT.stage_id,
                    "scenario_id": support.SCREEN_TAXONOMY_BENCHMARK_CONTRACT.scenario_id,
                    "tool_kind": "benchmark",
                    "database_root": "/refs/taxonomy_db",
                    "database_digest": "sha256:db",
                }
            )

    def test_screen_taxonomy_summary_preserves_configured_corpus_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            run_root = (
                repo_root / "results" / "corpus_01" / "fastq.screen_taxonomy" / "lunarc"
            )
            docs_root = (
                repo_root / "docs" / "benchmark" / "fastq.screen_taxonomy" / "corpus-01"
            )
            sample_report = (
                run_root / "bench" / "screen_taxonomy" / "sample_0001" / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {
                                    "tool": "kraken2",
                                    "database_artifact_id": "taxonomy_db",
                                    "database_namespace": "read_screening",
                                    "database_scope": "read_screening",
                                    "database_catalog_id": "taxonomy_reference",
                                    "database_root": "/refs/taxonomy",
                                    "database_digest": "sha256:test",
                                    "database_size_bytes": 1234,
                                    "database_lineage_json": "/refs/taxonomy/lineage.json",
                                    "database_lineage_digest": "sha256:lineage",
                                },
                                "execution": {"runtime_s": 1.5, "exit_code": 0},
                                "metrics": {
                                    "contamination_rate": 0.0,
                                    "classified_fraction": 1.0,
                                    "unclassified_fraction": 0.0,
                                    "top_taxon": "Homo sapiens",
                                },
                            }
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "platform": "lunarc-apptainer",
                        "corpus_root": "/home/bijan/bijux/corpus_01",
                        "stage_id": "fastq.screen_taxonomy",
                        "scenario_id": "screen_fairness",
                        "tools": ["kraken2"],
                        "samples_failed": 0,
                        "database_root": "/refs/taxonomy",
                        "database_digest": "sha256:test",
                        "database_size_bytes": 1234,
                        "database_lineage_json": "/refs/taxonomy/lineage.json",
                        "database_lineage_digest": "sha256:lineage",
                        "database_artifact_id": "taxonomy_db",
                        "database_namespace": "read_screening",
                        "database_scope": "read_screening",
                        "database_catalog_id": "taxonomy_reference",
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "report_json": str(sample_report),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            original_argv = sys.argv
            try:
                sys.argv = [
                    "render_fastq_screen_taxonomy_corpus_01_report.py",
                    "--repo-root",
                    str(repo_root),
                    "--corpus-root",
                    "/home/bijan/lu2024-12-24/.cache/corpus_01",
                    "--run-root",
                    str(run_root),
                    "--docs-root",
                    str(docs_root.relative_to(repo_root)),
                ]
                with mock.patch.object(
                    screen_taxonomy_report,
                    "load_corpus_spec",
                    return_value={"corpus_id": "corpus-01"},
                ), mock.patch.object(
                    screen_taxonomy_report,
                    "resolve_corpus_metadata",
                    return_value={
                        "sample_0001": {
                            "accession": "ACC1",
                            "era": "ancient",
                            "layout": "se",
                            "study_accession": "PRJ1",
                            "size_band": "under_100mb",
                        }
                    },
                ):
                    self.assertEqual(screen_taxonomy_report.main(), 0)
            finally:
                sys.argv = original_argv

            summary = json.loads((docs_root / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(
                summary["corpus_root"],
                "/home/bijan/lu2024-12-24/.cache/corpus_01",
            )

    def test_correct_errors_report_contract_rejects_policy_drift(self) -> None:
        run_manifest = {
            "tools": ["lighter"],
            "quality_encoding": "phred33",
            "kmer_size": 31,
            "musket_kmer_budget": 536870912,
            "genome_size": 2800000,
            "max_memory_gb": None,
            "trusted_kmer_artifact": "trusted.kmers",
            "conservative_mode": False,
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "layout": "se",
                "tool": "lighter",
                "paired_mode": "single_end",
                "quality_encoding": "phred33",
                "kmer_size": 29,
                "musket_kmer_budget": None,
                "genome_size": 2800000,
                "max_memory_gb": None,
                "trusted_kmer_artifact": "trusted.kmers",
                "conservative_mode": False,
                "reads_in": 100,
                "reads_out": 100,
                "bases_in": 1000,
                "bases_out": 1000,
                "corrected_reads": 10,
            }
        ]
        with self.assertRaises(SystemExit):
            correct_errors_report.validate_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0001"],
            )

    def test_correct_errors_summary_preserves_configured_corpus_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            run_root = repo_root / "results" / "corpus_01" / "fastq.correct_errors" / "lunarc"
            docs_root = (
                repo_root / "docs" / "benchmark" / "fastq.correct_errors" / "corpus-01"
            )
            sample_report = (
                run_root / "bench" / "correct_errors" / "sample_0001" / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {"tool": "lighter"},
                                "execution": {"runtime_s": 1.5, "exit_code": 0},
                                "metrics": {
                                    "metrics": {
                                        "paired_mode": "paired_end",
                                        "quality_encoding": "phred33",
                                        "kmer_size": 31,
                                        "musket_kmer_budget": None,
                                        "genome_size": 2800000,
                                        "max_memory_gb": None,
                                        "trusted_kmer_artifact": "trusted.kmers",
                                        "conservative_mode": False,
                                        "reads_in": 100,
                                        "reads_out": 100,
                                        "bases_in": 1000,
                                        "bases_out": 1000,
                                        "pairs_in": 50,
                                        "pairs_out": 50,
                                        "corrected_reads": 10,
                                        "mean_q_before": 30.0,
                                        "mean_q_after": 31.0,
                                        "kmer_fix_rate": 0.1,
                                    }
                                },
                            }
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "platform": "lunarc-apptainer",
                        "corpus_root": "/home/bijan/bijux/corpus_01",
                        "stage_id": "fastq.correct_errors",
                        "scenario_id": "correction_fairness",
                        "tool_kind": "benchmark",
                        "sample_scope": "paired",
                        "tools": ["lighter"],
                        "quality_encoding": "phred33",
                        "kmer_size": 31,
                        "musket_kmer_budget": 536870912,
                        "genome_size": 2800000,
                        "max_memory_gb": None,
                        "trusted_kmer_artifact": "trusted.kmers",
                        "trusted_kmer_artifact_digest": "sha256:trusted",
                        "trusted_kmer_artifact_size_bytes": 1234,
                        "conservative_mode": False,
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "layout": "pe",
                                "report_json": str(sample_report),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            original_argv = sys.argv
            try:
                sys.argv = [
                    "render_fastq_correct_errors_corpus_01_report.py",
                    "--repo-root",
                    str(repo_root),
                    "--corpus-root",
                    "/home/bijan/lu2024-12-24/.cache/corpus_01",
                    "--run-root",
                    str(run_root),
                    "--docs-root",
                    str(docs_root.relative_to(repo_root)),
                ]
                with mock.patch.object(
                    correct_errors_report,
                    "load_corpus_spec",
                    return_value={"corpus_id": "corpus-01"},
                ), mock.patch.object(
                    correct_errors_report,
                    "resolve_corpus_metadata",
                    return_value={
                        "sample_0001": {
                            "accession": "ACC1",
                            "era": "modern",
                            "layout": "pe",
                            "study_accession": "PRJ1",
                            "size_band": "under_100mb",
                        }
                    },
                ):
                    self.assertEqual(correct_errors_report.main(), 0)
            finally:
                sys.argv = original_argv

            summary = json.loads((docs_root / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(
                summary["corpus_root"],
                "/home/bijan/lu2024-12-24/.cache/corpus_01",
            )

    def test_extract_umis_report_contract_rejects_single_end_row(self) -> None:
        run_manifest = {
            "tools": ["umi_tools"],
            "umi_pattern": "NNNNNNNN",
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "layout": "se",
                "tool": "umi_tools",
                "paired_mode": "single_end",
                "umi_pattern": "NNNNNNNN",
                "raw_backend_report_format": "umi_tools_log",
                "reads_in": 100,
                "reads_out": 100,
                "reads_with_umi": 100,
            }
        ]
        with self.assertRaises(SystemExit):
            extract_umis_report.validate_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0001"],
            )

    def test_extract_umis_report_reads_fallback_values_from_context_parameters(
        self,
    ) -> None:
        record = {
            "context": {
                "parameters": {
                    "umi_pattern": "NNNNNNNN",
                    "raw_backend_report_format": "umi_tools_log",
                }
            },
            "metrics": {
                "metrics": {
                    "reads_in": 100,
                    "reads_out": 100,
                    "reads_with_umi": 100,
                }
            },
        }

        self.assertEqual(
            extract_umis_report.normalize_parameter(record, "umi_pattern"),
            "NNNNNNNN",
        )
        self.assertEqual(
            extract_umis_report.normalize_parameter(record, "raw_backend_report_format"),
            "umi_tools_log",
        )

    def test_extract_umis_summary_preserves_configured_corpus_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            run_root = repo_root / "results" / "corpus_01" / "fastq.extract_umis" / "lunarc"
            docs_root = (
                repo_root / "docs" / "benchmark" / "fastq.extract_umis" / "corpus-01"
            )
            sample_report = (
                run_root / "bench" / "extract_umis" / "sample_0001" / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {
                                    "tool": "umi_tools",
                                    "parameters": {
                                        "paired_mode": "paired_end",
                                        "umi_pattern": "NNNNNNNN",
                                        "raw_backend_report_format": "umi_tools_log",
                                    },
                                },
                                "execution": {"runtime_s": 1.5, "exit_code": 0},
                                "metrics": {
                                    "metrics": {
                                        "reads_in": 100,
                                        "reads_out": 100,
                                        "bases_in": 1000,
                                        "bases_out": 1000,
                                        "pairs_in": 50,
                                        "pairs_out": 50,
                                        "reads_with_umi": 100,
                                        "mean_q_before": 32.0,
                                        "mean_q_after": 32.0,
                                    }
                                },
                            }
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "platform": "lunarc-apptainer",
                        "corpus_root": "/home/bijan/bijux/corpus_01",
                        "stage_id": "fastq.extract_umis",
                        "scenario_id": "umi_extraction_fairness",
                        "tool_kind": "benchmark",
                        "sample_scope": "paired",
                        "tools": ["umi_tools"],
                        "samples_failed": 0,
                        "umi_pattern": "NNNNNNNN",
                        "allow_missing_umi_headers": False,
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "layout": "pe",
                                "report_json": str(sample_report),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            original_argv = sys.argv
            try:
                sys.argv = [
                    "render_fastq_extract_umis_corpus_01_report.py",
                    "--repo-root",
                    str(repo_root),
                    "--corpus-root",
                    "/home/bijan/lu2024-12-24/.cache/corpus_01",
                    "--run-root",
                    str(run_root),
                    "--docs-root",
                    str(docs_root.relative_to(repo_root)),
                ]
                with mock.patch.object(
                    extract_umis_report,
                    "load_corpus_spec",
                    return_value={"corpus_id": "corpus-01"},
                ), mock.patch.object(
                    extract_umis_report,
                    "resolve_corpus_metadata",
                    return_value={
                        "sample_0001": {
                            "accession": "ACC1",
                            "era": "ancient",
                            "layout": "pe",
                            "study_accession": "PRJ1",
                            "size_band": "under_100mb",
                        }
                    },
                ):
                    self.assertEqual(extract_umis_report.main(), 0)
            finally:
                sys.argv = original_argv

            summary = json.loads((docs_root / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(
                summary["corpus_root"],
                "/home/bijan/lu2024-12-24/.cache/corpus_01",
            )

    def test_filter_reads_report_contract_rejects_parameter_drift(self) -> None:
        run_manifest = {
            "tools": ["bbduk", "fastp", "prinseq", "seqkit"],
            "max_n": 0,
            "max_n_fraction": None,
            "max_n_count": 3,
            "low_complexity_threshold": 20.0,
            "entropy_threshold": 18.0,
            "kmer_ref": None,
            "polyx_policy": "trim",
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "fastp",
                "max_n": 0,
                "max_n_fraction": None,
                "max_n_count": 2,
                "low_complexity_threshold": 20.0,
                "entropy_threshold": 18.0,
                "kmer_ref": None,
                "polyx_policy": "trim",
                "raw_backend_report_format": "fastp_json",
                "reads_in": 100,
                "reads_out": 90,
                "bases_in": 1000,
                "bases_out": 900,
            }
        ]
        with self.assertRaises(SystemExit):
            filter_reads_report.validate_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0001"],
            )

    def test_filter_low_complexity_report_contract_rejects_missing_tool_row(self) -> None:
        run_manifest = {
            "tools": ["bbduk", "prinseq"],
            "entropy_threshold": 0.55,
            "polyx_threshold": 20,
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "bbduk",
                "entropy_threshold": 0.55,
                "polyx_threshold": 20,
                "raw_backend_report_format": "bbduk_stats",
            }
        ]
        with self.assertRaises(SystemExit):
            filter_low_complexity_report.validate_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
            )

    def test_filter_low_complexity_summary_preserves_configured_corpus_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            run_root = (
                repo_root
                / "results"
                / "corpus_01"
                / "fastq.filter_low_complexity"
                / "lunarc"
            )
            docs_root = (
                repo_root
                / "docs"
                / "benchmark"
                / "fastq.filter_low_complexity"
                / "corpus-01"
            )
            sample_report = (
                run_root
                / "bench"
                / "filter_low_complexity"
                / "sample_0001"
                / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {
                                    "tool": "bbduk",
                                    "parameters": {
                                        "entropy_threshold": 0.55,
                                        "polyx_threshold": 20,
                                        "raw_backend_report_format": "bbduk_stats",
                                    },
                                },
                                "execution": {"runtime_s": 1.5, "exit_code": 0},
                                "metrics": {
                                    "metrics": {
                                        "reads_in": 100,
                                        "reads_out": 92,
                                        "bases_in": 1000,
                                        "bases_out": 930,
                                        "reads_removed_low_complexity": 8,
                                        "delta_metrics": {
                                            "base_retention": 0.93,
                                            "read_retention": 0.92,
                                            "mean_q_delta": 0.1,
                                        },
                                    }
                                },
                            },
                            {
                                "context": {
                                    "tool": "prinseq",
                                    "parameters": {
                                        "entropy_threshold": 0.55,
                                        "polyx_threshold": 20,
                                    },
                                },
                                "execution": {"runtime_s": 1.8, "exit_code": 0},
                                "metrics": {
                                    "metrics": {
                                        "reads_in": 100,
                                        "reads_out": 90,
                                        "bases_in": 1000,
                                        "bases_out": 920,
                                        "reads_removed_low_complexity": 10,
                                        "delta_metrics": {
                                            "base_retention": 0.92,
                                            "read_retention": 0.90,
                                            "mean_q_delta": 0.2,
                                        },
                                    }
                                },
                            },
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "platform": "lunarc-apptainer",
                        "corpus_root": "/home/bijan/bijux/corpus_01",
                        "stage_id": "fastq.filter_low_complexity",
                        "scenario_id": "low_complexity_fairness",
                        "samples_total": 1,
                        "samples_failed": 0,
                        "tools": ["bbduk", "prinseq"],
                        "entropy_threshold": 0.55,
                        "polyx_threshold": 20,
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "layout": "se",
                                "report_json": str(sample_report),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            original_argv = sys.argv
            try:
                sys.argv = [
                    "render_fastq_filter_low_complexity_corpus_01_report.py",
                    "--repo-root",
                    str(repo_root),
                    "--corpus-root",
                    "/home/bijan/lu2024-12-24/.cache/corpus_01",
                    "--run-root",
                    str(run_root),
                    "--docs-root",
                    str(docs_root.relative_to(repo_root)),
                ]
                with mock.patch.object(
                    filter_low_complexity_report,
                    "load_corpus_spec",
                    return_value={"corpus_id": "corpus-01"},
                ), mock.patch.object(
                    filter_low_complexity_report,
                    "resolve_corpus_metadata",
                    return_value={
                        "sample_0001": {
                            "accession": "ACC1",
                            "era": "modern",
                            "layout": "se",
                            "study_accession": "PRJ1",
                            "size_band": "under_100mb",
                        }
                    },
                ):
                    self.assertEqual(filter_low_complexity_report.main(), 0)
            finally:
                sys.argv = original_argv

            summary = json.loads((docs_root / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(
                summary["corpus_root"],
                "/home/bijan/lu2024-12-24/.cache/corpus_01",
            )

    def test_remove_duplicates_report_contract_rejects_single_end_row(self) -> None:
        run_manifest = {
            "tools": ["clumpify", "fastuniq"],
            "dedup_mode": "exact",
            "keep_order": True,
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "layout": "se",
                "tool": "clumpify",
                "dedup_mode": "exact",
                "keep_order": True,
                "paired_mode": "single_end",
                "raw_backend_report_format": "clumpify_log",
                "reads_in": 100,
                "reads_out": 90,
                "duplicate_reads": 10,
                "pair_count_match": None,
            }
        ]
        with self.assertRaises(SystemExit):
            remove_duplicates_report.validate_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0001"],
            )

    def test_remove_duplicates_summary_preserves_configured_corpus_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            run_root = (
                repo_root / "results" / "corpus_01" / "fastq.remove_duplicates" / "lunarc"
            )
            docs_root = (
                repo_root
                / "docs"
                / "benchmark"
                / "fastq.remove_duplicates"
                / "corpus-01"
            )
            sample_report = (
                run_root / "bench" / "remove_duplicates" / "sample_0001" / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {
                                    "tool": "clumpify",
                                    "parameters": {
                                        "dedup_mode": "exact",
                                        "keep_order": True,
                                        "paired_mode": "paired_end",
                                        "pair_count_match": True,
                                        "raw_backend_report_format": "clumpify_log",
                                    },
                                },
                                "execution": {"runtime_s": 1.5, "exit_code": 0},
                                "metrics": {
                                    "layout": "pe",
                                    "reads_in": 100,
                                    "reads_out": 90,
                                    "duplicate_reads": 10,
                                    "dedup_rate": 0.1,
                                },
                            }
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "platform": "lunarc-apptainer",
                        "stage_id": "fastq.remove_duplicates",
                        "scenario_id": "dedup_fairness",
                        "sample_scope": "paired",
                        "corpus_root": "/home/bijan/bijux/corpus_01",
                        "tools": ["clumpify"],
                        "dedup_mode": "exact",
                        "keep_order": True,
                        "samples_failed": 0,
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "layout": "pe",
                                "report_json": str(sample_report),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            original_argv = sys.argv
            try:
                sys.argv = [
                    "render_fastq_remove_duplicates_corpus_01_report.py",
                    "--repo-root",
                    str(repo_root),
                    "--corpus-root",
                    "/home/bijan/lu2024-12-24/.cache/corpus_01",
                    "--run-root",
                    str(run_root),
                    "--docs-root",
                    str(docs_root.relative_to(repo_root)),
                ]
                with mock.patch.object(
                    remove_duplicates_report,
                    "load_corpus_spec",
                    return_value={"corpus_id": "corpus-01"},
                ), mock.patch.object(
                    remove_duplicates_report,
                    "resolve_corpus_metadata",
                    return_value={
                        "sample_0001": {
                            "accession": "ACC1",
                            "era": "ancient",
                            "layout": "pe",
                            "study_accession": "PRJ1",
                            "size_band": "under_100mb",
                        }
                    },
                ):
                    self.assertEqual(remove_duplicates_report.main(), 0)
            finally:
                sys.argv = original_argv

            summary = json.loads((docs_root / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(
                summary["corpus_root"],
                "/home/bijan/lu2024-12-24/.cache/corpus_01",
            )
            self.assertEqual(summary["sample_scope"], "paired")

    def test_filter_low_complexity_briefing_summarizes_removed_reads(self) -> None:
        rows = [
            {
                "tool": "bbduk",
                "runtime_s": "2.0",
                "base_retention": "0.92",
                "read_retention": "0.90",
                "reads_removed_low_complexity": "10",
                "mean_q_delta": "0.2",
                "exit_code": "0",
            },
            {
                "tool": "bbduk",
                "runtime_s": "4.0",
                "base_retention": "0.91",
                "read_retention": "0.89",
                "reads_removed_low_complexity": "12",
                "mean_q_delta": "0.3",
                "exit_code": "0",
            },
        ]
        summary_rows = filter_low_complexity_briefing.tool_runtime_summary(rows)
        self.assertEqual(summary_rows[0]["tool"], "bbduk")
        self.assertEqual(summary_rows[0]["mean_reads_removed_low_complexity"], 11.0)

    def test_filter_reads_briefing_summarizes_reads_dropped(self) -> None:
        rows = [
            {
                "tool": "fastp",
                "runtime_s": "2.0",
                "base_retention": "0.96",
                "read_retention": "0.95",
                "reads_dropped": "10",
                "reads_removed_low_complexity": "3",
                "reads_removed_by_n": "2",
                "mean_q_delta": "0.1",
                "exit_code": "0",
            },
            {
                "tool": "fastp",
                "runtime_s": "4.0",
                "base_retention": "0.95",
                "read_retention": "0.94",
                "reads_dropped": "14",
                "reads_removed_low_complexity": "4",
                "reads_removed_by_n": "1",
                "mean_q_delta": "0.2",
                "exit_code": "0",
            },
        ]
        summary_rows = filter_reads_briefing.tool_runtime_summary(rows)
        self.assertEqual(summary_rows[0]["tool"], "fastp")
        self.assertEqual(summary_rows[0]["mean_reads_dropped"], 12.0)
        self.assertEqual(summary_rows[0]["mean_reads_removed_by_n"], 1.5)

    def test_deplete_rrna_briefing_summarizes_fraction_removed(self) -> None:
        rows = [
            {
                "tool": "sortmerna",
                "runtime_s": "1.0",
                "read_retention": "0.98",
                "base_retention": "0.98",
                "rrna_fraction_removed": "0.02",
                "reads_removed": "20",
                "exit_code": "0",
            },
            {
                "tool": "sortmerna",
                "runtime_s": "1.2",
                "read_retention": "0.99",
                "base_retention": "0.99",
                "rrna_fraction_removed": "0.01",
                "reads_removed": "10",
                "exit_code": "0",
            },
        ]
        summary_rows = deplete_rrna_briefing.tool_runtime_summary(rows)
        self.assertEqual(summary_rows[0]["tool"], "sortmerna")
        self.assertAlmostEqual(summary_rows[0]["mean_rrna_fraction_removed"], 0.015)
        self.assertAlmostEqual(summary_rows[0]["mean_reads_removed"], 15.0)

    def test_deplete_host_briefing_summarizes_fraction_removed(self) -> None:
        rows = [
            {
                "tool": "bowtie2",
                "runtime_s": "1.0",
                "read_retention": "0.70",
                "base_retention": "0.72",
                "host_fraction_removed": "0.30",
                "reads_removed": "60",
                "exit_code": "0",
            },
            {
                "tool": "bowtie2",
                "runtime_s": "1.4",
                "read_retention": "0.68",
                "base_retention": "0.71",
                "host_fraction_removed": "0.32",
                "reads_removed": "64",
                "exit_code": "0",
            },
        ]
        summary_rows = deplete_host_briefing.tool_runtime_summary(rows)
        self.assertEqual(summary_rows[0]["tool"], "bowtie2")
        self.assertAlmostEqual(summary_rows[0]["mean_host_fraction_removed"], 0.31)
        self.assertAlmostEqual(summary_rows[0]["mean_reads_removed"], 62.0)

    def test_deplete_reference_contaminants_briefing_summarizes_fraction_removed(
        self,
    ) -> None:
        rows = [
            {
                "tool": "bowtie2",
                "runtime_s": "1.0",
                "read_retention": "0.96",
                "base_retention": "0.97",
                "contaminant_fraction_removed": "0.04",
                "reads_removed": "8",
                "exit_code": "0",
            },
            {
                "tool": "bowtie2",
                "runtime_s": "1.4",
                "read_retention": "0.95",
                "base_retention": "0.96",
                "contaminant_fraction_removed": "0.05",
                "reads_removed": "10",
                "exit_code": "0",
            },
        ]
        summary_rows = deplete_reference_contaminants_briefing.tool_runtime_summary(rows)
        self.assertEqual(summary_rows[0]["tool"], "bowtie2")
        self.assertAlmostEqual(
            summary_rows[0]["mean_contaminant_fraction_removed"], 0.045
        )
        self.assertAlmostEqual(summary_rows[0]["mean_reads_removed"], 9.0)

    def test_screen_taxonomy_briefing_summarizes_contamination_and_classification(
        self,
    ) -> None:
        rows = [
            {
                "tool": "kraken2",
                "runtime_s": "2.0",
                "contamination_rate": "0.03",
                "classified_fraction": "0.97",
                "unclassified_fraction": "0.03",
                "exit_code": "0",
            },
            {
                "tool": "kraken2",
                "runtime_s": "4.0",
                "contamination_rate": "0.05",
                "classified_fraction": "0.95",
                "unclassified_fraction": "0.05",
                "exit_code": "0",
            },
        ]
        summary_rows = screen_taxonomy_briefing.tool_runtime_summary(rows)
        self.assertEqual(summary_rows[0]["tool"], "kraken2")
        self.assertAlmostEqual(summary_rows[0]["mean_contamination_rate"], 0.04)
        self.assertAlmostEqual(summary_rows[0]["mean_classified_fraction"], 0.96)

    def test_correct_errors_briefing_summarizes_quality_uplift(self) -> None:
        rows = [
            {
                "tool": "lighter",
                "runtime_s": "2.0",
                "read_retention": "1.0",
                "corrected_reads": "10",
                "kmer_fix_rate": "0.04",
                "mean_q_delta": "0.8",
                "exit_code": "0",
            },
            {
                "tool": "lighter",
                "runtime_s": "4.0",
                "read_retention": "1.0",
                "corrected_reads": "12",
                "kmer_fix_rate": "0.06",
                "mean_q_delta": "1.2",
                "exit_code": "0",
            },
        ]
        summary_rows = correct_errors_briefing.tool_runtime_summary(rows)
        self.assertEqual(summary_rows[0]["tool"], "lighter")
        self.assertAlmostEqual(summary_rows[0]["mean_kmer_fix_rate"], 0.05)
        self.assertAlmostEqual(summary_rows[0]["mean_quality_uplift"], 1.0)

    def test_extract_umis_briefing_summarizes_detection_fraction(self) -> None:
        rows = [
            {
                "tool": "umi_tools",
                "runtime_s": "2.0",
                "read_retention": "1.0",
                "reads_with_umi": "180",
                "reads_with_umi_fraction": "0.90",
                "exit_code": "0",
            },
            {
                "tool": "umi_tools",
                "runtime_s": "4.0",
                "read_retention": "1.0",
                "reads_with_umi": "190",
                "reads_with_umi_fraction": "0.95",
                "exit_code": "0",
            },
        ]
        summary_rows = extract_umis_briefing.tool_runtime_summary(rows)
        self.assertEqual(summary_rows[0]["tool"], "umi_tools")
        self.assertAlmostEqual(summary_rows[0]["mean_reads_with_umi"], 185.0)
        self.assertAlmostEqual(
            summary_rows[0]["mean_reads_with_umi_fraction"], 0.925
        )

    def test_remove_duplicates_briefing_summarizes_duplicate_reads(self) -> None:
        rows = [
            {
                "tool": "clumpify",
                "runtime_s": "2.0",
                "dedup_rate": "0.10",
                "duplicate_reads": "10",
                "exit_code": "0",
            },
            {
                "tool": "clumpify",
                "runtime_s": "4.0",
                "dedup_rate": "0.12",
                "duplicate_reads": "12",
                "exit_code": "0",
            },
        ]
        summary_rows = remove_duplicates_briefing.tool_runtime_summary(rows)
        self.assertEqual(summary_rows[0]["tool"], "clumpify")
        self.assertEqual(summary_rows[0]["mean_duplicate_reads"], 11.0)

    def test_normalize_results_mirror_moves_raw_lunarc_tree_into_canonical_layout(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            results_root = Path(tmpdir) / "results"
            raw_run_root = (
                results_root
                / "home"
                / "bijan"
                / "bijux"
                / "results"
                / "corpus_01"
                / "fastq.merge_pairs"
                / "lunarc"
            )
            raw_run_root.mkdir(parents=True)
            (raw_run_root / "run_manifest.json").write_text("{}", encoding="utf-8")

            report = normalize_results_mirror.normalize_results_root(
                results_root,
                "corpus_01",
                dry_run=False,
            )

            canonical_run_root = results_root / "corpus_01" / "fastq.merge_pairs" / "lunarc"
            self.assertTrue((canonical_run_root / "run_manifest.json").is_file())
            self.assertFalse(raw_run_root.exists())
            self.assertEqual(report["actions"][0]["status"], "moved")

    def test_normalize_results_mirror_parse_args_uses_workspace_results_root(self) -> None:
        original_argv = sys.argv
        try:
            with mock.patch.object(
                normalize_results_mirror,
                "benchmark_local_results_root",
                return_value=Path("/tmp/workspace-results"),
            ):
                sys.argv = ["normalize_lunarc_results_mirror.py"]
                args = normalize_results_mirror.parse_args()
        finally:
            sys.argv = original_argv

        self.assertEqual(args.results_root, "/tmp/workspace-results")

    def test_normalize_results_mirror_skips_existing_canonical_target(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            results_root = Path(tmpdir) / "results"
            raw_run_root = (
                results_root
                / "home"
                / "bijan"
                / "bijux"
                / "results"
                / "corpus_01"
                / "fastq.merge_pairs"
                / "lunarc"
            )
            canonical_run_root = results_root / "corpus_01" / "fastq.merge_pairs" / "lunarc"
            raw_run_root.mkdir(parents=True)
            canonical_run_root.mkdir(parents=True)

            report = normalize_results_mirror.normalize_results_root(
                results_root,
                "corpus_01",
                dry_run=False,
            )

            self.assertTrue(raw_run_root.exists())
            self.assertEqual(report["actions"][0]["status"], "skipped_existing_target")

    def test_normalize_results_mirror_moves_legacy_cache_results_tree(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            results_root = Path(tmpdir) / "results"
            raw_run_root = (
                results_root
                / "home"
                / "bijan"
                / "lu2024-12-24"
                / ".cache"
                / "bijux-dna-results"
                / "corpus_01"
                / "fastq.correct_errors"
                / "lunarc"
            )
            raw_run_root.mkdir(parents=True)
            (raw_run_root / "run_manifest.json").write_text("{}", encoding="utf-8")

            report = normalize_results_mirror.normalize_results_root(
                results_root,
                "corpus_01",
                dry_run=False,
            )

            canonical_run_root = (
                results_root / "corpus_01" / "fastq.correct_errors" / "lunarc"
            )
            self.assertTrue((canonical_run_root / "run_manifest.json").is_file())
            self.assertFalse(raw_run_root.exists())
            self.assertEqual(report["actions"][0]["target"], str(canonical_run_root))

    def test_repair_results_manifests_parse_args_uses_workspace_results_root(self) -> None:
        original_argv = sys.argv
        try:
            with mock.patch.object(
                repair_results_manifests,
                "benchmark_local_results_root",
                return_value=Path("/tmp/workspace-results"),
            ):
                sys.argv = ["repair_corpus_01_fastq_result_manifests.py"]
                args = repair_results_manifests.parse_args()
        finally:
            sys.argv = original_argv

        self.assertEqual(args.results_root, "/tmp/workspace-results")

    def test_repair_results_manifests_reconstructs_detect_adapters_run_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            run_root = (
                Path(tmpdir)
                / "results"
                / "corpus_01"
                / "fastq.detect_adapters"
                / "lunarc"
            )
            sample_report = (
                run_root
                / "bench"
                / "detect_adapters"
                / "sample_0001"
                / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {
                                    "platform": "lunarc-apptainer",
                                    "parameters": {
                                        "input_r1": "/home/bijan/bijux/corpus_01/normalized/sample_0001_R1.fastq.gz",
                                        "out_dir": "/home/bijan/bijux/results/corpus_01/fastq.detect_adapters/lunarc/bench/detect_adapters/sample_0001/tools/fastqc",
                                        "report_json": "/home/bijan/bijux/results/corpus_01/fastq.detect_adapters/lunarc/bench/detect_adapters/sample_0001/tools/fastqc/adapter_report.json",
                                        "threads": 1,
                                        "tool": "fastqc",
                                    },
                                }
                            }
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            result = repair_results_manifests.repair_stage(
                run_root,
                "fastq.detect_adapters",
            )
            manifest = json.loads((run_root / "run_manifest.json").read_text(encoding="utf-8"))

            self.assertEqual(result["status"], "reconstructed-stage-run-manifest")
            self.assertEqual(manifest["stage_id"], "fastq.detect_adapters")
            self.assertEqual(manifest["scenario_id"], "detect_adapters_fairness")
            self.assertEqual(manifest["runs"][0]["report_json"], str(sample_report.resolve()))

    def test_repair_results_manifests_uses_workspace_remote_repo_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            run_root = (
                Path(tmpdir)
                / "results"
                / "corpus_01"
                / "fastq.detect_adapters"
                / "lunarc"
            )
            sample_report = (
                run_root
                / "bench"
                / "detect_adapters"
                / "sample_0001"
                / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {
                                    "platform": "lunarc-apptainer",
                                    "parameters": {
                                        "input_r1": "/home/bijan/bijux/corpus_01/normalized/sample_0001_R1.fastq.gz",
                                        "out_dir": "/home/bijan/bijux/results/corpus_01/fastq.detect_adapters/lunarc/bench/detect_adapters/sample_0001/tools/fastqc",
                                        "report_json": "/home/bijan/bijux/results/corpus_01/fastq.detect_adapters/lunarc/bench/detect_adapters/sample_0001/tools/fastqc/adapter_report.json",
                                        "threads": 1,
                                        "tool": "fastqc",
                                    },
                                }
                            }
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            with mock.patch.object(
                repair_results_manifests,
                "benchmark_remote_repo_root",
                return_value=Path("/remote/worktree/bijux-dna"),
            ):
                repair_results_manifests.repair_stage(
                    run_root,
                    "fastq.detect_adapters",
                )
            manifest = json.loads((run_root / "run_manifest.json").read_text(encoding="utf-8"))

            self.assertEqual(manifest["repo_root"], "/remote/worktree/bijux-dna")

    def test_repair_results_manifests_refuses_partial_validate_tool_roster(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            run_root = (
                Path(tmpdir)
                / "results"
                / "corpus_01"
                / "fastq.validate_reads"
                / "lunarc"
            )
            sample_report = (
                run_root
                / "bench"
                / "validate_reads"
                / "sample_0001"
                / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {
                                    "tool": "fastqvalidator",
                                    "platform": "lunarc-apptainer",
                                    "parameters": {
                                        "input_r1": "/home/bijan/bijux/corpus_01/normalized/sample_0001_R1.fastq.gz",
                                        "threads": 4,
                                    },
                                }
                            }
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            result = repair_results_manifests.repair_stage(
                run_root,
                "fastq.validate_reads",
            )

            self.assertEqual(result["status"], "tool-roster-incomplete")
            self.assertFalse((run_root / "run_manifest.json").exists())

    def test_repair_results_manifests_normalizes_merge_report_paths(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            run_root = (
                Path(tmpdir)
                / "results"
                / "corpus_01"
                / "fastq.merge_pairs"
                / "lunarc"
            )
            sample_report = (
                run_root
                / "bench"
                / "merge_pairs"
                / "sample_0001"
                / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            sample_report.write_text('{"records":[{}]}\n', encoding="utf-8")
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "stage_id": "fastq.merge_pairs",
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "report_json": "/home/bijan/bijux/results/corpus_01/fastq.merge_pairs/lunarc/bench/merge/sample_0001/report.json",
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            result = repair_results_manifests.repair_stage(
                run_root,
                "fastq.merge_pairs",
            )
            manifest = json.loads((run_root / "run_manifest.json").read_text(encoding="utf-8"))

            self.assertEqual(result["status"], "updated-existing-manifest")
            self.assertEqual(manifest["runs"][0]["report_json"], str(sample_report.resolve()))


class CorpusBenchmarkDocsAuditTests(unittest.TestCase):
    def test_audit_docs_reports_missing_stage_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            docs_root = Path(tmpdir) / "docs" / "benchmark"
            corpus_spec = repo_root / "configs" / "runtime" / "corpora"
            corpus_spec.mkdir(parents=True)
            (corpus_spec / "corpus-01.toml").write_text(
                "\n".join(
                    [
                        'corpus_id = "corpus-01"',
                        "target_ancient_se = 1",
                        "target_ancient_pe = 1",
                        "target_modern_se = 1",
                        "target_modern_pe = 1",
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            stage_root = docs_root / "fastq.validate_reads"
            corpus_root = stage_root / "corpus-01"
            corpus_root.mkdir(parents=True)
            (stage_root / "corpus-01-method.md").write_text("# method\n", encoding="utf-8")
            (corpus_root / "summary.json").write_text(
                json.dumps(
                    {
                        "stage_id": "fastq.validate_reads",
                        "scenario_id": "validation_fairness",
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (corpus_root / "sample_results.csv").write_text("sample_id,tool\n", encoding="utf-8")
            report = benchmark_docs_audit.audit_docs(
                docs_root,
                repo_root=repo_root,
                stage_contracts=[
                    support.CorpusBenchmarkContract(
                        stage_id="fastq.validate_reads",
                        scenario_id="validation_fairness",
                        tools=["fastqvalidator"],
                    )
                ],
                exclusions=[],
            )
            validate_report = next(
                stage for stage in report["stages"] if stage["stage_id"] == "fastq.validate_reads"
            )

            self.assertEqual(validate_report["status"], "incomplete")
            self.assertGreaterEqual(validate_report["issue_count"], 4)
            self.assertTrue(
                any(issue["issue_id"] == "missing-lunarc-md" for issue in validate_report["issues"])
            )

    def test_render_markdown_summarizes_completion_and_issue_count(self) -> None:
        report = {
            "benchmarkable_stage_count": 3,
            "applicable_stage_count": 2,
            "completed_stage_count": 1,
            "incomplete_stage_count": 1,
            "excluded_stage_count": 1,
            "issue_count": 3,
            "excluded_stages": [
                {
                    "stage_id": "fastq.index_reference",
                    "reason": "reference bundle benchmark",
                }
            ],
            "stages": [
                {
                    "stage_id": "fastq.validate_reads",
                    "status": "complete",
                    "issue_count": 0,
                    "issues": [],
                    "sample_scope": "full",
                },
                {
                    "stage_id": "fastq.trim_reads",
                    "status": "incomplete",
                    "issue_count": 3,
                    "sample_scope": "full",
                    "issues": [
                        {
                            "issue_id": "missing-corpus-dir",
                            "detail": "missing docs/benchmark/fastq.trim_reads/corpus-01",
                        }
                    ],
                },
            ],
        }

        markdown = benchmark_docs_audit.render_markdown(report)

        self.assertIn("Benchmarkable governed stages: `3`", markdown)
        self.assertIn("Completed stage dossiers: `1`", markdown)
        self.assertIn("Publication issues: `3`", markdown)
        self.assertIn("`fastq.trim_reads`: `incomplete` (`3` issues, scope `full`)", markdown)
        self.assertIn("`fastq.index_reference`: reference bundle benchmark", markdown)

    def test_audit_docs_appends_supplemental_findings(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            docs_root = repo_root / "docs" / "benchmark"
            corpus_spec = repo_root / "configs" / "runtime" / "corpora"
            corpus_spec.mkdir(parents=True)
            (corpus_spec / "corpus-01.toml").write_text(
                "\n".join(
                    [
                        'corpus_id = "corpus-01"',
                        'preferred_root = "/tmp/corpus_01"',
                        "target_ancient_se = 1",
                        "target_ancient_pe = 0",
                        "target_modern_se = 1",
                        "target_modern_pe = 0",
                    ]
                ),
                encoding="utf-8",
            )
            stage_root = docs_root / "fastq.validate_reads"
            stage_root.mkdir(parents=True)
            (stage_root / "corpus-01-method.md").write_text("# method\n", encoding="utf-8")
            (stage_root / "corpus-01").mkdir()

            report = benchmark_docs_audit.audit_docs(
                docs_root,
                repo_root=repo_root,
                stage_contracts=[
                    support.CorpusBenchmarkContract(
                        stage_id="fastq.validate_reads",
                        scenario_id="validation_fairness",
                        tools=["fastqvalidator"],
                    )
                ],
                exclusions=[],
                supplemental_findings={
                    "fastq.validate_reads": [
                        benchmark_docs_audit.StageAuditIssue(
                            stage_id="fastq.validate_reads",
                            issue_id="fixture-integrity-gap",
                            severity="error",
                            detail="synthetic fixture does not represent a publishable benchmark lineage",
                        )
                    ]
                },
            )

            validate_report = next(
                stage for stage in report["stages"] if stage["stage_id"] == "fastq.validate_reads"
            )

            self.assertEqual(validate_report["status"], "incomplete")
            self.assertTrue(
                any(
                    issue["issue_id"] == "fixture-integrity-gap"
                    for issue in validate_report["issues"]
                )
            )

    def test_merge_stage_is_tracked_in_publication_audit(self) -> None:
        stage_ids = [
            contract.stage_id for contract in benchmark_docs_audit.CORPUS_01_PUBLICATION_CONTRACTS
        ]

        self.assertIn("fastq.merge_pairs", stage_ids)

    def test_report_qc_stage_is_tracked_in_publication_audit(self) -> None:
        stage_ids = [
            contract.stage_id for contract in benchmark_docs_audit.CORPUS_01_PUBLICATION_CONTRACTS
        ]

        self.assertIn("fastq.report_qc", stage_ids)

    def test_audit_docs_rejects_missing_tool_coverage_in_sample_results(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            docs_root = repo_root / "docs" / "benchmark"
            corpus_spec = repo_root / "configs" / "runtime" / "corpora"
            corpus_spec.mkdir(parents=True)
            (corpus_spec / "corpus-01.toml").write_text(
                "\n".join(
                    [
                        'corpus_id = "corpus-01"',
                        "target_ancient_se = 1",
                        "target_ancient_pe = 1",
                        "target_modern_se = 1",
                        "target_modern_pe = 1",
                    ]
                )
                + "\n",
                encoding="utf-8",
            )

            stage_root = docs_root / "fastq.validate_reads"
            corpus_root = stage_root / "corpus-01"
            corpus_root.mkdir(parents=True)
            (stage_root / "corpus-01-method.md").write_text("# method\n", encoding="utf-8")
            (corpus_root / "summary.json").write_text(
                json.dumps(
                    {
                        "stage_id": "fastq.validate_reads",
                        "scenario_id": "validation_fairness",
                        "tools": ["fastqvalidator", "seqtk"],
                        "samples_total": 4,
                        "samples_failed": 0,
                        "cohort_counts": {
                            "ancient_pe": 1,
                            "ancient_se": 1,
                            "modern_pe": 1,
                            "modern_se": 1,
                        },
                        "tool_summary": [
                            {"tool": "fastqvalidator"},
                            {"tool": "seqtk"},
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (corpus_root / "sample_results.csv").write_text(
                "\n".join(
                    [
                        "sample_id,accession,era,layout,study_accession,size_band,tool",
                        "sample_0001,ACC1,ancient,se,PRJ1,under_100mb,fastqvalidator",
                        "sample_0002,ACC2,ancient,pe,PRJ2,under_100mb,fastqvalidator",
                        "sample_0003,ACC3,modern,se,PRJ3,under_500mb,fastqvalidator",
                        "sample_0004,ACC4,modern,pe,PRJ4,under_500mb,fastqvalidator",
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            (corpus_root / "tool_runtime_summary.csv").write_text(
                "tool\nfastqvalidator\nseqtk\n",
                encoding="utf-8",
            )
            (corpus_root / "cohort_runtime_summary.csv").write_text(
                "cohort\nancient_pe\nancient_se\nmodern_pe\nmodern_se\n",
                encoding="utf-8",
            )
            (corpus_root / "sample_runtime_outliers.csv").write_text(
                "sample_id\nsample_0001\nsample_0002\nsample_0003\nsample_0004\n",
                encoding="utf-8",
            )
            (corpus_root / "lunarc.md").write_text("# dossier\n", encoding="utf-8")

            report = benchmark_docs_audit.audit_docs(
                docs_root,
                repo_root=repo_root,
                stage_contracts=[
                    support.CorpusBenchmarkContract(
                        stage_id="fastq.validate_reads",
                        scenario_id="validation_fairness",
                        tools=["fastqvalidator", "seqtk"],
                    )
                ],
                exclusions=[],
            )
            validate_report = report["stages"][0]

            self.assertEqual(validate_report["status"], "incomplete")
            self.assertTrue(
                any(
                    issue["issue_id"] == "sample-results-tool-coverage-drift"
                    for issue in validate_report["issues"]
                )
            )

    def test_audit_docs_flags_contract_roster_drift_against_registry(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            docs_root = repo_root / "docs" / "benchmark"
            corpus_spec = repo_root / "configs" / "runtime" / "corpora"
            corpus_spec.mkdir(parents=True)
            (corpus_spec / "corpus-01.toml").write_text(
                "\n".join(
                    [
                        'corpus_id = "corpus-01"',
                        "target_ancient_se = 1",
                        "target_ancient_pe = 1",
                        "target_modern_se = 1",
                        "target_modern_pe = 1",
                    ]
                )
                + "\n",
                encoding="utf-8",
            )

            stage_root = docs_root / "fastq.trim_reads"
            corpus_root = stage_root / "corpus-01"
            corpus_root.mkdir(parents=True)
            (stage_root / "corpus-01-method.md").write_text("# method\n", encoding="utf-8")
            (corpus_root / "summary.json").write_text(
                json.dumps(
                    {
                        "stage_id": "fastq.trim_reads",
                        "scenario_id": "trim_fairness",
                        "tools": ["fastp"],
                        "samples_total": 4,
                        "samples_failed": 0,
                        "cohort_counts": {
                            "ancient_pe": 1,
                            "ancient_se": 1,
                            "modern_pe": 1,
                            "modern_se": 1,
                        },
                        "tool_summary": [
                            {"tool": "fastp"},
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (corpus_root / "sample_results.csv").write_text(
                "\n".join(
                    [
                        "sample_id,accession,era,layout,study_accession,size_band,tool",
                        "sample_0001,ACC1,ancient,se,PRJ1,under_100mb,fastp",
                        "sample_0002,ACC2,ancient,pe,PRJ2,under_100mb,fastp",
                        "sample_0003,ACC3,modern,se,PRJ3,under_500mb,fastp",
                        "sample_0004,ACC4,modern,pe,PRJ4,under_500mb,fastp",
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            (corpus_root / "tool_runtime_summary.csv").write_text(
                "tool\nfastp\n",
                encoding="utf-8",
            )
            (corpus_root / "cohort_runtime_summary.csv").write_text(
                "dimension,cohort\nera_layout,ancient_pe\nera_layout,ancient_se\nera_layout,modern_pe\nera_layout,modern_se\n",
                encoding="utf-8",
            )
            (corpus_root / "sample_runtime_outliers.csv").write_text(
                "sample_id\nsample_0001\nsample_0002\nsample_0003\nsample_0004\n",
                encoding="utf-8",
            )
            (corpus_root / "lunarc.md").write_text("# dossier\n", encoding="utf-8")

            with mock.patch.object(
                benchmark_docs_audit,
                "resolve_benchmark_tool_roster",
                return_value=(["bbduk", "fastp"], None),
            ):
                report = benchmark_docs_audit.audit_docs(
                    docs_root,
                    repo_root=repo_root,
                    stage_contracts=[
                        support.CorpusBenchmarkContract(
                            stage_id="fastq.trim_reads",
                            scenario_id="trim_fairness",
                            tools=["fastp"],
                        )
                    ],
                    exclusions=[],
                )

        trim_report = report["stages"][0]
        self.assertEqual(trim_report["status"], "incomplete")
        self.assertTrue(
            any(
                issue["issue_id"] == "contract-tool-roster-drift"
                for issue in trim_report["issues"]
            )
        )
        self.assertEqual(trim_report["expected_tool_roster"], ["bbduk", "fastp"])

    def test_audit_docs_flags_publication_subset_against_stage_toolset(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            docs_root = repo_root / "docs" / "benchmark"
            corpus_spec = repo_root / "configs" / "runtime" / "corpora"
            corpus_spec.mkdir(parents=True)
            (corpus_spec / "corpus-01.toml").write_text(
                "\n".join(
                    [
                        'corpus_id = "corpus-01"',
                        "target_ancient_se = 1",
                        "target_ancient_pe = 1",
                        "target_modern_se = 1",
                        "target_modern_pe = 1",
                    ]
                )
                + "\n",
                encoding="utf-8",
            )

            stage_root = docs_root / "fastq.trim_reads"
            corpus_root = stage_root / "corpus-01"
            corpus_root.mkdir(parents=True)
            (stage_root / "corpus-01-method.md").write_text("# method\n", encoding="utf-8")
            (corpus_root / "summary.json").write_text(
                json.dumps(
                    {
                        "stage_id": "fastq.trim_reads",
                        "scenario_id": "trim_fairness",
                        "tools": ["fastp"],
                        "samples_total": 4,
                        "samples_failed": 0,
                        "cohort_counts": {
                            "ancient_pe": 1,
                            "ancient_se": 1,
                            "modern_pe": 1,
                            "modern_se": 1,
                        },
                        "tool_summary": [
                            {"tool": "fastp"},
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (corpus_root / "sample_results.csv").write_text(
                "\n".join(
                    [
                        "sample_id,accession,era,layout,study_accession,size_band,tool",
                        "sample_0001,ACC1,ancient,se,PRJ1,under_100mb,fastp",
                        "sample_0002,ACC2,ancient,pe,PRJ2,under_100mb,fastp",
                        "sample_0003,ACC3,modern,se,PRJ3,under_500mb,fastp",
                        "sample_0004,ACC4,modern,pe,PRJ4,under_500mb,fastp",
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            (corpus_root / "tool_runtime_summary.csv").write_text(
                "tool\nfastp\n",
                encoding="utf-8",
            )
            (corpus_root / "cohort_runtime_summary.csv").write_text(
                "dimension,cohort\nera_layout,ancient_pe\nera_layout,ancient_se\nera_layout,modern_pe\nera_layout,modern_se\n",
                encoding="utf-8",
            )
            (corpus_root / "sample_runtime_outliers.csv").write_text(
                "sample_id\nsample_0001\nsample_0002\nsample_0003\nsample_0004\n",
                encoding="utf-8",
            )
            (corpus_root / "lunarc.md").write_text("# dossier\n", encoding="utf-8")

            with mock.patch.object(
                benchmark_docs_audit,
                "resolve_benchmark_tool_roster",
                return_value=(["fastp"], None),
            ):
                with mock.patch.object(
                    benchmark_docs_audit,
                    "resolve_stage_toolset",
                    return_value=(["bbduk", "fastp"], None),
                ):
                    report = benchmark_docs_audit.audit_docs(
                        docs_root,
                        repo_root=repo_root,
                        stage_contracts=[
                            support.CorpusBenchmarkContract(
                                stage_id="fastq.trim_reads",
                                scenario_id="trim_fairness",
                                tools=["fastp"],
                            )
                        ],
                        exclusions=[],
                    )

        trim_report = report["stages"][0]
        self.assertEqual(trim_report["status"], "incomplete")
        self.assertTrue(
            any(
                issue["issue_id"] == "publication-toolset-subset"
                for issue in trim_report["issues"]
            )
        )


class CorpusBenchmarkResultsAuditTests(unittest.TestCase):
    def test_result_audit_flags_contract_roster_drift_against_registry(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            docs_root = repo_root / "docs" / "benchmark" / "fastq.validate_reads" / "corpus-01"
            docs_root.mkdir(parents=True)
            local_results_root = Path(tmpdir) / "mirror"
            with mock.patch.object(support, "LOCAL_RESULTS_ROOT", local_results_root):
                run_root = (
                    local_results_root
                    / "corpus_01"
                    / "fastq.validate_reads"
                    / "lunarc"
                )
                run_root.mkdir(parents=True)
                (docs_root / "summary.json").write_text(
                    json.dumps(
                        {
                            "corpus_root": "/home/bijan/bijux/corpus_01",
                            "run_root": str(run_root),
                        }
                    )
                    + "\n",
                    encoding="utf-8",
                )
                (run_root / "run_manifest.json").write_text(
                    json.dumps(
                        {
                            "stage_id": "fastq.validate_reads",
                            "scenario_id": "validation_fairness",
                            "tools": ["fastqvalidator"],
                            "dry_run": False,
                            "sample_limit": None,
                            "samples_failed": 0,
                            "runs": [],
                        }
                    )
                    + "\n",
                    encoding="utf-8",
                )

                with mock.patch.object(
                    published_results_audit,
                    "resolve_benchmark_tool_roster",
                    return_value=(["fastqvalidator", "seqtk"], None),
                ):
                    report = published_results_audit.audit_stage(
                        repo_root,
                        "fastq.validate_reads",
                        "validation_fairness",
                        ["fastqvalidator"],
                    )

        self.assertEqual(report["status"], "incomplete")
        self.assertTrue(
            any(
                issue["issue_id"] == "contract-tool-roster-drift"
                for issue in report["issues"]
            )
        )

    def test_result_audit_flags_partial_tool_roster_in_sample_reports(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            docs_root = repo_root / "docs" / "benchmark" / "fastq.validate_reads" / "corpus-01"
            docs_root.mkdir(parents=True)
            local_results_root = Path(tmpdir) / "mirror"
            with mock.patch.object(support, "LOCAL_RESULTS_ROOT", local_results_root):
                run_root = (
                    local_results_root
                    / "corpus_01"
                    / "fastq.validate_reads"
                    / "lunarc"
                )
                (docs_root / "summary.json").write_text(
                    json.dumps(
                        {
                            "corpus_root": "/home/bijan/bijux/corpus_01",
                            "run_root": str(run_root),
                        }
                    )
                    + "\n",
                    encoding="utf-8",
                )
                sample_report = run_root / "bench" / "validate_reads" / "sample_0001" / "report.json"
                sample_report.parent.mkdir(parents=True)
                sample_report.write_text(
                    json.dumps(
                        {
                            "records": [
                                {
                                    "context": {
                                        "tool": "fastqvalidator",
                                    }
                                }
                            ]
                        }
                    )
                    + "\n",
                    encoding="utf-8",
                )
                (run_root / "run_manifest.json").write_text(
                    json.dumps(
                        {
                            "stage_id": "fastq.validate_reads",
                            "scenario_id": "validation_fairness",
                            "tools": ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
                            "dry_run": False,
                            "sample_limit": None,
                            "samples_failed": 0,
                            "runs": [
                                {
                                    "sample_id": "sample_0001",
                                    "report_json": str(sample_report),
                                }
                            ],
                        }
                    )
                    + "\n",
                    encoding="utf-8",
                )

                report = published_results_audit.audit_stage(
                    repo_root,
                    "fastq.validate_reads",
                    "validation_fairness",
                    ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
                )

            self.assertEqual(report["status"], "incomplete")
            self.assertTrue(
                any(
                    issue["issue_id"] == "report-tool-roster-drift"
                    for issue in report["issues"]
                )
            )

    def test_result_audit_accepts_existing_reported_run_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            docs_root = repo_root / "docs" / "benchmark" / "fastq.validate_reads" / "corpus-01"
            docs_root.mkdir(parents=True)
            run_root = repo_root / "mirror" / "corpus_01" / "fastq.validate_reads" / "lunarc"
            sample_report = run_root / "bench" / "validate_reads" / "sample_0001" / "report.json"
            sample_report.parent.mkdir(parents=True)
            (docs_root / "summary.json").write_text(
                json.dumps(
                    {
                        "corpus_root": "/home/bijan/bijux/corpus_01",
                        "run_root": str(run_root),
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {"context": {"tool": "fastqvalidator"}},
                            {"context": {"tool": "fastqc"}},
                            {"context": {"tool": "fastq_scan"}},
                            {"context": {"tool": "fqtools"}},
                            {"context": {"tool": "seqtk"}},
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "stage_id": "fastq.validate_reads",
                        "scenario_id": "validation_fairness",
                        "tools": ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
                        "dry_run": False,
                        "sample_limit": None,
                        "samples_failed": 0,
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "report_json": str(sample_report),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            report = published_results_audit.audit_stage(
                repo_root,
                "fastq.validate_reads",
                "validation_fairness",
                ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
            )

        self.assertFalse(
            any(
                issue["issue_id"] == "summary-run-root-drift"
                for issue in report["issues"]
            )
        )


class ValidateReadsReportingTests(unittest.TestCase):
    def test_validate_reads_summary_preserves_contract_identity_and_corpus_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            run_root = repo_root / "results" / "corpus_01" / "fastq.validate_reads" / "lunarc"
            docs_root = repo_root / "docs" / "benchmark" / "fastq.validate_reads" / "corpus-01"
            sample_report = (
                run_root / "bench" / "validate_reads" / "sample_0001" / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {"tool": "fastq_scan"},
                                "execution": {"runtime_s": 1.2, "exit_code": 0},
                                "metrics": {"reads_invalid": 0, "reads_total": 10},
                            }
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "platform": "lunarc-apptainer",
                        "corpus_root": "/home/bijan/bijux/corpus_01",
                        "stage_id": "fastq.validate_reads",
                        "scenario_id": "validation_fairness",
                        "samples_total": 1,
                        "samples_failed": 0,
                        "tools": ["fastq_scan"],
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "layout": "se",
                                "report_json": str(sample_report),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            original_argv = sys.argv
            try:
                sys.argv = [
                    "render_fastq_validate_reads_corpus_01_report.py",
                    "--repo-root",
                    str(repo_root),
                    "--corpus-root",
                    "/home/bijan/lu2024-12-24/.cache/corpus_01",
                    "--run-root",
                    str(run_root),
                    "--docs-root",
                    str(docs_root.relative_to(repo_root)),
                ]
                with mock.patch.object(
                    validate_reads_report,
                    "load_corpus_spec",
                    return_value={"corpus_id": "corpus-01"},
                ), mock.patch.object(
                    validate_reads_report,
                    "resolve_corpus_metadata",
                    return_value={
                        "sample_0001": {
                            "accession": "ACC1",
                            "era": "ancient",
                            "layout": "se",
                            "study_accession": "PRJ1",
                            "size_band": "under_100mb",
                        }
                    },
                ):
                    self.assertEqual(validate_reads_report.main(), 0)
            finally:
                sys.argv = original_argv

            summary = json.loads((docs_root / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(summary["stage_id"], "fastq.validate_reads")
            self.assertEqual(summary["scenario_id"], "validation_fairness")
            self.assertEqual(
                summary["corpus_root"],
                "/home/bijan/lu2024-12-24/.cache/corpus_01",
            )


class TrimPolygReportingTests(unittest.TestCase):
    def test_trim_polyg_summary_tracks_runtime_and_retention(self) -> None:
        records = [
            {
                "tool": "fastp",
                "runtime_s": "0.8",
                "exit_code": "0",
                "base_retention": "0.97",
                "bases_trimmed_polyg": "24",
                "mean_q_delta": "0.30",
            },
            {
                "tool": "fastp",
                "runtime_s": "1.0",
                "exit_code": "0",
                "base_retention": "0.95",
                "bases_trimmed_polyg": "28",
                "mean_q_delta": "0.40",
            },
            {
                "tool": "bbduk",
                "runtime_s": "1.6",
                "exit_code": "0",
                "base_retention": "0.96",
                "bases_trimmed_polyg": "21",
                "mean_q_delta": "0.20",
            },
            {
                "tool": "bbduk",
                "runtime_s": "1.8",
                "exit_code": "0",
                "base_retention": "0.94",
                "bases_trimmed_polyg": "25",
                "mean_q_delta": "0.10",
            },
        ]

        summary_rows = trim_polyg_briefing.tool_runtime_summary(records)
        by_tool = {row["tool"]: row for row in summary_rows}

        self.assertAlmostEqual(by_tool["fastp"]["median_runtime_s"], 0.9)
        self.assertAlmostEqual(by_tool["fastp"]["median_base_retention"], 0.96)
        self.assertAlmostEqual(by_tool["fastp"]["mean_bases_trimmed_polyg"], 26.0)
        self.assertGreater(
            by_tool["bbduk"]["slowdown_vs_fastest_median"],
            by_tool["fastp"]["slowdown_vs_fastest_median"],
        )

    def test_trim_polyg_outliers_capture_slowest_and_strongest_trim_tools(self) -> None:
        rows = [
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "modern",
                "layout": "pe",
                "size_band": "under_500mb",
                "study_accession": "PRJ1",
                "tool": "fastp",
                "runtime_s": "5.0",
                "bases_trimmed_polyg": "40",
            },
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "modern",
                "layout": "pe",
                "size_band": "under_500mb",
                "study_accession": "PRJ1",
                "tool": "bbduk",
                "runtime_s": "6.5",
                "bases_trimmed_polyg": "22",
            },
        ]

        outliers = trim_polyg_briefing.sample_runtime_outliers(rows)

        self.assertEqual(outliers[0]["slowest_tool"], "bbduk")
        self.assertEqual(outliers[0]["most_trimming_tool"], "fastp")
        self.assertAlmostEqual(outliers[0]["most_trimmed_bases"], 40.0)

    def test_trim_polyg_markdown_mentions_polyx_preset(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/results/corpus_01/fastq.trim_polyg_tails/lunarc",
            "samples_total": 20,
            "samples_failed": 0,
            "tools": ["fastp", "bbduk"],
            "polyx_preset": "illumina_twocolor",
            "min_polyg_run": 10,
            "era_counts": {"ancient": 10, "modern": 10},
            "layout_counts": {"se": 10, "pe": 10},
            "cohort_counts": {"ancient_pe": 5, "ancient_se": 5, "modern_pe": 5, "modern_se": 5},
            "headline": {
                "fastest_tool": "fastp",
                "fastest_runtime_s": 0.9,
                "largest_polyg_trim_tool": "fastp",
                "largest_polyg_trim_bases": 26.0,
                "best_base_retention_tool": "fastp",
                "best_base_retention": 0.96,
            },
            "tool_summary": [
                {
                    "tool": "fastp",
                    "records": 20,
                    "pass_rate": 1.0,
                    "median_runtime_s": 0.9,
                    "median_base_retention": 0.96,
                    "mean_bases_trimmed_polyg": 26.0,
                    "mean_q_delta": 0.35,
                }
            ],
        }

        markdown = trim_polyg_report.render_markdown(summary)

        self.assertIn("PolyX preset: `illumina_twocolor`", markdown)
        self.assertIn("Mean bases trimmed", markdown)

    def test_trim_polyg_report_contract_rejects_mixed_preset_rows(self) -> None:
        run_manifest = {
            "tools": ["fastp", "bbduk"],
            "polyx_preset": "illumina_twocolor",
            "min_polyg_run": 10,
            "trim_polyg": True,
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "fastp",
                "raw_backend_report_format": "fastp_json",
                "polyx_preset": "illumina_twocolor",
                "min_polyg_run": 10,
                "trim_polyg": True,
            },
            {
                "sample_id": "sample_0001",
                "tool": "bbduk",
                "raw_backend_report_format": "bbduk_stats",
                "polyx_preset": "wrong_preset",
                "min_polyg_run": 10,
                "trim_polyg": True,
            },
        ]

        with self.assertRaises(SystemExit):
            trim_polyg_report.validate_trim_polyg_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
            )

    def test_trim_polyg_summary_preserves_configured_corpus_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            run_root = (
                repo_root / "results" / "corpus_01" / "fastq.trim_polyg_tails" / "lunarc"
            )
            docs_root = (
                repo_root / "docs" / "benchmark" / "fastq.trim_polyg_tails" / "corpus-01"
            )
            sample_report = (
                run_root / "bench" / "trim_polyg_tails" / "sample_0001" / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {
                                    "tool": "fastp",
                                    "parameters": {
                                        "polyx_preset": "illumina_twocolor",
                                        "trim_polyg": True,
                                        "min_polyg_run": 10,
                                        "raw_backend_report_format": "fastp_json",
                                    },
                                },
                                "execution": {"runtime_s": 1.5, "exit_code": 0},
                                "metrics": {
                                    "metrics": {
                                        "reads_in": 100,
                                    "reads_out": 95,
                                    "bases_in": 1000,
                                    "bases_out": 960,
                                    "bases_trimmed_polyg": 40,
                                    "raw_backend_report_format": "fastp_json",
                                    "delta_metrics": {
                                        "base_retention": 0.96,
                                        "read_retention": 0.95,
                                        "mean_q_delta": 0.0,
                                        },
                                        "polyx_preset": "illumina_twocolor",
                                        "trim_polyg": True,
                                        "min_polyg_run": 10,
                                    }
                                },
                            }
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "platform": "lunarc-apptainer",
                        "corpus_root": "/home/bijan/bijux/corpus_01",
                        "samples_total": 1,
                        "samples_failed": 0,
                        "tools": ["fastp"],
                        "polyx_preset": "illumina_twocolor",
                        "trim_polyg": True,
                        "min_polyg_run": 10,
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "layout": "se",
                                "report_json": str(sample_report),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            original_argv = sys.argv
            try:
                sys.argv = [
                    "render_fastq_trim_polyg_tails_corpus_01_report.py",
                    "--repo-root",
                    str(repo_root),
                    "--corpus-root",
                    "/home/bijan/lu2024-12-24/.cache/corpus_01",
                    "--run-root",
                    str(run_root),
                    "--docs-root",
                    str(docs_root.relative_to(repo_root)),
                ]
                with mock.patch.object(
                    trim_polyg_report,
                    "load_corpus_spec",
                    return_value={"corpus_id": "corpus-01"},
                ), mock.patch.object(
                    trim_polyg_report,
                    "resolve_corpus_metadata",
                    return_value={
                        "sample_0001": {
                            "accession": "ACC1",
                            "era": "modern",
                            "layout": "se",
                            "study_accession": "PRJ1",
                            "size_band": "under_100mb",
                        }
                    },
                ):
                    self.assertEqual(trim_polyg_report.main(), 0)
            finally:
                sys.argv = original_argv

            summary = json.loads((docs_root / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(
                summary["corpus_root"],
                "/home/bijan/lu2024-12-24/.cache/corpus_01",
            )


class ReportQcReportingTests(unittest.TestCase):
    def test_report_qc_tool_summary_tracks_multiqc_and_governed_inputs(self) -> None:
        rows = [
            {
                "tool": "multiqc",
                "runtime_s": "4.2",
                "exit_code": "0",
                "multiqc_module_count": "8",
                "multiqc_sample_count": "1",
                "governed_qc_input_count": "6",
                "contamination_rate": "0.012",
                "mean_q": "34.1",
            },
            {
                "tool": "multiqc",
                "runtime_s": "4.8",
                "exit_code": "0",
                "multiqc_module_count": "9",
                "multiqc_sample_count": "1",
                "governed_qc_input_count": "6",
                "contamination_rate": "0.010",
                "mean_q": "34.6",
            },
        ]

        summary_rows = report_qc_briefing.tool_runtime_summary(rows)

        self.assertEqual(len(summary_rows), 1)
        self.assertAlmostEqual(summary_rows[0]["median_runtime_s"], 4.5)
        self.assertAlmostEqual(summary_rows[0]["median_multiqc_module_count"], 8.5)
        self.assertAlmostEqual(summary_rows[0]["median_governed_qc_input_count"], 6.0)
        self.assertAlmostEqual(summary_rows[0]["median_mean_q"], 34.35)

    def test_report_qc_markdown_mentions_aggregation_contract(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/results/corpus_01/fastq.report_qc/lunarc",
            "scenario_id": "qc_aggregation_fairness",
            "samples_total": 20,
            "samples_failed": 0,
            "tools": ["multiqc"],
            "aggregation_engine": "multiqc",
            "aggregation_scope": "governed_qc_artifacts",
            "report_only": True,
            "mutates_fastq": False,
            "may_change_read_count": False,
            "governed_contributor_stage_ids": [
                "fastq.validate_reads",
                "fastq.detect_adapters",
                "fastq.profile_reads",
                "fastq.profile_read_lengths",
            ],
            "era_counts": {"ancient": 10, "modern": 10},
            "layout_counts": {"se": 10, "pe": 10},
            "cohort_counts": {"ancient_pe": 5, "ancient_se": 5, "modern_pe": 5, "modern_se": 5},
            "headline": {
                "fastest_tool": "multiqc",
                "fastest_runtime_s": 4.5,
                "largest_multiqc_module_tool": "multiqc",
                "largest_multiqc_module_count": 8.5,
                "highest_governed_input_tool": "multiqc",
                "highest_governed_input_count": 6.0,
            },
            "tool_summary": [
                {
                    "tool": "multiqc",
                    "records": 20,
                    "pass_rate": 1.0,
                    "median_runtime_s": 4.5,
                    "median_multiqc_module_count": 8.5,
                    "median_multiqc_sample_count": 1.0,
                    "median_governed_qc_input_count": 6.0,
                    "median_contamination_rate": 0.011,
                    "median_mean_q": 34.35,
                }
            ],
        }

        markdown = report_qc_report.render_markdown(summary)

        self.assertIn("aggregation_engine: `multiqc`", markdown)
        self.assertIn("Governed contributor stages", markdown)
        self.assertIn("Median governed inputs", markdown)

    def test_report_qc_report_contract_rejects_mismatched_governed_input_count(self) -> None:
        run_manifest = {
            "tools": ["multiqc"],
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "multiqc",
                "reads_in": 100,
                "reads_out": 100,
                "bases_in": 1000,
                "bases_out": 1000,
                "governed_qc_input_count": 5,
                "expected_governed_qc_input_count": 6,
                "mean_q": 34.0,
                "contamination_rate": 0.01,
                "exit_code": 1,
            }
        ]

        with self.assertRaises(SystemExit):
            report_qc_report.validate_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0001"],
            )

    def test_report_qc_enriches_missing_multiqc_artifacts_from_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            data_dir = Path(tmpdir) / "multiqc_data"
            report_data_dir = data_dir / "multiqc_report_data"
            report_data_dir.mkdir(parents=True)
            (data_dir / "multiqc_report.html").write_text("<html></html>\n", encoding="utf-8")
            (report_data_dir / "multiqc_data.json").write_text(
                json.dumps(
                    {
                        "report_general_stats_data": [
                            {"sample_a": {"total_sequences": 10}, "sample_b": {"total_sequences": 12}}
                        ],
                        "report_plot_data": {
                            "general_stats_table": {},
                            "fastqc_sequence_counts_plot": {},
                            "fastqc_adapter_content_plot": {},
                        },
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            row = report_qc_report.enrich_multiqc_artifacts(
                {
                    "multiqc_data": str(data_dir),
                    "multiqc_report": "",
                    "multiqc_sample_count": None,
                    "multiqc_module_count": None,
                }
            )

            self.assertEqual(row["multiqc_sample_count"], 2)
            self.assertEqual(row["multiqc_module_count"], 3)
            self.assertEqual(row["multiqc_report"], str(data_dir / "multiqc_report.html"))

    def test_report_qc_summary_preserves_configured_corpus_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            run_root = repo_root / "results" / "corpus_01" / "fastq.report_qc" / "lunarc"
            docs_root = repo_root / "docs" / "benchmark" / "fastq.report_qc" / "corpus-01"
            sample_root = run_root / "bench" / "report_qc" / "sample_0001"
            tool_root = sample_root / "tools" / "multiqc"
            raw_fastqc_dir = sample_root / "tools" / "fastqc" / "fastqc"
            raw_fastqc_dir.mkdir(parents=True)
            multiqc_data = tool_root / "multiqc_data"
            report_data_dir = multiqc_data / "multiqc_report_data"
            report_data_dir.mkdir(parents=True)
            (multiqc_data / "multiqc_report.html").write_text("<html></html>\n", encoding="utf-8")
            (report_data_dir / "multiqc_data.json").write_text(
                json.dumps(
                    {
                        "report_general_stats_data": [{"sample_0001": {"total_sequences": 100}}],
                        "report_plot_data": {
                            "general_stats_table": {},
                            "fastqc_sequence_counts_plot": {},
                            "fastqc_adapter_content_plot": {},
                        },
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            governed_manifest = sample_root / "governed_qc_inputs_manifest.json"
            governed_manifest.write_text(json.dumps({"inputs": []}) + "\n", encoding="utf-8")
            sample_report = sample_root / "report.json"
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {"tool": "multiqc"},
                                "execution": {"runtime_s": 1.5, "exit_code": 0},
                                "metrics": {
                                    "metrics": {
                                        "reads_in": 100,
                                        "reads_out": 100,
                                        "bases_in": 1000,
                                        "bases_out": 1000,
                                        "pairs_in": 50,
                                        "pairs_out": 50,
                                        "mean_q": 34.0,
                                        "contamination_rate": 0.01,
                                        "multiqc_sample_count": 1,
                                        "multiqc_module_count": 3,
                                        "governed_qc_input_count": 6,
                                        "governed_qc_lineage_hash": "sha256:lineage",
                                        "raw_fastqc_dir": str(raw_fastqc_dir),
                                        "multiqc_report": str(multiqc_data / "multiqc_report.html"),
                                        "multiqc_data": str(multiqc_data),
                                    }
                                },
                            }
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (tool_root / "report_qc_report.json").write_text(
                json.dumps({"status": "ok"}) + "\n",
                encoding="utf-8",
            )
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "platform": "lunarc-apptainer",
                        "corpus_root": "/home/bijan/bijux/corpus_01",
                        "stage_id": "fastq.report_qc",
                        "scenario_id": "qc_aggregation_fairness",
                        "tool_kind": "benchmark",
                        "tools": ["multiqc"],
                        "aggregation_engine": "multiqc",
                        "aggregation_scope": "governed_qc_artifacts",
                        "report_only": True,
                        "mutates_fastq": False,
                        "may_change_read_count": False,
                        "samples_total": 1,
                        "samples_failed": 0,
                        "governed_contributor_stage_ids": [
                            "fastq.validate_reads",
                            "fastq.detect_adapters",
                            "fastq.profile_reads",
                            "fastq.profile_read_lengths",
                        ],
                        "governed_contributor_tool_ids": [
                            "fastqvalidator",
                            "fastqc",
                            "seqkit_stats",
                        ],
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "layout": "pe",
                                "report_json": str(sample_report),
                                "governed_qc_input_count": 6,
                                "governed_qc_manifest": str(governed_manifest),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            original_argv = sys.argv
            try:
                sys.argv = [
                    "render_fastq_report_qc_corpus_01_report.py",
                    "--repo-root",
                    str(repo_root),
                    "--corpus-root",
                    "/home/bijan/lu2024-12-24/.cache/corpus_01",
                    "--run-root",
                    str(run_root),
                    "--docs-root",
                    str(docs_root.relative_to(repo_root)),
                ]
                with mock.patch.object(
                    report_qc_report,
                    "load_corpus_spec",
                    return_value={"corpus_id": "corpus-01"},
                ), mock.patch.object(
                    report_qc_report,
                    "resolve_corpus_metadata",
                    return_value={
                        "sample_0001": {
                            "accession": "ACC1",
                            "era": "modern",
                            "layout": "pe",
                            "study_accession": "PRJ1",
                            "size_band": "under_100mb",
                        }
                    },
                ):
                    self.assertEqual(report_qc_report.main(), 0)
            finally:
                sys.argv = original_argv

            summary = json.loads((docs_root / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(
                summary["corpus_root"],
                "/home/bijan/lu2024-12-24/.cache/corpus_01",
            )


class TrimReadsReportingTests(unittest.TestCase):
    def test_trim_reads_summary_tracks_runtime_and_retention(self) -> None:
        rows = [
            {
                "tool": "fastp",
                "runtime_s": "0.8",
                "exit_code": "0",
                "base_retention": "0.97",
                "read_retention": "0.96",
                "mean_q_delta": "0.30",
            },
            {
                "tool": "fastp",
                "runtime_s": "1.0",
                "exit_code": "0",
                "base_retention": "0.95",
                "read_retention": "0.94",
                "mean_q_delta": "0.40",
            },
            {
                "tool": "bbduk",
                "runtime_s": "1.6",
                "exit_code": "0",
                "base_retention": "0.96",
                "read_retention": "0.93",
                "mean_q_delta": "0.20",
            },
            {
                "tool": "bbduk",
                "runtime_s": "1.8",
                "exit_code": "0",
                "base_retention": "0.94",
                "read_retention": "0.91",
                "mean_q_delta": "0.10",
            },
        ]

        summary_rows = trim_reads_briefing.tool_runtime_summary(rows)
        by_tool = {row["tool"]: row for row in summary_rows}

        self.assertAlmostEqual(by_tool["fastp"]["median_runtime_s"], 0.9)
        self.assertAlmostEqual(by_tool["fastp"]["median_base_retention"], 0.96)
        self.assertAlmostEqual(by_tool["fastp"]["median_read_retention"], 0.95)
        self.assertGreater(
            by_tool["bbduk"]["slowdown_vs_fastest_median"],
            by_tool["fastp"]["slowdown_vs_fastest_median"],
        )

    def test_trim_reads_outliers_capture_slowest_and_lowest_retention_tools(self) -> None:
        rows = [
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "modern",
                "layout": "pe",
                "size_band": "under_500mb",
                "study_accession": "PRJ1",
                "tool": "fastp",
                "runtime_s": "5.0",
                "base_retention": "0.98",
            },
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "modern",
                "layout": "pe",
                "size_band": "under_500mb",
                "study_accession": "PRJ1",
                "tool": "bbduk",
                "runtime_s": "6.5",
                "base_retention": "0.91",
            },
        ]

        outliers = trim_reads_briefing.sample_runtime_outliers(rows)

        self.assertEqual(outliers[0]["slowest_tool"], "bbduk")
        self.assertEqual(outliers[0]["lowest_retention_tool"], "bbduk")
        self.assertAlmostEqual(outliers[0]["lowest_base_retention"], 0.91)

    def test_trim_reads_markdown_mentions_trim_policy_bundle(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/results/corpus_01/fastq.trim_reads/lunarc",
            "scenario_id": "trim_fairness",
            "samples_total": 20,
            "samples_failed": 0,
            "tools": ["fastp", "bbduk"],
            "min_length": 30,
            "quality_cutoff": None,
            "n_policy": "retain",
            "adapter_policy": "none",
            "polyx_policy": "none",
            "contaminant_policy": "none",
            "era_counts": {"ancient": 10, "modern": 10},
            "layout_counts": {"se": 10, "pe": 10},
            "cohort_counts": {"ancient_pe": 5, "ancient_se": 5, "modern_pe": 5, "modern_se": 5},
            "headline": {
                "fastest_tool": "fastp",
                "fastest_runtime_s": 0.9,
                "best_base_retention_tool": "fastp",
                "best_base_retention": 0.96,
                "best_read_retention_tool": "fastp",
                "best_read_retention": 0.95,
                "best_q_gain_tool": "fastp",
                "best_q_gain": 0.35,
            },
            "tool_summary": [
                {
                    "tool": "fastp",
                    "records": 20,
                    "pass_rate": 1.0,
                    "median_runtime_s": 0.9,
                    "median_base_retention": 0.96,
                    "median_read_retention": 0.95,
                    "mean_q_delta": 0.35,
                }
            ],
        }

        markdown = trim_reads_report.render_markdown(summary)

        self.assertIn("adapter_policy: `none`", markdown)
        self.assertIn("Median read retention", markdown)

    def test_trim_reads_briefing_uses_dynamic_tool_and_sample_counts(self) -> None:
        summary = {
            "platform": "lunarc-apptainer",
            "stage_id": "fastq.trim_reads",
            "scenario_id": "trim_fairness",
            "samples_total": 2,
            "tools": ["fastp", "bbduk"],
            "min_length": 30,
            "quality_cutoff": None,
            "n_policy": "retain",
            "adapter_policy": "none",
            "polyx_policy": "none",
            "contaminant_policy": "none",
        }
        rows = [
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "modern",
                "layout": "pe",
                "size_band": "under_100mb",
                "study_accession": "PRJ1",
                "tool": "fastp",
                "runtime_s": "1.0",
                "exit_code": "0",
                "base_retention": "0.95",
                "read_retention": "0.94",
                "mean_q_delta": "0.2",
            },
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "modern",
                "layout": "pe",
                "size_band": "under_100mb",
                "study_accession": "PRJ1",
                "tool": "bbduk",
                "runtime_s": "2.0",
                "exit_code": "0",
                "base_retention": "0.96",
                "read_retention": "0.95",
                "mean_q_delta": "0.1",
            },
        ]
        runtime_rows = trim_reads_briefing.tool_runtime_summary(rows)
        cohort_rows = [
            {
                "tool": "fastp",
                "dimension": "era_layout",
                "cohort": "modern_pe",
                "median_runtime_s": 1.0,
            },
            {
                "tool": "fastp",
                "dimension": "era_layout",
                "cohort": "ancient_se",
                "median_runtime_s": 0.8,
            },
        ]
        outliers = trim_reads_briefing.sample_runtime_outliers(rows)

        markdown = trim_reads_briefing.render_markdown(summary, rows, runtime_rows, cohort_rows, outliers)

        self.assertIn("`2` governed trim backends were benchmarked across `2` human samples", markdown)

    def test_trim_reads_report_contract_rejects_policy_drift(self) -> None:
        run_manifest = {
            "tools": ["fastp", "bbduk"],
            "min_length": 30,
            "quality_cutoff": None,
            "n_policy": "retain",
            "adapter_policy": "none",
            "polyx_policy": "none",
            "contaminant_policy": "none",
            "adapter_bank_preset": None,
            "polyx_preset": None,
            "contaminant_preset": None,
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "fastp",
                "raw_backend_report_format": "fastp_json",
                "min_length": 30,
                "quality_cutoff": None,
                "n_policy": "retain",
                "adapter_policy": "none",
                "polyx_policy": "none",
                "contaminant_policy": "none",
                "adapter_bank_preset": None,
                "polyx_preset": None,
                "contaminant_preset": None,
            },
            {
                "sample_id": "sample_0001",
                "tool": "bbduk",
                "raw_backend_report_format": "bbduk_stats",
                "min_length": 20,
                "quality_cutoff": None,
                "n_policy": "retain",
                "adapter_policy": "none",
                "polyx_policy": "none",
                "contaminant_policy": "none",
                "adapter_bank_preset": None,
                "polyx_preset": None,
                "contaminant_preset": None,
            },
        ]

        with self.assertRaises(SystemExit):
            trim_reads_report.validate_trim_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
            )

    def test_trim_reads_report_rejects_dry_run_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            trim_reads_report.validate_trim_run_manifest_contract(
                {
                    "stage_id": "fastq.trim_reads",
                    "scenario_id": "trim_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": True,
                }
            )

    def test_trim_reads_summary_preserves_configured_corpus_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            run_root = repo_root / "results" / "corpus_01" / "fastq.trim_reads" / "lunarc"
            docs_root = (
                repo_root / "docs" / "benchmark" / "fastq.trim_reads" / "corpus-01"
            )
            sample_report = (
                run_root / "bench" / "trim_reads" / "sample_0001" / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            trim_metrics = {
                "reads_in": 100,
                "reads_out": 95,
                "bases_in": 1000,
                "bases_out": 940,
                "delta_metrics": {
                    "base_retention": 0.94,
                    "read_retention": 0.95,
                    "mean_q_delta": 0.1,
                },
                "min_length": 30,
                "quality_cutoff": None,
                "n_policy": "retain",
                "adapter_policy": "none",
                "polyx_policy": "none",
                "contaminant_policy": "none",
                "adapter_preset": None,
                "polyx_preset": None,
                "contaminant_preset": None,
            }
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {"tool": "fastp"},
                                "execution": {"runtime_s": 1.5, "exit_code": 0},
                                "metrics": {
                                    "metrics": {
                                        **trim_metrics,
                                        "raw_backend_report_format": "fastp_json",
                                    }
                                },
                            },
                            {
                                "context": {"tool": "bbduk"},
                                "execution": {"runtime_s": 1.7, "exit_code": 0},
                                "metrics": {
                                    "metrics": {
                                        **trim_metrics,
                                        "raw_backend_report_format": "bbduk_stats",
                                    }
                                },
                            },
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "platform": "lunarc-apptainer",
                        "corpus_root": "/home/bijan/bijux/corpus_01",
                        "stage_id": "fastq.trim_reads",
                        "scenario_id": "trim_fairness",
                        "tool_kind": "benchmark",
                        "samples_total": 1,
                        "samples_failed": 0,
                        "tools": ["fastp", "bbduk"],
                        "min_length": 30,
                        "quality_cutoff": None,
                        "n_policy": "retain",
                        "adapter_policy": "none",
                        "polyx_policy": "none",
                        "contaminant_policy": "none",
                        "adapter_bank_preset": None,
                        "polyx_preset": None,
                        "contaminant_preset": None,
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "layout": "se",
                                "report_json": str(sample_report),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            original_argv = sys.argv
            try:
                sys.argv = [
                    "render_fastq_trim_reads_corpus_01_report.py",
                    "--repo-root",
                    str(repo_root),
                    "--corpus-root",
                    "/home/bijan/lu2024-12-24/.cache/corpus_01",
                    "--run-root",
                    str(run_root),
                    "--docs-root",
                    str(docs_root.relative_to(repo_root)),
                ]
                with mock.patch.object(
                    trim_reads_report,
                    "load_corpus_spec",
                    return_value={"corpus_id": "corpus-01"},
                ), mock.patch.object(
                    trim_reads_report,
                    "load_sample_metadata",
                    return_value={
                        "sample_0001": {
                            "accession": "ACC1",
                            "era": "modern",
                            "layout": "se",
                            "study_accession": "PRJ1",
                            "size_band": "under_100mb",
                        }
                    },
                ):
                    self.assertEqual(trim_reads_report.main(), 0)
            finally:
                sys.argv = original_argv

            summary = json.loads((docs_root / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(
                summary["corpus_root"],
                "/home/bijan/lu2024-12-24/.cache/corpus_01",
            )

    def test_trim_reads_runner_parse_args_supports_sample_jobs(self) -> None:
        original_argv = sys.argv
        try:
            sys.argv = ["run_fastq_trim_reads_corpus_01.py", "--sample-jobs", "3"]
            args = trim_reads_runner.parse_args()
        finally:
            sys.argv = original_argv

        self.assertEqual(args.sample_jobs, 3)

    def test_trim_reads_runner_resume_requires_successful_sample_report(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            report_path = Path(tmpdir) / "report.json"
            report_path.write_text(
                json.dumps(
                    {
                        "failures": [],
                        "gate": {"passes": True},
                        "records": [{"context": {"tool": "fastp"}}],
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            self.assertTrue(trim_reads_runner.sample_report_is_resume_ready(report_path))

            report_path.write_text(
                json.dumps(
                    {
                        "failures": [
                            {
                                "kind": "tool_exit",
                                "reason": "tool `alientrimmer` failed with status 2",
                            }
                        ],
                        "gate": {"passes": False},
                        "records": [],
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            self.assertFalse(
                trim_reads_runner.sample_report_is_resume_ready(report_path)
            )

    def test_trim_reads_runner_reruns_stale_resume_report(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir) / "repo"
            repo_root.mkdir()
            cache_root = Path(tmpdir) / ".cache"
            corpus_root = cache_root / "corpus_01"
            normalized_root = corpus_root / "normalized"
            normalized_root.mkdir(parents=True)
            r1_path = normalized_root / "sample_0001_R1.fastq.gz"
            r1_path.write_text("reads", encoding="utf-8")
            out_root = cache_root / "bijux-dna-results"
            stale_sample_root = out_root / "bench" / "trim_reads" / "sample_0001"
            stale_sample_root.mkdir(parents=True)
            stale_marker = stale_sample_root / "stale.marker"
            stale_marker.write_text("old", encoding="utf-8")
            stale_report = stale_sample_root / "report.json"
            stale_report.write_text(
                json.dumps(
                    {
                        "failures": [
                            {
                                "kind": "tool_exit",
                                "reason": "tool `alientrimmer` failed with status 2",
                            }
                        ],
                        "gate": {"passes": False},
                        "records": [],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            def fake_run(
                command: list[str],
                cwd: Path,
                check: bool = False,
                env: dict[str, str] | None = None,
            ):
                self.assertEqual(Path(cwd).resolve(), repo_root.resolve())
                self.assertFalse(stale_marker.exists())
                self.assertIsNotNone(env)
                self.assertEqual(Path(env["BIJUX_CACHE_ROOT"]).resolve(), cache_root.resolve())
                self.assertEqual(
                    Path(env["BIJUX_HPC_ROOT"]).resolve(),
                    Path(tmpdir).resolve(),
                )
                fresh_report = (
                    out_root / "bench" / "trim_reads" / "sample_0001" / "report.json"
                )
                fresh_report.parent.mkdir(parents=True, exist_ok=True)
                fresh_report.write_text(
                    json.dumps(
                        {
                            "failures": [],
                            "gate": {"passes": True},
                            "records": [{"context": {"tool": "fastp"}}],
                            "semantic_metrics": [],
                        }
                    )
                    + "\n",
                    encoding="utf-8",
                )
                return mock.Mock(returncode=0)

            argv = [
                "run_fastq_trim_reads_corpus_01.py",
                "--repo-root",
                str(repo_root),
                "--corpus-root",
                str(corpus_root),
                "--out-root",
                str(out_root),
            ]
            with mock.patch.object(sys, "argv", argv):
                with mock.patch.object(
                    trim_reads_runner,
                    "load_corpus_spec",
                    return_value={
                        "corpus_id": "corpus-01",
                        "preferred_root": str(corpus_root),
                    },
                ):
                    with mock.patch.object(
                        trim_reads_runner,
                        "discover_normalized_samples",
                        return_value=[
                            {
                                "sample_id": "sample_0001",
                                "r1": r1_path,
                                "r2": None,
                                "layout": "se",
                            }
                        ],
                    ):
                        with mock.patch.object(
                            trim_reads_runner,
                            "validate_corpus_contract",
                        ):
                            with mock.patch.object(
                                trim_reads_runner,
                                "require_canonical_tool_roster",
                                return_value=["fastp"],
                            ):
                                with mock.patch.object(
                                    trim_reads_runner.subprocess,
                                    "run",
                                    side_effect=fake_run,
                                ) as run_mock:
                                    exit_code = trim_reads_runner.main()

            self.assertEqual(exit_code, 0)
            self.assertEqual(run_mock.call_count, 1)
            self.assertFalse(stale_marker.exists())
            manifest = json.loads(
                (out_root / "run_manifest.json").read_text(encoding="utf-8")
            )
            self.assertEqual(manifest["samples_failed"], 0)
            self.assertEqual(manifest["runs"][0]["status"], "completed")

    def test_trim_reads_runner_resets_orphaned_sample_payload_before_resume(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir) / "repo"
            repo_root.mkdir()
            cache_root = Path(tmpdir) / ".cache"
            corpus_root = cache_root / "corpus_01"
            normalized_root = corpus_root / "normalized"
            normalized_root.mkdir(parents=True)
            r1_path = normalized_root / "sample_0001_R1.fastq.gz"
            r1_path.write_text("reads", encoding="utf-8")
            out_root = cache_root / "bijux-dna-results"
            orphaned_sample_root = out_root / "bench" / "trim_reads" / "sample_0001"
            orphaned_sample_root.mkdir(parents=True)
            stale_marker = orphaned_sample_root / "stale.marker"
            stale_marker.write_text("old", encoding="utf-8")

            def fake_run(
                command: list[str],
                cwd: Path,
                check: bool = False,
                env: dict[str, str] | None = None,
            ):
                self.assertEqual(Path(cwd).resolve(), repo_root.resolve())
                self.assertFalse(stale_marker.exists())
                self.assertIsNotNone(env)
                self.assertEqual(Path(env["BIJUX_CACHE_ROOT"]).resolve(), cache_root.resolve())
                self.assertEqual(
                    Path(env["BIJUX_HPC_ROOT"]).resolve(),
                    Path(tmpdir).resolve(),
                )
                fresh_report = (
                    out_root / "bench" / "trim_reads" / "sample_0001" / "report.json"
                )
                fresh_report.parent.mkdir(parents=True, exist_ok=True)
                fresh_report.write_text(
                    json.dumps(
                        {
                            "failures": [],
                            "gate": {"passes": True},
                            "records": [{"context": {"tool": "fastp"}}],
                            "semantic_metrics": [],
                        }
                    )
                    + "\n",
                    encoding="utf-8",
                )
                return mock.Mock(returncode=0)

            argv = [
                "run_fastq_trim_reads_corpus_01.py",
                "--repo-root",
                str(repo_root),
                "--corpus-root",
                str(corpus_root),
                "--out-root",
                str(out_root),
            ]
            with mock.patch.object(sys, "argv", argv):
                with mock.patch.object(
                    trim_reads_runner,
                    "load_corpus_spec",
                    return_value={
                        "corpus_id": "corpus-01",
                        "preferred_root": str(corpus_root),
                    },
                ):
                    with mock.patch.object(
                        trim_reads_runner,
                        "discover_normalized_samples",
                        return_value=[
                            {
                                "sample_id": "sample_0001",
                                "r1": r1_path,
                                "r2": None,
                                "layout": "se",
                            }
                        ],
                    ):
                        with mock.patch.object(
                            trim_reads_runner,
                            "validate_corpus_contract",
                        ):
                            with mock.patch.object(
                                trim_reads_runner,
                                "require_canonical_tool_roster",
                                return_value=["fastp"],
                            ):
                                with mock.patch.object(
                                    trim_reads_runner.subprocess,
                                    "run",
                                    side_effect=fake_run,
                                ) as run_mock:
                                    exit_code = trim_reads_runner.main()

            self.assertEqual(exit_code, 0)
            self.assertEqual(run_mock.call_count, 1)
            self.assertFalse(stale_marker.exists())
            manifest = json.loads(
                (out_root / "run_manifest.json").read_text(encoding="utf-8")
            )
            self.assertEqual(manifest["samples_failed"], 0)
            self.assertEqual(manifest["runs"][0]["status"], "completed")

    def test_trim_reads_report_localizes_lunarc_report_paths(self) -> None:
        local_results_root = Path("/tmp/local-results")

        localized = trim_reads_report.localize_results_path(
            "/home/bijan/bijux/results/corpus_01/fastq.trim_reads/lunarc/bench/trim_reads/sample_0008/report.json",
            local_results_root,
        )

        self.assertEqual(
            localized,
            local_results_root
            / "corpus_01/fastq.trim_reads/lunarc/bench/trim_reads/sample_0008/report.json",
        )


class MergeReportingTests(unittest.TestCase):
    def test_merge_runner_parse_args_supports_sample_jobs(self) -> None:
        original_argv = sys.argv
        try:
            sys.argv = ["run_fastq_merge_pairs_corpus_01.py", "--sample-jobs", "3"]
            args = merge_runner.parse_args()
        finally:
            sys.argv = original_argv

        self.assertEqual(args.sample_jobs, 3)

    def test_merge_runtime_summary_tracks_runtime_and_merge_rate(self) -> None:
        rows = [
            {
                "tool": "pear",
                "runtime_s": "1.0",
                "exit_code": "0",
                "merge_rate": "0.80",
                "base_retention": "0.70",
                "reads_merged": "800",
            },
            {
                "tool": "pear",
                "runtime_s": "1.2",
                "exit_code": "0",
                "merge_rate": "0.84",
                "base_retention": "0.74",
                "reads_merged": "840",
            },
            {
                "tool": "bbmerge",
                "runtime_s": "2.0",
                "exit_code": "0",
                "merge_rate": "0.81",
                "base_retention": "0.72",
                "reads_merged": "810",
            },
            {
                "tool": "bbmerge",
                "runtime_s": "2.2",
                "exit_code": "0",
                "merge_rate": "0.82",
                "base_retention": "0.71",
                "reads_merged": "820",
            },
        ]

        summary_rows = merge_briefing.tool_runtime_summary(rows)
        by_tool = {row["tool"]: row for row in summary_rows}

        self.assertAlmostEqual(by_tool["pear"]["median_runtime_s"], 1.1)
        self.assertAlmostEqual(by_tool["pear"]["median_merge_rate"], 0.82)
        self.assertAlmostEqual(by_tool["pear"]["mean_reads_merged"], 820.0)
        self.assertGreater(
            by_tool["bbmerge"]["slowdown_vs_fastest_median"],
            by_tool["pear"]["slowdown_vs_fastest_median"],
        )

    def test_merge_outliers_capture_slowest_and_best_merge_tool(self) -> None:
        rows = [
            {
                "sample_id": "sample_0008",
                "accession": "ACC8",
                "era": "ancient",
                "layout": "pe",
                "size_band": "under_500mb",
                "study_accession": "PRJ8",
                "tool": "pear",
                "runtime_s": "3.0",
                "merge_rate": "0.83",
            },
            {
                "sample_id": "sample_0008",
                "accession": "ACC8",
                "era": "ancient",
                "layout": "pe",
                "size_band": "under_500mb",
                "study_accession": "PRJ8",
                "tool": "bbmerge",
                "runtime_s": "4.5",
                "merge_rate": "0.79",
            },
        ]

        outliers = merge_briefing.sample_runtime_outliers(rows)

        self.assertEqual(outliers[0]["slowest_tool"], "bbmerge")
        self.assertEqual(outliers[0]["best_merge_rate_tool"], "pear")
        self.assertAlmostEqual(outliers[0]["best_merge_rate"], 0.83)

    def test_merge_briefing_reproducibility_uses_configured_archive_language(self) -> None:
        summary = {
            "stage_id": "fastq.merge_pairs",
            "scenario_id": "merge_fairness",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/lu2024-12-24/.cache/corpus_01",
            "run_root": "/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.merge_pairs/lunarc",
            "samples_total": 2,
            "samples_failed": 0,
            "layout_counts": {"pe": 2, "se": 0},
            "era_counts": {"ancient": 1, "modern": 1},
            "tools": ["pear", "bbmerge"],
            "merge_overlap": None,
            "min_length": None,
            "unmerged_read_policy": "emit_unmerged_pairs",
        }
        rows = [
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "ancient",
                "layout": "pe",
                "size_band": "under_100mb",
                "study_accession": "PRJ1",
                "tool": "pear",
                "runtime_s": "1.0",
                "exit_code": "0",
                "merge_rate": "0.80",
                "base_retention": "0.70",
                "reads_merged": "800",
            },
            {
                "sample_id": "sample_0002",
                "accession": "ACC2",
                "era": "modern",
                "layout": "pe",
                "size_band": "under_500mb",
                "study_accession": "PRJ2",
                "tool": "bbmerge",
                "runtime_s": "1.5",
                "exit_code": "0",
                "merge_rate": "0.82",
                "base_retention": "0.72",
                "reads_merged": "820",
            },
        ]

        markdown = merge_briefing.render_markdown(
            summary=summary,
            rows=rows,
            runtime_rows=merge_briefing.tool_runtime_summary(rows),
            cohort_rows=merge_briefing.cohort_runtime_summary(rows),
            outliers=merge_briefing.sample_runtime_outliers(rows),
        )

        self.assertIn("configured local benchmark archive", markdown)
        self.assertNotIn(
            "mirrored locally under `/Users/bijan/bijux/bijux-dna-results`",
            markdown,
        )

    def test_merge_markdown_mentions_paired_only_contract(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/results/corpus_01/fastq.merge_pairs/lunarc",
            "scenario_id": "merge_fairness",
            "samples_total": 10,
            "samples_failed": 0,
            "tools": ["adapterremoval", "pear"],
            "merge_overlap": None,
            "min_length": None,
            "unmerged_read_policy": "emit_unmerged_pairs",
            "era_counts": {"ancient": 5, "modern": 5},
            "cohort_counts": {"ancient_pe": 5, "modern_pe": 5},
            "headline": {
                "fastest_tool": "pear",
                "fastest_runtime_s": 1.1,
                "best_merge_rate_tool": "pear",
                "best_merge_rate": 0.82,
                "best_base_retention_tool": "pear",
                "best_base_retention": 0.72,
            },
            "tool_summary": [
                {
                    "tool": "pear",
                    "records": 10,
                    "pass_rate": 1.0,
                    "median_runtime_s": 1.1,
                    "median_merge_rate": 0.82,
                    "median_base_retention": 0.72,
                    "mean_reads_merged": 820.0,
                }
            ],
        }

        markdown = merge_report.render_markdown(summary)

        self.assertIn("Samples benchmarked: `10` paired-end inputs", markdown)
        self.assertIn("merge_overlap: `governed tool default`", markdown)
        self.assertIn("min_length: `governed tool default`", markdown)
        self.assertIn("unmerged_read_policy: `emit_unmerged_pairs`", markdown)

    def test_merge_report_contract_rejects_rate_drift(self) -> None:
        run_manifest = {
            "tools": ["pear"],
            "merge_overlap": None,
            "min_length": None,
            "unmerged_read_policy": "emit_unmerged_pairs",
        }
        sample_rows = [
            {
                "sample_id": "sample_0008",
                "tool": "pear",
                "layout": "pe",
                "merge_overlap": None,
                "min_length": None,
                "unmerged_read_policy": "emit_unmerged_pairs",
                "pairs_in": 100,
                "reads_merged": 80,
                "reads_unmerged": 20,
                "merge_rate": 0.70,
            }
        ]

        with self.assertRaises(SystemExit):
            merge_report.validate_merge_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0008"],
            )

    def test_merge_report_contract_rejects_missing_sample_rows(self) -> None:
        run_manifest = {
            "tools": ["pear"],
            "merge_overlap": None,
            "min_length": None,
            "unmerged_read_policy": "emit_unmerged_pairs",
        }
        sample_rows = [
            {
                "sample_id": "sample_0008",
                "tool": "pear",
                "layout": "pe",
                "merge_overlap": None,
                "min_length": None,
                "unmerged_read_policy": "emit_unmerged_pairs",
                "pairs_in": 100,
                "reads_merged": 80,
                "reads_unmerged": 20,
                "merge_rate": 0.8,
            }
        ]

        with self.assertRaises(SystemExit):
            merge_report.validate_merge_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0008", "sample_0009"],
            )

    def test_merge_report_localizes_remote_results_path(self) -> None:
        local_results_root = Path("/tmp/local-results")

        localized = merge_report.localize_results_path(
            "/home/bijan/bijux/results/corpus_01/fastq.merge_pairs/lunarc/bench/merge_pairs/sample_0008/report.json",
            local_results_root,
        )

        self.assertEqual(
            localized,
            local_results_root
            / "corpus_01/fastq.merge_pairs/lunarc/bench/merge_pairs/sample_0008/report.json",
        )

    def test_merge_report_prefers_canonical_report_path_when_manifest_path_drifts(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            run_root = Path(tmpdir)
            canonical = run_root / "bench" / "merge_pairs" / "sample_0008" / "report.json"
            canonical.parent.mkdir(parents=True)
            canonical.write_text("{}", encoding="utf-8")

            resolved = merge_report.resolve_merge_report_path(
                sample_id="sample_0008",
                reported_path="/home/bijan/bijux/results/corpus_01/fastq.merge_pairs/lunarc/bench/merge/sample_0008/report.json",
                run_root=run_root,
                local_results_root=Path("/tmp/local-results"),
            )

            self.assertEqual(resolved, canonical)

    def test_merge_summary_publishes_paired_sample_scope(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            run_root = repo_root / "results" / "corpus_01" / "fastq.merge_pairs" / "lunarc"
            docs_root = repo_root / "docs" / "benchmark" / "fastq.merge_pairs" / "corpus-01"
            sample_report = (
                run_root / "bench" / "merge_pairs" / "sample_0001" / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {
                                    "tool": "bbmerge",
                                    "parameters": {
                                        "merge_overlap": 20,
                                        "min_length": 30,
                                        "unmerged_read_policy": "emit_unmerged_pairs",
                                    },
                                },
                                "execution": {"runtime_s": 1.5, "exit_code": 0},
                                "metrics": {
                                    "metrics": {
                                        "reads_in": 100,
                                        "reads_out": 90,
                                        "bases_in": 1000,
                                        "bases_out": 920,
                                        "pairs_in": 50,
                                        "pairs_out": 45,
                                        "reads_r1": 50,
                                        "reads_r2": 50,
                                        "reads_merged": 10,
                                        "reads_unmerged": 40,
                                        "merge_rate": 0.2,
                                    }
                                },
                            }
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "platform": "lunarc-apptainer",
                        "stage_id": "fastq.merge_pairs",
                        "scenario_id": "merge_fairness",
                        "tool_kind": "benchmark",
                        "sample_scope": "paired",
                        "corpus_root": "/home/bijan/bijux/corpus_01",
                        "tools": ["bbmerge"],
                        "merge_overlap": 20,
                        "min_length": 30,
                        "unmerged_read_policy": "emit_unmerged_pairs",
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "layout": "pe",
                                "report_json": str(sample_report),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            original_argv = sys.argv
            try:
                sys.argv = [
                    "render_fastq_merge_pairs_corpus_01_report.py",
                    "--repo-root",
                    str(repo_root),
                    "--corpus-root",
                    "/home/bijan/lu2024-12-24/.cache/corpus_01",
                    "--run-root",
                    str(run_root),
                    "--docs-root",
                    str(docs_root.relative_to(repo_root)),
                ]
                with mock.patch.object(
                    merge_report,
                    "load_corpus_spec",
                    return_value={"corpus_id": "corpus-01"},
                ), mock.patch.object(
                    merge_report,
                    "resolve_corpus_metadata",
                    return_value={
                        "sample_0001": {
                            "accession": "ACC1",
                            "era": "modern",
                            "layout": "pe",
                            "study_accession": "PRJ1",
                            "size_band": "under_100mb",
                        }
                    },
                ):
                    self.assertEqual(merge_report.main(), 0)
            finally:
                sys.argv = original_argv

            summary = json.loads((docs_root / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(summary["sample_scope"], "paired")

class DetectAdaptersReportingTests(unittest.TestCase):
    def test_detect_adapters_summary_tracks_runtime_and_signal(self) -> None:
        rows = [
            {
                "tool": "fastqc",
                "runtime_s": "1.2",
                "exit_code": "0",
                "candidate_adapter_count": "2",
                "adapter_trimmed_fraction": "",
                "mean_q": "31.5",
            },
            {
                "tool": "fastqc",
                "runtime_s": "1.4",
                "exit_code": "0",
                "candidate_adapter_count": "4",
                "adapter_trimmed_fraction": "",
                "mean_q": "32.5",
            },
        ]

        summary_rows = detect_adapters_briefing.tool_runtime_summary(rows)

        self.assertEqual(len(summary_rows), 1)
        self.assertAlmostEqual(summary_rows[0]["median_runtime_s"], 1.3)
        self.assertAlmostEqual(summary_rows[0]["mean_candidate_adapter_count"], 3.0)
        self.assertAlmostEqual(summary_rows[0]["median_mean_q"], 32.0)

    def test_detect_adapters_briefing_avoids_hardcoded_tool_name(self) -> None:
        summary = {
            "stage_id": "fastq.detect_adapters",
            "scenario_id": "detect_adapters_fairness",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/results/corpus_01/fastq.detect_adapters/lunarc",
            "samples_total": 1,
            "samples_failed": 0,
            "tools": ["adapter_observer"],
            "inspection_mode": "evidence_only",
            "report_only": True,
            "evidence_scope": "full_input",
            "evidence_format": "fastqc_summary",
            "era_counts": {"ancient": 1, "modern": 0},
            "layout_counts": {"se": 1, "pe": 0},
        }
        rows = [
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "ancient",
                "layout": "se",
                "size_band": "under_100mb",
                "study_accession": "PRJ1",
                "tool": "adapter_observer",
                "runtime_s": "1.0",
                "exit_code": "0",
                "candidate_adapter_count": "2",
                "adapter_trimmed_fraction": "",
                "mean_q": "31.0",
            }
        ]
        runtime_rows = detect_adapters_briefing.tool_runtime_summary(rows)
        cohort_rows = detect_adapters_briefing.cohort_runtime_summary(rows)
        outliers = detect_adapters_briefing.sample_runtime_outliers(rows)

        markdown = detect_adapters_briefing.render_markdown(
            summary, rows, runtime_rows, cohort_rows, outliers
        )

        self.assertIn("`adapter_observer` ran at", markdown)
        self.assertNotIn("`fastqc` ran at", markdown)

    def test_detect_adapters_briefing_rejects_tool_drift(self) -> None:
        with self.assertRaises(SystemExit):
            detect_adapters_briefing.validate_rows_contract(
                {"tools": ["fastqc"]},
                rows=[
                    {
                        "tool": "other_tool",
                    }
                ],
            )

    def test_detect_adapters_markdown_mentions_observer_contract(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/results/corpus_01/fastq.detect_adapters/lunarc",
            "scenario_id": "detect_adapters_fairness",
            "samples_total": 20,
            "samples_failed": 0,
            "tools": ["fastqc"],
            "inspection_mode": "evidence_only",
            "report_only": True,
            "evidence_scope": "full_input",
            "evidence_format": "fastqc_summary",
            "era_counts": {"ancient": 10, "modern": 10},
            "layout_counts": {"se": 10, "pe": 10},
            "cohort_counts": {"ancient_pe": 5, "ancient_se": 5, "modern_pe": 5, "modern_se": 5},
            "headline": {
                "fastest_tool": "fastqc",
                "fastest_runtime_s": 1.3,
                "largest_adapter_signal_tool": "fastqc",
                "largest_adapter_signal": 3.0,
                "highest_trimmed_fraction_tool": None,
                "highest_trimmed_fraction": None,
            },
            "tool_summary": [
                {
                    "tool": "fastqc",
                    "records": 20,
                    "pass_rate": 1.0,
                    "median_runtime_s": 1.3,
                    "mean_candidate_adapter_count": 3.0,
                    "mean_adapter_trimmed_fraction": None,
                    "median_mean_q": 32.0,
                }
            ],
        }

        markdown = detect_adapters_report.render_markdown(summary)

        self.assertIn("inspection_mode: `evidence_only`", markdown)
        self.assertIn("report_only: `True`", markdown)

    def test_detect_adapters_report_contract_rejects_mutating_rows(self) -> None:
        run_manifest = {"tools": ["fastqc"]}
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "fastqc",
                "reads_in": 100,
                "reads_out": 99,
                "bases_in": 1000,
                "bases_out": 1000,
                "adapter_trimmed_fraction": None,
            }
        ]

        with self.assertRaises(SystemExit):
            detect_adapters_report.validate_detect_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0001"],
            )

    def test_detect_adapters_report_contract_rejects_missing_sample_rows(self) -> None:
        with self.assertRaises(SystemExit):
            detect_adapters_report.validate_detect_row_contract(
                run_manifest={"tools": ["fastqc"]},
                sample_rows=[],
                expected_sample_ids=["sample_0001"],
            )

    def test_detect_adapters_report_rejects_dry_run_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            detect_adapters_report.validate_detect_run_manifest_contract(
                {
                    "stage_id": "fastq.detect_adapters",
                    "scenario_id": "detect_adapters_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": True,
                    "inspection_mode": "evidence_only",
                    "report_only": True,
                    "evidence_scope": "full_input",
                    "evidence_format": "fastqc_summary",
                }
            )

    def test_detect_adapters_report_rejects_sample_limited_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            detect_adapters_report.validate_detect_run_manifest_contract(
                {
                    "stage_id": "fastq.detect_adapters",
                    "scenario_id": "detect_adapters_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": False,
                    "sample_limit": 2,
                    "inspection_mode": "evidence_only",
                    "report_only": True,
                    "evidence_scope": "full_input",
                    "evidence_format": "fastqc_summary",
                }
            )


class OverrepresentedReportingTests(unittest.TestCase):
    def test_overrepresented_markdown_mentions_top_k_contract(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/results/corpus_01/fastq.profile_overrepresented_sequences/lunarc",
            "scenario_id": "overrepresented_sequence_fairness",
            "samples_total": 20,
            "samples_failed": 0,
            "tools": ["fastqc", "fastq_scan", "seqkit"],
            "top_k": 50,
            "report_only": True,
            "mutates_fastq": False,
            "may_change_read_count": False,
            "era_counts": {"ancient": 10, "modern": 10},
            "layout_counts": {"se": 10, "pe": 10},
            "cohort_counts": {"ancient_pe": 5, "ancient_se": 5, "modern_pe": 5, "modern_se": 5},
            "headline": {
                "fastest_tool": "seqkit",
                "fastest_runtime_s": 0.8,
                "largest_sequence_count_tool": "fastqc",
                "largest_sequence_count": 12.0,
                "highest_top_fraction_tool": "fastq_scan",
                "highest_top_fraction": 0.12,
            },
            "tool_summary": [
                {
                    "tool": "seqkit",
                    "records": 20,
                    "pass_rate": 1.0,
                    "median_runtime_s": 0.8,
                    "median_sequence_count": 10.0,
                    "median_flagged_sequences": 2.0,
                    "median_top_fraction": 0.09,
                }
            ],
        }

        markdown = overrepresented_report.render_markdown(summary)

        self.assertIn("top_k: `50`", markdown)
        self.assertIn("Median flagged sequences", markdown)

    def test_overrepresented_report_rejects_dry_run_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            overrepresented_report.validate_overrepresented_run_manifest_contract(
                {
                    "stage_id": "fastq.profile_overrepresented_sequences",
                    "scenario_id": "overrepresented_sequence_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": True,
                }
            )

    def test_overrepresented_report_rejects_sample_limited_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            overrepresented_report.validate_overrepresented_run_manifest_contract(
                {
                    "stage_id": "fastq.profile_overrepresented_sequences",
                    "scenario_id": "overrepresented_sequence_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": False,
                    "sample_limit": 4,
                    "report_only": True,
                    "mutates_fastq": False,
                    "may_change_read_count": False,
                    "top_k": 50,
                    "overrepresented_artifacts": [
                        "overrepresented_sequences_tsv",
                        "overrepresented_sequences_json",
                        "report_json",
                    ],
                }
            )

    def test_overrepresented_report_contract_rejects_missing_sample_tool_rows(self) -> None:
        with self.assertRaises(SystemExit):
            overrepresented_report.validate_overrepresented_row_contract(
                run_manifest={"tools": ["fastqc", "seqkit"], "top_k": 50},
                sample_rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "fastqc",
                        "sequence_count": 5,
                        "flagged_sequences": 1,
                        "top_fraction": 0.1,
                        "top_k": 50,
                    }
                ],
                expected_sample_ids=["sample_0001"],
            )

    def test_overrepresented_report_contract_rejects_excess_ranked_sequences(self) -> None:
        with self.assertRaises(SystemExit):
            overrepresented_report.validate_overrepresented_row_contract(
                run_manifest={"tools": ["fastqc"], "top_k": 5},
                sample_rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "fastqc",
                        "sequence_count": 6,
                        "flagged_sequences": 1,
                        "top_fraction": 0.1,
                        "top_k": 5,
                    }
                ],
                expected_sample_ids=["sample_0001"],
            )

    def test_overrepresented_report_validates_artifact_publication(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            report_path = (
                Path(tmpdir)
                / "bench"
                / "profile_overrepresented_sequences"
                / "sample_0001"
                / "report.json"
            )
            tool_dir = report_path.parent / "tools" / "fastqc"
            tool_dir.mkdir(parents=True)
            (tool_dir / "overrepresented_sequences.tsv").write_text(
                "sequence\tcount\nACGT\t4\n",
                encoding="utf-8",
            )
            (tool_dir / "overrepresented_sequences.json").write_text(
                json.dumps({"sequence_count": 1}) + "\n",
                encoding="utf-8",
            )
            (tool_dir / "overrepresented_report.json").write_text(
                json.dumps({"top_fraction": 0.2}) + "\n",
                encoding="utf-8",
            )

            artifacts = overrepresented_report.validate_artifact_paths(report_path, "fastqc")

        self.assertTrue(
            artifacts["overrepresented_sequences_tsv_artifact"].endswith(
                "overrepresented_sequences.tsv"
            )
        )
        self.assertTrue(
            artifacts["overrepresented_sequences_json_artifact"].endswith(
                "overrepresented_sequences.json"
            )
        )

    def test_overrepresented_report_rejects_empty_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            report_path = (
                Path(tmpdir)
                / "bench"
                / "profile_overrepresented_sequences"
                / "sample_0001"
                / "report.json"
            )
            tool_dir = report_path.parent / "tools" / "fastqc"
            tool_dir.mkdir(parents=True)
            (tool_dir / "overrepresented_sequences.tsv").write_text("", encoding="utf-8")
            (tool_dir / "overrepresented_sequences.json").write_text("{}", encoding="utf-8")
            (tool_dir / "overrepresented_report.json").write_text("{}", encoding="utf-8")

            with self.assertRaises(SystemExit):
                overrepresented_report.validate_artifact_paths(report_path, "fastqc")

    def test_overrepresented_briefing_avoids_hardcoded_tool_name(self) -> None:
        summary = {
            "stage_id": "fastq.profile_overrepresented_sequences",
            "scenario_id": "overrepresented_sequence_fairness",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/results/corpus_01/fastq.profile_overrepresented_sequences/lunarc",
            "samples_total": 1,
            "samples_failed": 0,
            "tools": ["observer_a"],
            "top_k": 50,
            "report_only": True,
            "mutates_fastq": False,
            "may_change_read_count": False,
            "era_counts": {"ancient": 1, "modern": 0},
            "layout_counts": {"se": 1, "pe": 0},
        }
        rows = [
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "ancient",
                "layout": "se",
                "size_band": "under_100mb",
                "study_accession": "PRJ1",
                "tool": "observer_a",
                "runtime_s": "1.0",
                "exit_code": "0",
                "sequence_count": "5",
                "flagged_sequences": "1",
                "top_fraction": "0.1",
                "top_k": "50",
                "overrepresented_sequences_tsv_artifact": "/tmp/overrepresented_sequences.tsv",
                "overrepresented_sequences_json_artifact": "/tmp/overrepresented_sequences.json",
                "report_json_artifact": "/tmp/overrepresented_report.json",
            }
        ]

        runtime_rows = overrepresented_briefing.tool_runtime_summary(rows)
        cohort_rows = overrepresented_briefing.cohort_runtime_summary(rows)
        outliers = overrepresented_briefing.sample_runtime_outliers(rows)

        markdown = overrepresented_briefing.render_markdown(
            summary, rows, runtime_rows, cohort_rows, outliers
        )

        self.assertIn("`observer_a` ran at", markdown)
        self.assertNotIn("`fastqc` ran at", markdown)

    def test_overrepresented_briefing_rejects_sequence_count_drift(self) -> None:
        with self.assertRaises(SystemExit):
            overrepresented_briefing.validate_rows_contract(
                {"tools": ["fastqc"], "top_k": 5},
                rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "fastqc",
                        "sequence_count": "6",
                        "flagged_sequences": "1",
                        "top_fraction": "0.1",
                        "top_k": "5",
                        "overrepresented_sequences_tsv_artifact": "/tmp/overrepresented_sequences.tsv",
                        "overrepresented_sequences_json_artifact": "/tmp/overrepresented_sequences.json",
                        "report_json_artifact": "/tmp/overrepresented_report.json",
                    }
                ],
            )

    def test_overrepresented_briefing_rejects_artifact_suffix_drift(self) -> None:
        with self.assertRaises(SystemExit):
            overrepresented_briefing.validate_rows_contract(
                {"tools": ["fastqc"], "top_k": 50},
                rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "fastqc",
                        "sequence_count": "5",
                        "flagged_sequences": "1",
                        "top_fraction": "0.1",
                        "top_k": "50",
                        "overrepresented_sequences_tsv_artifact": "/tmp/wrong.tsv",
                        "overrepresented_sequences_json_artifact": "/tmp/overrepresented_sequences.json",
                        "report_json_artifact": "/tmp/overrepresented_report.json",
                    }
                ],
            )


class ProfileReadsReportingTests(unittest.TestCase):
    def test_profile_reads_summary_tracks_runtime_and_profile_metrics(self) -> None:
        histogram = [{"length": 50, "count": 2}, {"length": 75, "count": 2}]
        derived = profile_reads_report.derived_histogram_metrics(histogram)

        self.assertEqual(derived["histogram_bin_count"], 2)
        self.assertEqual(derived["max_observed_length"], 75)
        self.assertAlmostEqual(derived["mean_read_length"], 62.5)

    def test_profile_reads_markdown_mentions_profile_contract(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/results/corpus_01/fastq.profile_reads/lunarc",
            "scenario_id": "profile_reads_fairness",
            "samples_total": 20,
            "samples_failed": 0,
            "tools": ["seqkit_stats"],
            "report_only": True,
            "mutates_fastq": False,
            "may_change_read_count": False,
            "raw_backend_report_format": "seqkit_stats_tsv",
            "length_histogram_source": "seqkit_fx2tab",
            "era_counts": {"ancient": 10, "modern": 10},
            "layout_counts": {"se": 10, "pe": 10},
            "cohort_counts": {"ancient_pe": 5, "ancient_se": 5, "modern_pe": 5, "modern_se": 5},
            "headline": {
                "fastest_tool": "seqkit_stats",
                "fastest_runtime_s": 1.1,
                "highest_mean_q_tool": "seqkit_stats",
                "highest_mean_q": 33.2,
                "widest_histogram_tool": "seqkit_stats",
                "widest_histogram_bins": 42,
            },
            "tool_summary": [
                {
                    "tool": "seqkit_stats",
                    "records": 20,
                    "pass_rate": 1.0,
                    "median_runtime_s": 1.1,
                    "median_reads_total": 1000.0,
                    "median_bases_total": 75000.0,
                    "median_mean_q": 33.2,
                    "median_gc_percent": 45.0,
                    "median_read_length": 75.0,
                    "median_histogram_bin_count": 42.0,
                }
            ],
        }

        markdown = profile_reads_report.render_markdown(summary)

        self.assertIn("raw_backend_report_format: `seqkit_stats_tsv`", markdown)
        self.assertIn("length_histogram_source: `seqkit_fx2tab`", markdown)

    def test_profile_reads_report_contract_rejects_empty_histograms(self) -> None:
        with self.assertRaises(SystemExit):
            profile_reads_report.validate_profile_reads_row_contract(
                run_manifest={"tools": ["seqkit_stats"]},
                sample_rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "seqkit_stats",
                        "reads_total": 100,
                        "bases_total": 1000,
                        "mean_q": 31.0,
                        "gc_percent": 45.0,
                        "histogram_bin_count": 0,
                        "max_observed_length": 75,
                        "mean_read_length": 10.0,
                    }
                ],
                expected_sample_ids=["sample_0001"],
            )

    def test_profile_reads_report_contract_rejects_histogram_length_drift(self) -> None:
        with self.assertRaises(SystemExit):
            profile_reads_report.validate_profile_reads_row_contract(
                run_manifest={"tools": ["seqkit_stats"]},
                sample_rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "seqkit_stats",
                        "reads_total": 100,
                        "bases_total": 1000,
                        "mean_q": 31.0,
                        "gc_percent": 45.0,
                        "histogram_bin_count": 10,
                        "max_observed_length": 5,
                        "mean_read_length": 10.0,
                    }
                ],
                expected_sample_ids=["sample_0001"],
            )

    def test_profile_reads_report_rejects_dry_run_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            profile_reads_report.validate_profile_reads_run_manifest_contract(
                {
                    "stage_id": "fastq.profile_reads",
                    "scenario_id": "profile_reads_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": True,
                    "report_only": True,
                    "mutates_fastq": False,
                    "may_change_read_count": False,
                    "raw_backend_report_format": "seqkit_stats_tsv",
                    "length_histogram_source": "seqkit_fx2tab",
                }
            )

    def test_profile_reads_report_rejects_sample_limited_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            profile_reads_report.validate_profile_reads_run_manifest_contract(
                {
                    "stage_id": "fastq.profile_reads",
                    "scenario_id": "profile_reads_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": False,
                    "sample_limit": 2,
                    "report_only": True,
                    "mutates_fastq": False,
                    "may_change_read_count": False,
                    "raw_backend_report_format": "seqkit_stats_tsv",
                    "length_histogram_source": "seqkit_fx2tab",
                }
            )

    def test_profile_reads_briefing_avoids_hardcoded_tool_name(self) -> None:
        summary = {
            "stage_id": "fastq.profile_reads",
            "scenario_id": "profile_reads_fairness",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/results/corpus_01/fastq.profile_reads/lunarc",
            "samples_total": 1,
            "samples_failed": 0,
            "tools": ["profile_observer"],
            "report_only": True,
            "mutates_fastq": False,
            "may_change_read_count": False,
            "raw_backend_report_format": "seqkit_stats_tsv",
            "length_histogram_source": "seqkit_fx2tab",
            "era_counts": {"ancient": 1, "modern": 0},
            "layout_counts": {"se": 1, "pe": 0},
        }
        rows = [
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "ancient",
                "layout": "se",
                "size_band": "under_100mb",
                "study_accession": "PRJ1",
                "tool": "profile_observer",
                "runtime_s": "1.0",
                "exit_code": "0",
                "reads_total": "100",
                "bases_total": "5000",
                "mean_q": "31.0",
                "gc_percent": "44.0",
                "histogram_bin_count": "2",
                "max_observed_length": "75",
                "mean_read_length": "50.0",
            }
        ]
        runtime_rows = profile_reads_briefing.tool_runtime_summary(rows)
        cohort_rows = profile_reads_briefing.cohort_runtime_summary(rows)
        outliers = profile_reads_briefing.sample_runtime_outliers(rows)

        markdown = profile_reads_briefing.render_markdown(
            summary, rows, runtime_rows, cohort_rows, outliers
        )

        self.assertIn("`profile_observer` ran at", markdown)
        self.assertNotIn("`seqkit_stats` ran at", markdown)

    def test_profile_reads_briefing_rejects_contract_drift(self) -> None:
        with self.assertRaises(SystemExit):
            profile_reads_briefing.validate_summary_contract(
                {
                    "stage_id": "fastq.profile_reads",
                    "scenario_id": "profile_reads_fairness",
                    "tools": ["seqkit_stats"],
                    "report_only": True,
                    "mutates_fastq": False,
                    "may_change_read_count": False,
                    "raw_backend_report_format": "wrong",
                    "length_histogram_source": "seqkit_fx2tab",
                }
            )

    def test_profile_reads_briefing_rejects_histogram_row_drift(self) -> None:
        with self.assertRaises(SystemExit):
            profile_reads_briefing.validate_rows_contract(
                {"tools": ["seqkit_stats"]},
                rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "seqkit_stats",
                        "histogram_bin_count": "0",
                    }
                ],
            )


class ProfileReadLengthsReportingTests(unittest.TestCase):
    def test_read_length_markdown_mentions_histogram_contract(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/results/corpus_01/fastq.profile_read_lengths/lunarc",
            "scenario_id": "read_length_fairness",
            "samples_total": 20,
            "samples_failed": 0,
            "tools": ["seqkit_stats"],
            "report_only": True,
            "mutates_fastq": False,
            "may_change_read_count": False,
            "raw_backend_report_format": "seqkit_stats_length_histogram",
            "histogram_bins": 100,
            "era_counts": {"ancient": 10, "modern": 10},
            "layout_counts": {"se": 10, "pe": 10},
            "cohort_counts": {"ancient_pe": 5, "ancient_se": 5, "modern_pe": 5, "modern_se": 5},
            "headline": {
                "fastest_tool": "seqkit_stats",
                "fastest_runtime_s": 1.2,
                "highest_max_read_length_tool": "seqkit_stats",
                "highest_max_read_length": 151.0,
                "widest_length_support_tool": "seqkit_stats",
                "widest_length_support": 48.0,
            },
            "tool_summary": [
                {
                    "tool": "seqkit_stats",
                    "records": 20,
                    "pass_rate": 1.0,
                    "median_runtime_s": 1.2,
                    "median_read_count": 1000.0,
                    "median_mean_read_length": 74.2,
                    "median_max_read_length": 151.0,
                    "median_distinct_lengths": 48.0,
                }
            ],
        }

        markdown = profile_read_lengths_report.render_markdown(summary)

        self.assertIn("raw_backend_report_format: `seqkit_stats_length_histogram`", markdown)
        self.assertIn("histogram_bins: `100`", markdown)

    def test_read_length_report_contract_rejects_invalid_distinct_lengths(self) -> None:
        with self.assertRaises(SystemExit):
            profile_read_lengths_report.validate_read_length_row_contract(
                run_manifest={"tools": ["seqkit_stats"]},
                sample_rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "seqkit_stats",
                        "read_count": 100,
                        "mean_read_length": 50.0,
                        "max_read_length": 75,
                        "distinct_lengths": 101,
                    }
                ],
                expected_sample_ids=["sample_0001"],
            )

    def test_read_length_report_rejects_sample_limited_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            profile_read_lengths_report.validate_read_length_run_manifest_contract(
                {
                    "stage_id": "fastq.profile_read_lengths",
                    "scenario_id": "read_length_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": False,
                    "sample_limit": 2,
                    "report_only": True,
                    "mutates_fastq": False,
                    "may_change_read_count": False,
                    "raw_backend_report_format": "seqkit_stats_length_histogram",
                    "histogram_bins": 100,
                    "length_histogram_artifacts": [
                        "report_json",
                        "length_distribution_tsv",
                        "length_distribution_json",
                    ],
                }
            )

    def test_read_length_report_rejects_nonpositive_histogram_bins(self) -> None:
        with self.assertRaises(SystemExit):
            profile_read_lengths_report.validate_read_length_run_manifest_contract(
                {
                    "stage_id": "fastq.profile_read_lengths",
                    "scenario_id": "read_length_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": False,
                    "report_only": True,
                    "mutates_fastq": False,
                    "may_change_read_count": False,
                    "raw_backend_report_format": "seqkit_stats_length_histogram",
                    "histogram_bins": 0,
                    "length_histogram_artifacts": [
                        "report_json",
                        "length_distribution_tsv",
                        "length_distribution_json",
                    ],
                }
            )

    def test_read_length_report_rejects_nonpositive_mean_length(self) -> None:
        with self.assertRaises(SystemExit):
            profile_read_lengths_report.validate_read_length_row_contract(
                run_manifest={"tools": ["seqkit_stats"]},
                sample_rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "seqkit_stats",
                        "read_count": 100,
                        "mean_read_length": 0.0,
                        "max_read_length": 75,
                        "distinct_lengths": 10,
                    }
                ],
                expected_sample_ids=["sample_0001"],
            )

    def test_read_length_artifact_check_rejects_empty_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            report_path = Path(tmpdir) / "bench" / "profile_read_lengths" / "sample_0001" / "report.json"
            tool_dir = report_path.parent / "tools" / "seqkit_stats"
            tool_dir.mkdir(parents=True)
            report_path.write_text("{}", encoding="utf-8")
            (tool_dir / "profile_read_lengths_report.json").write_text(
                "{}",
                encoding="utf-8",
            )
            (tool_dir / "length_distribution.tsv").write_text("", encoding="utf-8")
            (tool_dir / "length_distribution.json").write_text(
                "{\"histogram\": []}",
                encoding="utf-8",
            )

            with self.assertRaises(SystemExit):
                profile_read_lengths_report.validate_artifact_paths(
                    report_path, "seqkit_stats"
                )

    def test_read_length_briefing_avoids_hardcoded_tool_name(self) -> None:
        summary = {
            "stage_id": "fastq.profile_read_lengths",
            "scenario_id": "read_length_fairness",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/results/corpus_01/fastq.profile_read_lengths/lunarc",
            "samples_total": 1,
            "samples_failed": 0,
            "tools": ["length_observer"],
            "report_only": True,
            "mutates_fastq": False,
            "may_change_read_count": False,
            "raw_backend_report_format": "seqkit_stats_length_histogram",
            "histogram_bins": 100,
            "era_counts": {"ancient": 1, "modern": 0},
            "layout_counts": {"se": 1, "pe": 0},
        }
        rows = [
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "ancient",
                "layout": "se",
                "size_band": "under_100mb",
                "study_accession": "PRJ1",
                "tool": "length_observer",
                "runtime_s": "1.0",
                "exit_code": "0",
                "read_count": "100",
                "mean_read_length": "50.0",
                "max_read_length": "75",
                "distinct_lengths": "12",
                "report_json_artifact": "/tmp/profile_read_lengths_report.json",
                "length_distribution_tsv_artifact": "/tmp/length_distribution.tsv",
                "length_distribution_json_artifact": "/tmp/length_distribution.json",
            }
        ]
        runtime_rows = profile_read_lengths_briefing.tool_runtime_summary(rows)
        cohort_rows = profile_read_lengths_briefing.cohort_runtime_summary(rows)
        outliers = profile_read_lengths_briefing.sample_runtime_outliers(rows)

        markdown = profile_read_lengths_briefing.render_markdown(
            summary, rows, runtime_rows, cohort_rows, outliers
        )

        self.assertIn("`length_observer` ran at", markdown)
        self.assertNotIn("`seqkit_stats` ran at", markdown)
        self.assertIn("Governed artifacts per sample/tool", markdown)

    def test_read_length_briefing_rejects_contract_drift(self) -> None:
        with self.assertRaises(SystemExit):
            profile_read_lengths_briefing.validate_summary_contract(
                {
                    "stage_id": "fastq.profile_read_lengths",
                    "scenario_id": "read_length_fairness",
                    "tools": ["seqkit_stats"],
                    "report_only": True,
                    "mutates_fastq": False,
                    "may_change_read_count": False,
                    "raw_backend_report_format": "wrong",
                    "histogram_bins": 100,
                }
            )

    def test_read_length_briefing_rejects_artifact_row_drift(self) -> None:
        with self.assertRaises(SystemExit):
            profile_read_lengths_briefing.validate_rows_contract(
                {"tools": ["seqkit_stats"]},
                rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "seqkit_stats",
                        "distinct_lengths": "10",
                        "report_json_artifact": "/tmp/not-right.json",
                        "length_distribution_tsv_artifact": "/tmp/length_distribution.tsv",
                        "length_distribution_json_artifact": "/tmp/length_distribution.json",
                    }
                ],
            )


class TerminalDamageReportingTests(unittest.TestCase):
    def test_terminal_damage_summary_tracks_runtime_and_asymmetry(self) -> None:
        rows = [
            {
                "tool": "cutadapt",
                "runtime_s": "0.8",
                "exit_code": "0",
                "base_retention": "0.95",
                "asymmetry_reduction": "0.25",
                "mean_q_delta": "0.30",
            },
            {
                "tool": "cutadapt",
                "runtime_s": "1.0",
                "exit_code": "0",
                "base_retention": "0.93",
                "asymmetry_reduction": "0.20",
                "mean_q_delta": "0.20",
            },
            {
                "tool": "seqkit",
                "runtime_s": "1.6",
                "exit_code": "0",
                "base_retention": "0.97",
                "asymmetry_reduction": "0.05",
                "mean_q_delta": "0.10",
            },
            {
                "tool": "seqkit",
                "runtime_s": "1.8",
                "exit_code": "0",
                "base_retention": "0.96",
                "asymmetry_reduction": "0.04",
                "mean_q_delta": "0.10",
            },
        ]

        summary_rows = terminal_damage_briefing.tool_runtime_summary(rows)
        by_tool = {row["tool"]: row for row in summary_rows}

        self.assertAlmostEqual(by_tool["cutadapt"]["median_runtime_s"], 0.9)
        self.assertAlmostEqual(by_tool["cutadapt"]["mean_asymmetry_reduction"], 0.225)
        self.assertGreater(
            by_tool["seqkit"]["median_base_retention"],
            by_tool["cutadapt"]["median_base_retention"],
        )

    def test_terminal_damage_markdown_mentions_damage_policy(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/results/corpus_01/fastq.trim_terminal_damage/lunarc",
            "scenario_id": "terminal_damage_fairness",
            "samples_total": 20,
            "samples_failed": 0,
            "tools": ["adapterremoval", "cutadapt", "seqkit"],
            "damage_mode": "ancient",
            "execution_policy": "explicit_terminal_trim",
            "trim_5p_bases": 2,
            "trim_3p_bases": 2,
            "era_counts": {"ancient": 10, "modern": 10},
            "layout_counts": {"se": 10, "pe": 10},
            "cohort_counts": {"ancient_pe": 5, "ancient_se": 5, "modern_pe": 5, "modern_se": 5},
            "headline": {
                "fastest_tool": "cutadapt",
                "fastest_runtime_s": 0.9,
                "best_base_retention_tool": "seqkit",
                "best_base_retention": 0.965,
                "largest_asymmetry_reduction_tool": "cutadapt",
                "largest_asymmetry_reduction": 0.225,
            },
            "tool_summary": [
                {
                    "tool": "cutadapt",
                    "records": 20,
                    "pass_rate": 1.0,
                    "median_runtime_s": 0.9,
                    "median_base_retention": 0.94,
                    "mean_asymmetry_reduction": 0.225,
                    "mean_q_delta": 0.25,
                }
            ],
        }

        markdown = terminal_damage_report.render_markdown(summary)

        self.assertIn("execution_policy: `explicit_terminal_trim`", markdown)
        self.assertIn("Mean asymmetry reduction", markdown)

    def test_terminal_damage_briefing_uses_dynamic_tool_and_sample_counts(self) -> None:
        summary = {
            "platform": "lunarc-apptainer",
            "stage_id": "fastq.trim_terminal_damage",
            "scenario_id": "terminal_damage_fairness",
            "samples_total": 2,
            "tools": ["cutadapt", "seqkit"],
            "damage_mode": "ancient",
            "execution_policy": "explicit_terminal_trim",
            "trim_5p_bases": 2,
            "trim_3p_bases": 2,
        }
        rows = [
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "modern",
                "layout": "pe",
                "size_band": "under_100mb",
                "study_accession": "PRJ1",
                "tool": "cutadapt",
                "runtime_s": "1.0",
                "exit_code": "0",
                "base_retention": "0.95",
                "asymmetry_reduction": "0.20",
                "mean_q_delta": "0.2",
            },
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "modern",
                "layout": "pe",
                "size_band": "under_100mb",
                "study_accession": "PRJ1",
                "tool": "seqkit",
                "runtime_s": "2.0",
                "exit_code": "0",
                "base_retention": "0.96",
                "asymmetry_reduction": "0.10",
                "mean_q_delta": "0.1",
            },
        ]
        runtime_rows = terminal_damage_briefing.tool_runtime_summary(rows)
        cohort_rows = []
        outliers = terminal_damage_briefing.sample_runtime_outliers(rows)

        markdown = terminal_damage_briefing.render_markdown(
            summary,
            rows,
            runtime_rows,
            cohort_rows,
            outliers,
        )

        self.assertIn(
            "`2` governed terminal-damage backends were benchmarked across `2` samples",
            markdown,
        )

    def test_terminal_damage_report_contract_rejects_policy_drift(self) -> None:
        run_manifest = {
            "tools": ["adapterremoval", "cutadapt", "seqkit"],
            "damage_mode": "ancient",
            "execution_policy": "explicit_terminal_trim",
            "trim_5p_bases": 2,
            "trim_3p_bases": 2,
            "requested_trim_5p_bases": 2,
            "requested_trim_3p_bases": 2,
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "adapterremoval",
                "raw_backend_report_format": None,
                "damage_mode": "ancient",
                "execution_policy": "explicit_terminal_trim",
                "trim_5p_bases": 2,
                "trim_3p_bases": 2,
                "requested_trim_5p_bases": 2,
                "requested_trim_3p_bases": 2,
            },
            {
                "sample_id": "sample_0001",
                "tool": "cutadapt",
                "raw_backend_report_format": "cutadapt_json",
                "damage_mode": "ancient",
                "execution_policy": "explicit_terminal_trim",
                "trim_5p_bases": 1,
                "trim_3p_bases": 2,
                "requested_trim_5p_bases": 2,
                "requested_trim_3p_bases": 2,
            },
            {
                "sample_id": "sample_0001",
                "tool": "seqkit",
                "raw_backend_report_format": None,
                "damage_mode": "ancient",
                "execution_policy": "explicit_terminal_trim",
                "trim_5p_bases": 2,
                "trim_3p_bases": 2,
                "requested_trim_5p_bases": 2,
                "requested_trim_3p_bases": 2,
            },
        ]

        with self.assertRaises(SystemExit):
            terminal_damage_report.validate_terminal_damage_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
            )

    def test_terminal_damage_report_rejects_dry_run_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            terminal_damage_report.validate_terminal_damage_run_manifest_contract(
                {
                    "stage_id": "fastq.trim_terminal_damage",
                    "scenario_id": "terminal_damage_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": True,
                }
            )

    def test_terminal_damage_summary_preserves_configured_corpus_root(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            run_root = (
                repo_root
                / "results"
                / "corpus_01"
                / "fastq.trim_terminal_damage"
                / "lunarc"
            )
            docs_root = (
                repo_root
                / "docs"
                / "benchmark"
                / "fastq.trim_terminal_damage"
                / "corpus-01"
            )
            sample_report = (
                run_root
                / "bench"
                / "trim_terminal_damage"
                / "sample_0001"
                / "report.json"
            )
            sample_report.parent.mkdir(parents=True)
            record_metrics = {
                "reads_in": 100,
                "reads_out": 98,
                "bases_in": 1000,
                "bases_out": 980,
                "delta_metrics": {
                    "base_retention": 0.98,
                    "read_retention": 0.98,
                    "mean_q_delta": 0.1,
                },
                "damage_mode": "ancient",
                "execution_policy": "explicit_terminal_trim",
                "trim_5p_bases": 2,
                "trim_3p_bases": 2,
                "requested_trim_5p_bases": 2,
                "requested_trim_3p_bases": 2,
                "ct_ga_asymmetry_pre": 0.30,
                "ct_ga_asymmetry_post": 0.10,
            }
            sample_report.write_text(
                json.dumps(
                    {
                        "records": [
                            {
                                "context": {"tool": "adapterremoval"},
                                "execution": {"runtime_s": 1.5, "exit_code": 0},
                                "metrics": {"metrics": record_metrics},
                            },
                            {
                                "context": {"tool": "cutadapt"},
                                "execution": {"runtime_s": 1.4, "exit_code": 0},
                                "metrics": {
                                    "metrics": {
                                        **record_metrics,
                                        "raw_backend_report_format": "cutadapt_json",
                                    }
                                },
                            },
                            {
                                "context": {"tool": "seqkit"},
                                "execution": {"runtime_s": 1.3, "exit_code": 0},
                                "metrics": {"metrics": record_metrics},
                            },
                        ]
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (run_root / "run_manifest.json").write_text(
                json.dumps(
                    {
                        "platform": "lunarc-apptainer",
                        "corpus_root": "/home/bijan/bijux/corpus_01",
                        "stage_id": "fastq.trim_terminal_damage",
                        "scenario_id": "terminal_damage_fairness",
                        "tool_kind": "benchmark",
                        "samples_total": 1,
                        "samples_failed": 0,
                        "tools": ["adapterremoval", "cutadapt", "seqkit"],
                        "damage_mode": "ancient",
                        "execution_policy": "explicit_terminal_trim",
                        "trim_5p_bases": 2,
                        "trim_3p_bases": 2,
                        "requested_trim_5p_bases": 2,
                        "requested_trim_3p_bases": 2,
                        "runs": [
                            {
                                "sample_id": "sample_0001",
                                "layout": "pe",
                                "report_json": str(sample_report),
                            }
                        ],
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            original_argv = sys.argv
            try:
                sys.argv = [
                    "render_fastq_trim_terminal_damage_corpus_01_report.py",
                    "--repo-root",
                    str(repo_root),
                    "--corpus-root",
                    "/home/bijan/lu2024-12-24/.cache/corpus_01",
                    "--run-root",
                    str(run_root),
                    "--docs-root",
                    str(docs_root.relative_to(repo_root)),
                ]
                with mock.patch.object(
                    terminal_damage_report,
                    "load_corpus_spec",
                    return_value={"corpus_id": "corpus-01"},
                ), mock.patch.object(
                    terminal_damage_report,
                    "load_sample_metadata",
                    return_value={
                        "sample_0001": {
                            "accession": "ACC1",
                            "era": "ancient",
                            "layout": "pe",
                            "study_accession": "PRJ1",
                            "size_band": "under_100mb",
                        }
                    },
                ):
                    self.assertEqual(terminal_damage_report.main(), 0)
            finally:
                sys.argv = original_argv

            summary = json.loads((docs_root / "summary.json").read_text(encoding="utf-8"))
            self.assertEqual(
                summary["corpus_root"],
                "/home/bijan/lu2024-12-24/.cache/corpus_01",
            )

    def test_terminal_damage_runner_parse_args_supports_sample_jobs(self) -> None:
        original_argv = sys.argv
        try:
            sys.argv = [
                "run_fastq_trim_terminal_damage_corpus_01.py",
                "--sample-jobs",
                "4",
            ]
            args = terminal_damage_runner.parse_args()
        finally:
            sys.argv = original_argv

        self.assertEqual(args.sample_jobs, 4)

    def test_terminal_damage_report_localizes_lunarc_report_paths(self) -> None:
        local_results_root = Path("/tmp/local-results")

        localized = terminal_damage_report.localize_results_path(
            "/home/bijan/bijux/results/corpus_01/fastq.trim_terminal_damage/lunarc/bench/trim_terminal_damage/sample_0008/report.json",
            local_results_root,
        )

        self.assertEqual(
            localized,
            local_results_root
            / "corpus_01/fastq.trim_terminal_damage/lunarc/bench/trim_terminal_damage/sample_0008/report.json",
        )

    def test_trim_polyg_report_contract_rejects_missing_tool_rows(self) -> None:
        run_manifest = {
            "tools": ["fastp", "bbduk"],
            "polyx_preset": "illumina_twocolor",
            "min_polyg_run": 10,
            "trim_polyg": True,
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "fastp",
                "raw_backend_report_format": "fastp_json",
                "polyx_preset": "illumina_twocolor",
                "min_polyg_run": 10,
                "trim_polyg": True,
            }
        ]

        with self.assertRaises(SystemExit):
            trim_polyg_report.validate_trim_polyg_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
            )


if __name__ == "__main__":
    unittest.main()
