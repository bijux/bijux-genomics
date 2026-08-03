# LUNARC Apptainer canonical storage evidence

Purpose: record the reversible removal of unreferenced obsolete SIFs from the active canonical image root.

## Verdict

- Verdict: **passed**
- Repository commit: `32a447d03b36aeb617de08fda69f8b9fb1ff0f56`
- Quarantined files: **35**
- Quarantined bytes: **6107336704**
- Historical run dependencies preserved: **3**

No SIF was deleted. Every move remained on the governed filesystem and is recoverable through the checksummed source plan after target digests are revalidated.

## Acceptance checks

- PASS: `plan_applied`
- PASS: `all_moves_applied`
- PASS: `move_count_matches`
- PASS: `move_size_matches`
- PASS: `only_current_or_unsupported_tools_remain`
- PASS: `no_unknown_images`
- PASS: `no_staged_images`
- PASS: `no_unreferenced_obsolete_images`

## Quarantined images

| Image | Category | Size (bytes) | SHA-256 | Quarantine path |
|---|---|---:|---|---|
| `alientrimmer-3.2-linux-amd64-1a894b95acf4.sif` | `obsolete` | 177463296 | `1a894b95acf427105c8c93d2475c97dd047bbc63f85995c0b9016100d2a16f85` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/alientrimmer-3.2-linux-amd64-1a894b95acf4.sif` |
| `bbduk-39.06-linux-amd64-1a7dc6a90e7b.sif` | `obsolete` | 116957184 | `1a7dc6a90e7b4778a2ff7267f4701be9c916008c1ebbf505361dbbca8d0e40b8` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/bbduk-39.06-linux-amd64-1a7dc6a90e7b.sif` |
| `bbmerge-39.06-linux-amd64-8d7eac3a1965.sif` | `obsolete` | 116957184 | `8d7eac3a1965b71489d6c24638cfa14f9b1509ff7333d200f613dd0cdea69909` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/bbmerge-39.06-linux-amd64-8d7eac3a1965.sif` |
| `bcftools-1.20-linux-amd64-6f7cdc3f6e83.sif` | `obsolete` | 63156224 | `6f7cdc3f6e830f69c44fdc164c140ce8e82b52eecb14dc1e237fdd04a22f0f99` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/bcftools-1.20-linux-amd64-6f7cdc3f6e83.sif` |
| `bijux_dna-0.1.0-linux-amd64-a1f974c28c92.sif` | `obsolete` | 460128256 | `a1f974c28c927b5f4c695d2d2c24c28c7187974b32477c74ad20d1237e83a486` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/bijux_dna-0.1.0-linux-amd64-a1f974c28c92.sif` |
| `bijux_dna-0.1.0-linux-amd64-dc55c564c575.sif` | `obsolete` | 459919360 | `dc55c564c575190ffb94975f4099e154021819ef212f30604f977ea4f367c1fd` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/bijux_dna-0.1.0-linux-amd64-dc55c564c575.sif` |
| `bwa-0.7.17-linux-amd64-a12a4e24c6cf.sif` | `obsolete` | 103317504 | `a12a4e24c6cf9bcee14e51b7d8e31873ac84dd11eaf8287a6fab688be7b72f35` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/bwa-0.7.17-linux-amd64-a12a4e24c6cf.sif` |
| `clumpify-39.06-linux-amd64-14c6a7de5caa.sif` | `obsolete` | 116957184 | `14c6a7de5caa198c2fb35f97bc0fe4ec0ee79c4480d8d298159024e3f4758855` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/clumpify-39.06-linux-amd64-14c6a7de5caa.sif` |
| `cutadapt-4.9-linux-amd64-6da7c39ddb36.sif` | `obsolete` | 70537216 | `6da7c39ddb3601e13a66a3a93ca6a9b2d5ab4738677b161b78cde95296413439` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/cutadapt-4.9-linux-amd64-6da7c39ddb36.sif` |
| `fastq_screen-0.15.3-linux-amd64-a7cc81909148.sif` | `obsolete` | 46452736 | `a7cc819091480d041ccf4d699bbbefb728ba02b4d20f32f78a81f9e154030cef` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/fastq_screen-0.15.3-linux-amd64-a7cc81909148.sif` |
| `fastx_clipper-0.0.14-linux-amd64-bc54cc233692.sif` | `obsolete` | 31559680 | `bc54cc2336929e04879aed99e2a4eb4fdd25c824e760b9d49d7ad7a3e3c16aeb` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/fastx_clipper-0.0.14-linux-amd64-bc54cc233692.sif` |
| `fqtools-2.3-linux-amd64-ade0872ac698.sif` | `obsolete` | 111202304 | `ade0872ac69837cd2d080947250ab41ccbba02d99dc11193520937de519b3188` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/fqtools-2.3-linux-amd64-ade0872ac698.sif` |
| `gatk-4.6.2.0-linux-amd64-9057f042ae58.sif` | `obsolete` | 806256640 | `9057f042ae588d62f5b8539bdbbc8daa44ae1c6ad6fe0200944bd228b21de643` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/gatk-4.6.2.0-linux-amd64-9057f042ae58.sif` |
| `germline-1.5.3-linux-amd64-7c9bc32127d9.sif` | `obsolete` | 30781440 | `7c9bc32127d93a73f5c881f5e095e36e38cc902ab72e3d77b42901e68a597403` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/germline-1.5.3-linux-amd64-7c9bc32127d9.sif` |
| `glimpse-1.1.1-linux-amd64-0fac06ffca29.sif` | `obsolete` | 194019328 | `0fac06ffca298d98a774f0532c42c3582a331766a0521fd20fd673493e3471fe` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/glimpse-1.1.1-linux-amd64-0fac06ffca29.sif` |
| `ibdhap-3.4-linux-amd64-b0ac1b0a58e4.sif` | `obsolete` | 43503616 | `b0ac1b0a58e417152c02a87b95b647985c78aaa1067ef1da66f1c74d6509e032` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/ibdhap-3.4-linux-amd64-b0ac1b0a58e4.sif` |
| `kaiju-1.10.0-linux-amd64-89985e105b54.sif` | `obsolete` | 33050624 | `89985e105b5419a6c5b179f8f5d19ac76a7b772af31144fc5123952bb4b758df` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/kaiju-1.10.0-linux-amd64-89985e105b54.sif` |
| `king-2.3.0-linux-amd64-1dd81b328b10.sif` | `obsolete` | 34893824 | `1dd81b328b10094962cd891b1c7227c636ebd7f16567d5bb51ada4aedf683a01` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/king-2.3.0-linux-amd64-1dd81b328b10.sif` |
| `krakenuniq-1.0.4-linux-amd64-2d1206ab18f7.sif` | `obsolete` | 55177216 | `2d1206ab18f7f21df3c3e8690c78e2d9ac5b3d69976198521e2900f26d14dd91` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/krakenuniq-1.0.4-linux-amd64-2d1206ab18f7.sif` |
| `lighter-1.1.3-linux-amd64-4daa759e5a91.sif` | `obsolete` | 30830592 | `4daa759e5a917db8c6111002b923549c5858892a2f53d63bdb38788f1d714125` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/lighter-1.1.3-linux-amd64-4daa759e5a91.sif` |
| `metaphlan-4.1.1-linux-amd64-25bcb1fe0112.sif` | `obsolete` | 348401664 | `25bcb1fe01124c279060e3a652a20dfadcc0107df2695b949a8c24d94e51d650` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/metaphlan-4.1.1-linux-amd64-25bcb1fe0112.sif` |
| `minimac4-4.1.6-linux-amd64-9ced0aebf2a6.sif` | `obsolete` | 168591360 | `9ced0aebf2a63337f006c4433b4da05519530d477f5be1d4f529633fbe40239f` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/minimac4-4.1.6-linux-amd64-9ced0aebf2a6.sif` |
| `picard-3.3.0-linux-amd64-4a87eeff8ac4.sif` | `obsolete` | 160620544 | `4a87eeff8ac488096d923b640a40bc1b0c605422e094ac25fe32b8dc93b1cf84` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/picard-3.3.0-linux-amd64-4a87eeff8ac4.sif` |
| `picard-3.3.0-linux-amd64-a69cf8855519.sif` | `obsolete` | 160616448 | `a69cf8855519ad4fd232e6e12d274c96128591be48652d715b8b10d1c81577ba` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/picard-3.3.0-linux-amd64-a69cf8855519.sif` |
| `rcorrector-1.0.7-linux-amd64-413c8b1fb329.sif` | `obsolete` | 142594048 | `413c8b1fb329e34209ba0a48bee3b83e10c072a82384101299671e3c149b3e54` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/rcorrector-1.0.7-linux-amd64-413c8b1fb329.sif` |
| `rxy-2.0.0-linux-amd64-8dbfa75e2892.sif` | `obsolete` | 46592000 | `8dbfa75e28927582f0e2831b9906a9f8eb61ac168396c886dea72699f8cc88d7` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/rxy-2.0.0-linux-amd64-8dbfa75e2892.sif` |
| `schmutzi-1.5.4-linux-amd64-f8c686d1be5c.sif` | `obsolete` | 163659776 | `f8c686d1be5c921ce25b4af653bd4ecdbb7244aec3842ddf30fb28010bfc0a49` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/schmutzi-1.5.4-linux-amd64-f8c686d1be5c.sif` |
| `sortmerna-4.3.7-linux-amd64-21a8905e7c3c.sif` | `obsolete` | 34775040 | `21a8905e7c3ced815a5f366084e30836308a91fe8885ba689005eb2db32571ab` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/sortmerna-4.3.7-linux-amd64-21a8905e7c3c.sif` |
| `star-2.7.11b-linux-amd64-4f9df61e0f60.sif` | `obsolete` | 32333824 | `4f9df61e0f60c62520038b3008aed9413e84471a70cd85030e757b49223a6e4c` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/star-2.7.11b-linux-amd64-4f9df61e0f60.sif` |
| `trim_galore-0.6.10-linux-amd64-84a8667dc18e.sif` | `obsolete` | 368676864 | `84a8667dc18e25d2f99d11dd90b738caf0da2dc0e34ab64e42eb46381138d391` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/trim_galore-0.6.10-linux-amd64-84a8667dc18e.sif` |
| `trim_galore-0.6.10-linux-amd64-ac660f35441a.sif` | `obsolete` | 368676864 | `ac660f35441ac810d6ad3f2716739cf07c4b4077059d7d08b3bfdbda7bbb3cd6` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/trim_galore-0.6.10-linux-amd64-ac660f35441a.sif` |
| `trimmomatic-0.39-linux-amd64-b6759fd48872.sif` | `obsolete` | 99295232 | `b6759fd4887276a68ecc5c44723da3de4a1536e7145c7ee4f30a20e7ba5cd207` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/trimmomatic-0.39-linux-amd64-b6759fd48872.sif` |
| `umi_tools-1.1.6-linux-amd64-3c790b90bbb2.sif` | `obsolete` | 325095424 | `3c790b90bbb21a0016f44ce82667006871af398ca6add0db350cda169532d92e` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/umi_tools-1.1.6-linux-amd64-3c790b90bbb2.sif` |
| `verifybamid2-2.0.1-linux-amd64-a9208f6c433d.sif` | `obsolete` | 315506688 | `a9208f6c433d5c878c10047a6ac53dbffa6608f8a8d392be18d1b603ff3122f5` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/verifybamid2-2.0.1-linux-amd64-a9208f6c433d.sif` |
| `yleaf-4.1.4-linux-amd64-0b0a95d2e78a.sif` | `obsolete` | 268779520 | `0b0a95d2e78a186976eb3e1622cb49221422d700a7b9cf7f4c6c772811e01c4a` | `containers/apptainer/.quarantine/32a447d03b36aeb617de08fda69f8b9fb1ff0f56/obsolete/yleaf-4.1.4-linux-amd64-0b0a95d2e78a.sif` |
