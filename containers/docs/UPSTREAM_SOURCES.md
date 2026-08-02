# Non-Bijux Apptainer Sources

Purpose: Track upstream provenance for every non-bijux definition under `containers/apptainer/`.

Classification contract:
- `non-bijux` = upstream-derived recipe ownership/provenance (minimal packaging adaptation only).
- `bijux` = Bijux-owned recipe lifecycle in this repo.
- This split is not a license override mechanism; upstream license terms remain authoritative.

| tool_id | apptainer_def | why_non_bijux | upstream_source | upstream_license | upstream_checksum | patching_rules |
|---|---|---|---|---|---|---|
| `beagle` | `containers/apptainer/beagle.def` | upstream recipe retained to minimize divergence from vendor build process | https://faculty.washington.edu/browning/beagle/beagle.html | `GPL-3.0` | `sha256:938f0b1ab12385e0686790cef52d7b9491c96c0c1837af5c0d62c9a6576a8956` | only compatibility/packaging patches; no algorithmic behavior changes |
| `bcftools` | `containers/apptainer/bcftools.def` | upstream recipe retained to minimize divergence from vendor build process | https://github.com/samtools/bcftools | `MIT` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `eigensoft` | `containers/apptainer/eigensoft.def` | upstream recipe retained to minimize divergence from vendor build process | https://github.com/DReichLab/EIG | `GPL-2.0` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `germline` | `containers/apptainer/germline.def` | upstream recipe retained to minimize divergence from vendor build process | https://github.com/gusevlab/germline/tree/f8029fcf469a09a83aa1e0d1fb45769f6dcb3c6d | `Apache-2.0` | `sha256:42a6e14e003ac6aa5cc5c3b14db4ba355d748c014b0d0a10f0f734dca2bfb991` | only compatibility/packaging patches; no algorithmic behavior changes |
| `glimpse` | `containers/apptainer/glimpse.def` | upstream recipe retained to minimize divergence from vendor build process | https://odelaneau.github.io/GLIMPSE/ | `MIT` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `shapeit5` | `containers/apptainer/shapeit5.def` | upstream recipe retained to minimize divergence from vendor build process | https://odelaneau.github.io/shapeit5/ | `MIT` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `eagle` | `containers/apptainer/eagle.def` | upstream recipe retained to minimize divergence from vendor build process | https://alkesgroup.broadinstitute.org/Eagle/ | `GPL-3.0` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `minimac4` | `containers/apptainer/minimac4.def` | upstream recipe retained to minimize divergence from vendor build process | https://genome.sph.umich.edu/wiki/Minimac4 | `MIT` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
| `ibdhap` | `containers/apptainer/ibdhap.def` | verified MORGAN source is built with an explicit VCF-to-parameter adapter | https://sites.stat.washington.edu/thompson/Genepi/MORGAN/Morgan.shtml | `LicenseRef-MORGAN-2006` | `sha256:195a84d877413a83de15a20da45d92d2441303b8373b47727ed2e3ab92dc2e6a` | preserve the upstream IBD-state model; adapter changes require real-panel qualification |
| `ibdne` | `containers/apptainer/ibdne.def` | upstream recipe retained to minimize divergence from vendor build process | https://faculty.washington.edu/browning/ibdne.shtml | `GPL-3.0` | `sha256:pending` | only compatibility/packaging patches; no algorithmic behavior changes |
