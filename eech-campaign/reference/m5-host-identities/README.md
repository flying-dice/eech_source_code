# Host-issued identity prerequisite: retained evidence

This directory retains the bounded build and test record for the first product
source that requires an external host to issue opaque mobile identities and
keeps a durable campaign roster across membership transfer, physical loss and
entity-index reuse. It is a structural prerequisite for the M5 host boundary;
it does not complete any M5 criterion and does not establish host-owned loss or
arrival causality.

OpenAI Codex GPT implemented the identity change in
`74e040663e3ea3e0a9d1b5d9678f9303fa1f7250`. Its commit message records the
breaking API explicitly: Lua `dc.boot` requires `bind_mobile`, and the Rust/C
boot and frame calls require the corresponding host callback. The tested
executable product source is the clean merge
`5a3b0139ba6d66e4be6b778290025a3bff3dd6e8` of that identity commit and ABI
candidate `10ccad5b3a904562c08106d33b85bc1b90310141`. Codex performed the clean
combined build, tests and evidence retention; this does not transfer authorship
of the ABI correction.

The ABI candidate later merged as integration commit
`faac79024335db1772191dee2aac6284584c4a5d`. Its tree and the ABI candidate's
tree are both `362f3e215fed90aeaaf51aa51f73f337d43550e0`. A read-only
`git merge-tree --write-tree faac7902 74e04066` produces
`fc477956274f5fc8b8afb07df8be268a9f395e72`, exactly the tree of the tested
`5a3b0139` source. A rebuild for the metadata-only integration merge was
therefore neither needed nor performed.

## Exact build association

[BUILD-INFO.txt](BUILD-INFO.txt) records source `5a3b0139`, zero uncommitted
build inputs, immutable image `sha256:bb10e44a20e9c6304b3a318902bcf0f8ec8175c162862240ce25c34bd7e562be`,
Rust and Cargo 1.98.1, MinGW GCC 12-posix, and a fresh release
`x86_64-pc-windows-gnu` target. The build under test was
`target/host-identities-abi-final`.

- `eech_dc.dll`: `60cff8d96b97dfa820043db6b4aca136105e5514e5e70aa3de23a876e75f2057`
- `eech-world.exe`: `d9d1d8b76f66a35839908c1efb10e972619d8b3251b9f9ecd7c49e40af2c87cd`
- PUC Lua 5.1.5 `lua.dll`: `b224cb2eacdea06fcb7fe273979e660fbef0c1a9c65be53c7be9fc0a4571daca`
- copied DCS World `lua.dll` used only by standalone controls:
  `8a228b5437520bc8dda3e77aac7bb7f81c10d23f0747adbcbafe95647533b755`

The evidence-retention commit changes only this directory. It is not a build
input and does not replace the `5a3b0139` product-source association.

## Public contract and lifecycle results

The host callback returns a copied, 1–128 byte UTF-8 identity. Empty, NUL,
overlong, malformed UTF-8 and duplicate values fail. The same physical
incarnation retains its ID across MEMBER unlink/relink and group transfer. A
released incarnation remains in the append-only history, and reuse of the
physical entity index must bind a new identity. `initial_alive` is only the
internal fact captured at first link; it is not current state or a host loss
report. Existing object IDs, recordings, metrics, diagnostics and ACMI IDs are
unchanged.

The accepted lifecycle command was:

```text
pwsh -NoProfile -File ./tools/lifecycle-windows.ps1 -Root E:/eech-runs/m2/lebanon-root -Bin target/host-identities-abi-final -Out target/host-identities-abi-final-accepted-lifecycle
```

[Its summary](accepted-lifecycle.summary.txt) passes boot-twice, argument,
rejected-root, fatal-boot and no-reuse cases. The missing-map access violation
remains the accepted known defect.

The separate public identity lifecycle command was:

```text
pwsh -NoProfile -File ./tools/host-identities-lifecycle-windows.ps1 -Root E:/eech-runs/m2/lebanon-root -Bin target/host-identities-abi-final -Out target/host-identities-abi-final-host-lifecycle
```

[Its summary](host-lifecycle.summary.txt) additionally passes missing-host,
successful identity binding, nil/non-string/NUL/malformed-UTF-8/duplicate/error
callback cases and the callback re-entry matrix. Boot-time callbacks reject
nested boot and installation preparation; frame-time callbacks reject nested
members, roster statistics, objects, frame and installation preparation.
Callback errors return through Rust and Lua before the native fatal/poison
path, and later boot attempts cannot reuse partial C state.

The zero-hour public boot bound 3,432 members. [The retained summary](boot.summary.txt)
records 5,887,596 duplicate comparisons and 0.019 CPU seconds in the native
uniqueness scan. This is an observed normal-campaign cost, not a frame-budget
claim.

The bounded three-hour lifecycle audit used:

```text
pwsh -NoProfile -File ./tools/host-identities-windows.ps1 -Root E:/eech-runs/m2/lebanon-root -Bin target/host-identities-abi-final -Out target/host-identities-abi-final-audit-3h -Scenario lebanon_retail -Hours 3 -TargetEntity 87568 -RequireReuse -RequireTransfer -RequireTargetReleased -TimeoutSeconds 2400
```

[Its summary](audit-3h.summary.txt) records 3,769 bindings, 118 naturally
observed live-member transfers, four reused physical indices with distinct
identities, 338 durable removed records, and target 87568 retained as
`eech-world:3069` with `physical_present=false` and `in_group=false`. The
native uniqueness scan used 7,100,796 comparisons and 0.031 CPU seconds.

## ABI and M4 results

The standalone native-error controls ran six controls three times with each
Lua DLL:

```text
pwsh -NoProfile -File ./tools/native-error-controls/run-controls.ps1 -Bin target/host-identities-abi-final -LuaDll <PUC-or-copied-DCS-lua.dll> -Out <fresh-output> -Reps 3 -TimeoutSec 60
```

[PUC results](controls-puc.results.jsonl) and [summary](controls-puc.summary.txt)
are 18/18 pass. [DCS results](controls-dcs.results.jsonl) and
[summary](controls-dcs.summary.txt) are also 18/18 pass. Every attempt exited
`0x00000000`, reached its final marker and did not time out. These are
standalone CLI-host controls using a copied DCS Lua DLL, not an actual DCS
mission.

[pack.report.json](pack.report.json) and [pack.report.md](pack.report.md) are
the full retained and runtime M4 pack on the same combined binaries. The report
identifies source `5a3b0139`, the binary hashes above and unchanged
`windows-retail v1` baselines. All 40 checks pass: 17 directly protected, 12
baseline-sensitive, eight evidence-only and eight not covered. Georgia and
Lebanon are exact to their baselines; both Lebanon references are byte
identical in recordings, observations and metrics; the accepted supply and
loss chains pass. No baseline, tolerance, expectation or golden output changed.

The unchanged six-file loss structural proof also passes: nine `gp_loss.c`
helpers and 67 functions compare SAME to `163e9158`. Its self-test detects all
12 independent mutations and restores a passing candidate. `cargo fmt --all
-- --check` and `git diff --check` pass.

## Limits

The campaign DLL still owns the physical world, creation, movement, damage and
campaign consequences. Host identities are diagnostic correlation keys until
a real DCS adapter binds them; they are not DCS object identities. No actual
DCS mission ran. Loss and waypoint arrival still have internal authority, and
withholding a host observation cannot yet change campaign policy state. The
next approved slice moves the four SUPPLY-waypoint physical predicates to an
actual host callback while the campaign retains target, criteria, required
member and combination policy. Loss requires a later physical/campaign state
split that leaves physical death independent while withholding campaign loss
consequences and closes count, credit, task, supply and force-notification
bypasses.

## Retained file hashes

| File | SHA-256 |
|---|---|
| [BUILD-INFO.txt](BUILD-INFO.txt) | `c812b53c3f1442925f30cdfece0c75e0ae38d640fb8c2de6c7b41a41b567f026` |
| [accepted-lifecycle.summary.txt](accepted-lifecycle.summary.txt) | `55deeb742a1ce5f9bbcf756e93da52ee0f1ad899c8f09b6b0255476c6204907a` |
| [host-lifecycle.summary.txt](host-lifecycle.summary.txt) | `cad574d0293c258342fdd0cab9574188994a93d5467a26c7ecdd3dd59a667159` |
| [boot.summary.txt](boot.summary.txt) | `cf7b0cc09f9dc60d905e36181e06d88ebfafa9940d10c521efd81bef0c36d5d6` |
| [audit-3h.summary.txt](audit-3h.summary.txt) | `7f958a156d5cbaf88a2843b6d652ecc5612b2fc601b4eddb7fa0b07b7b92ef63` |
| [controls-puc.results.jsonl](controls-puc.results.jsonl) | `33ff2a80f9983f758d61d1183e849325f6e07837b602d4e5a19e607561c6718a` |
| [controls-puc.summary.txt](controls-puc.summary.txt) | `48035dc6fdd71ad8e26cf66515a8ea6e729e02fc29ee233e8d797a6fe291ec19` |
| [controls-dcs.results.jsonl](controls-dcs.results.jsonl) | `b5f8ef9c13d2d521c2b2add07e15c1d0778e85e0241ca4aaa55bf82bcedf6f59` |
| [controls-dcs.summary.txt](controls-dcs.summary.txt) | `46d71a1783f956aeb745b6076804a44ef2319b3d158387e34cdaefe9f1ea9fc5` |
| [pack.report.json](pack.report.json) | `88bf06177e1631a238b7957c9f27b026d7b627d4518d449a2cf4b25eec80ff78` |
| [pack.report.md](pack.report.md) | `400b8ee57a25312f7617055b900c2749733d938bca5fc36b26bf368c06ebef6e` |
