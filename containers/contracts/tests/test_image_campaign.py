from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest


CONTRACTS = Path(__file__).parents[1]
sys.path.insert(0, str(CONTRACTS))
SCRIPT = CONTRACTS / "image_campaign.py"
SPEC = importlib.util.spec_from_file_location("container_image_campaign", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
campaign = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(campaign)


class ImageCampaignTests(unittest.TestCase):
    def test_selection_contains_only_supported_noncurrent_images(self) -> None:
        selected = campaign.selected_tools(
            {
                "schema_version": campaign.INVENTORY_SCHEMA,
                "tools": [
                    {
                        "tool": "samtools",
                        "apptainer_supported": True,
                        "state": "current",
                    },
                    {
                        "tool": "bcftools",
                        "apptainer_supported": True,
                        "state": "stale_recipe",
                    },
                    {
                        "tool": "fastqc",
                        "apptainer_supported": True,
                        "state": "missing",
                    },
                    {
                        "tool": "impute5",
                        "apptainer_supported": False,
                        "state": "unsupported",
                    },
                ],
            }
        )

        self.assertEqual(selected, ["bcftools", "fastqc"])

    def test_campaign_requires_fixed_frontend_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "campaign.json"
            path.write_text(
                json.dumps(
                    {
                        "schema_version": campaign.CAMPAIGN_SCHEMA,
                        "run_id": "run-1000",
                        "runtime": "apptainer",
                        "site": "lunarc",
                        "selection": "catalog_apptainer_supported_noncurrent",
                    }
                ),
                encoding="utf-8",
            )

            loaded = campaign.load_campaign(path)

        self.assertEqual(loaded["run_id"], "run-1000")

    def test_other_run_identity_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "campaign.json"
            path.write_text(
                json.dumps(
                    {
                        "schema_version": campaign.CAMPAIGN_SCHEMA,
                        "run_id": "run-1001",
                        "runtime": "apptainer",
                        "site": "lunarc",
                        "selection": "catalog_apptainer_supported_noncurrent",
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(campaign.ContractError, "run-1000"):
                campaign.load_campaign(path)


if __name__ == "__main__":
    unittest.main()
