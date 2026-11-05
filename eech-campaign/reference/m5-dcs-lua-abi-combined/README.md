# Combined structural and DCS Lua ABI candidate: retained evidence

This directory retains the bounded build and test record for the first product source that combines the DCS `lua_Debug` correction with the reviewed M5 structural correction. It supplements the original ABI evidence in [m5-dcs-lua-abi](../m5-dcs-lua-abi/README.md); none of those 24 retained artifacts was replaced or modified.

The executable product source is `de4ab5d7f2a8e8a616cb64a38409d6dfff213543`, a clean merge of the ABI retention head `6bcfdce2768b73179f2f2c384d75b67a182c877d` and the reviewed structural integration `0ba0384ceb8e6c4c4b2647f149314bd0de0661ca`. The actual ABI product commit in that history remains `59a97bcabf132d7bd6be173267d33711ef9404bb`, authored by `fd-shockwave-bot` with Claude Sonnet 5.5 recorded as co-author. OpenAI Codex GPT performed the clean combined build, controls, pack, and evidence adoption. This evidence does not transfer authorship of the ABI experiment.

## Exact build association

[BUILD-INFO.txt](BUILD-INFO.txt) records a read-only `git archive` of `de4ab5d7f2a8e8a616cb64a38409d6dfff213543`, zero uncommitted build inputs, source archive SHA-256 `94a1a53d94bc2f9b4e332a8676d5c61c6f62ce6a9955160b84bf7d511f68ff3e`, and a genuinely fresh Cargo target volume `eech-win-target-combined-de4ab5d7`. The build used image `sha256:0c60c75db7b035e830c4834b1e15e096483631915bafc22c1af6aae02032bab3` by immutable ID, Rust 1.98.1, Cargo 1.98.1, MinGW GCC 12-posix, `cargo --locked`, and release `x86_64-pc-windows-gnu`.

- `eech_dc.dll`: `2d6a5490a00c6dee25303172aa892473f864f7493f8b06ae61fe59e0403b4bf0`
- `eech-world.exe`: `a37c1a319ca223b1f62bdba0876330460db8b8587026227e5d4d69bb5f4c276f`
- PUC Lua 5.1.5 `lua.dll`: `d0c799fe4e1e615d335d18504d28a7b0d7c78d7f97d1754bb885b1097ac60350`
- DCS World `lua.dll` used by the standalone controls: `8a228b5437520bc8dda3e77aac7bb7f81c10d23f0747adbcbafe95647533b755`

The later evidence-retention commit changes only documentation and copied output. It is not a build input and does not replace the `de4ab5d7` product-source association.

## Results and limits

The same standalone native-error control command used for the original ABI candidate ran six controls three times in fresh processes against each Lua DLL. [DCS results](controls-dcs.results.jsonl) and [summary](controls-dcs.summary.txt) are 18/18 pass; [PUC results](controls-puc.results.jsonl) and [summary](controls-puc.summary.txt) are 18/18 pass. All 36 attempts exit `0x00000000`, time out zero times, and record the combined DLL and host hashes above. These are standalone CLI-host controls, not an actual DCS mission.

[pack.report.json](pack.report.json) and [pack.report.md](pack.report.md) are the unchanged full M4 pack run on the same combined binaries. The report names exact source `de4ab5d7`, the same DLL and host hashes, both retained and runtime tiers, and `windows-retail v1` with its existing baseline hashes. It reports PASS with all 40 checks passing: 17 directly protected claims, 12 baseline-sensitive passes, 8 evidence-only claims, and 8 not covered. The runtime tier reports Georgia and Lebanon exact to their baselines, two byte-identical Lebanon reference recordings/observations/metrics, 31 complete supply chains of 32 deliveries in each reference run with zero unattributed deliveries, and the retained first in-flight SUPPLY loss chain passing. No baseline, tolerance, expectation, or golden output was changed.

The pack and controls support this exact combined candidate only. The risk-probe branch was not integrated into it. No corrected build has run inside a DCS mission, host separation remains unproved, and criterion 5 and M5 remain unaccepted. The borrowed PUC hook-record path remains unsupported; the product does not call `set_hook`.

## Files, sources, and hashes

The source run is `E:/eech-runs/dcs-probe/abi-microprobe/product-fix/combined-de4ab5d7f2a8e8a616cb64a38409d6dfff213543/` on the authoring machine. Every non-README file below is byte-identical to that run artifact.

| File | Original | SHA-256 |
|---|---|---|
| [BUILD-INFO.txt](BUILD-INFO.txt) | `build/bin/BUILD-INFO.txt` | `f85c39133bf5a14dcff221254a77b097d07bb685baf7eb08e60e86f318fd95f3` |
| [controls-dcs.results.jsonl](controls-dcs.results.jsonl) | `controls-dcs/results.jsonl` | `1fe665f1aaada3eaadc84d8888cfaf80caf5182d544319c193dce85bcb020074` |
| [controls-dcs.summary.txt](controls-dcs.summary.txt) | `controls-dcs/summary.txt` | `ba30ec73f8daa25aa9ad1e40ec46b250255d6f4f135d6719688258fc08b6f7ef` |
| [controls-puc.results.jsonl](controls-puc.results.jsonl) | `controls-puc/results.jsonl` | `3d2d77af569193db40a62753167c60752837f74fb43afae43cc0256c0ea1d94f` |
| [controls-puc.summary.txt](controls-puc.summary.txt) | `controls-puc/summary.txt` | `50d978632419e7a8d8d44694f83b358447815e52d68b5c1e0885f6cc506ece3c` |
| [pack.report.json](pack.report.json) | `pack/report.json` | `d67a3c439c110a023f6e0b2d1f3d6fa0f8f54c3921426f48195c930ce9f2f75e` |
| [pack.report.md](pack.report.md) | `pack/report.md` | `78b47d3ab058d40c99f0c388f73bebd47e6b52496f36b6366756887e7266efec` |
