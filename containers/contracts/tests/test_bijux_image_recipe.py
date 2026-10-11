from __future__ import annotations

import unittest
from pathlib import Path

import tomllib

ROOT = Path(__file__).parents[3]


class BijuxImageRecipeTests(unittest.TestCase):
    def test_recipe_carries_native_and_repository_build_inputs(self) -> None:
        recipe = (ROOT / "containers/apptainer/bijux_dna.def").read_text()

        self.assertIn("libsqlite3-dev", recipe)
        self.assertIn("assets /opt/bijux-src/assets", recipe)
        self.assertIn("configs /opt/bijux-src/configs", recipe)
        self.assertIn("domain /opt/bijux-src/domain", recipe)
        self.assertNotIn("benchmarks /opt/bijux-src/benchmarks", recipe)
        self.assertNotIn("/opt/bijux/benchmarks", recipe)

    def test_recipe_installs_binary_from_the_governed_cargo_target(self) -> None:
        cargo = tomllib.loads((ROOT / ".cargo/config.toml").read_text())
        target = cargo["build"]["target-dir"]
        recipe = (ROOT / "containers/apptainer/bijux_dna.def").read_text()

        self.assertIn(f"install -Dm0755 {target}/release/bijux-dna", recipe)


if __name__ == "__main__":
    unittest.main()
