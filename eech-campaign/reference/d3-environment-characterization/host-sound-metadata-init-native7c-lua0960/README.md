# Host-owned sample metadata at initialization: finite evidence

Scope: the **init-time** sample metadata and header lookup only (EEDCS-51, D3, PR111). The host now owns the asset
rates, where the sound files are, the header parse and the lookup; the campaign names its descriptors once and
writes its live catalogue only after the whole answer is checked. This folder is the independently reviewed
evidence for that finite slice. **Task51 and D3 remain open; D4/D5 have not started.** It is not Tacview, visual,
audio, shipping or normal-delivery evidence.

## Identities

| Item | Identity |
|---|---|
| Native production source (immutable) | `7c85100efb1e1aa4669f6c88051f9adce041629c` |
| External Lua (frozen) | `0960e22d1fc4cd6bf95373f839b5c49c72ec27d0` |
| Later test-only source | `0e25c5e00ee7e9aae41b4e97582c4d12a4681ac0` (only `mod tests` of `eech-engine/src/lib.rs`, for two clippy lints) |
| Publication | the commit that adds this folder; parent `0e25c5e00` |
| Old capability native (expected red) | `7574caa2ca87a2a44608a234fd1dd73136b8e4c0`, 62 payloads |
| New native payloads | 64; `BUILD_INFO` `9dac187b544458c2da527cc181798246f7999dd2c2e1f9f82cfa4952b4d84d35` (old `7824726195b658c3849d2f3872f88e4522923c0f182e3160fb6b8174ff51f603`) |

**The 7c native is not rebuilt at later commits and is never relabelled.** The 0960 Lua and 0e25 tests ran against
the frozen 7c binaries; the 0e25 change is test code, so no native rebuild was needed or done.

## Results (actual)

- **13 actual native executions**: 3 default controls (probe, observation-off, header-present) + 10 authority
  executions covering **8 unique cases** (healthy absent, healthy control, changed, withheld, malformed, short,
  incoherent, old-native capability red). All 26 pre/post retail-input checks exit 0. 12 new-native executions and
  controllers exit 0; the old 7574 native with its ORIGINAL delegate exits 1 with `sound-data-host-capability-missing`
  and 0 requests, which is the intended red and not a candidate success.
- **Observation-off**: complete and byte/ending identical to the original recording: 5191384 bytes, 17283 rows,
  SHA256 `48307ff0286cff2219a89d7d8b807a3624fa1cb673c440c4ed70d55a7b40d81d`.
- **Metadata report**: whole-report equality is **false by design** (a new host-answer mirror row; the original Core
  file attempts and open/match events and private header count are 0; raw-report path/length and derived keys change by declared
  provenance). 85 (probe) and 70 (header-present) complete stable lines are byte-identical, and every excluded row
  and field was classified by the review. Header-present: the same declared synthetic pair gives 7 entries, 4
  matches, 2065 misses and the original catalogue/constructor state. The host reply is a mirror, not a file operation of the
  campaign; no OS-level host IO trace is claimed.
- **Authority**: healthy digests `6605372942e1ea2c` (absent) and `8c600fed7309b2fc` (control; lightning size 96,
  thunder matched at 0). Changed: lightning 44100/4321/22050 and thunder 8000/0/8000 (rate/size/original rate),
  digest `80bbbccaa4bbb061`. Withheld records one intentional withdrawal with 0 base errors; malformed, short and
  incoherent stop at the named answer marker; each refusal's out file and sidecar were read back and are empty.
- **Builds and tests**: Windows 7c observer build 0; 64 payloads unchanged. Linux default and observer strict at
  0e25: 0 (06:08:18Z), Engine 94 tests. Windows Engine/DC and four selected integrations 0. Windows World: first run
  291 passed + 1 setup failure (compile-time source path missing, retained), then the SAME immutable executable
  (SHA256 `52c8ddadad032cd2faa369222a07297eedfe5634bc12847bb840efecd7aea0f6`) passed 292/292 with the exact 7c
  `snd_data.c` fixture at `R:/src`. All nine exact-0e25 CI jobs (three workflows) succeeded (`ci-0e25.json`).

## Preserved failures and a correction

- Initial 7c Linux pipeline: 18 passed, 2 new test assertions failed; later three test expectations were corrected
  for mlua's `runtime error:` envelope; 0960 strict failed on two test-only lints (fixed at 0e25). Nothing was
  weakened.
- **Bookkeeping correction (Jira 10917; `authority-bookkeeping-correction.json`).** The first authority controller
  completed **six** cases (healthy absent, healthy control, changed, withheld, malformed, short) and then stopped on a
  space guard; the remaining controller ran malformed and short again, then incoherent and the old-native case. An
  earlier summary (and the remaining controller's retained `prior_completed_cases` text, Jira 10916) said four
  completed and did not rerun any: that was wrong. The retained originals are not rewritten; the correction supersedes
  them, duplicates add no coverage, and there are 10 executions of 8 unique cases.

## Still required (not closed here)

Dynamic sample loaders and the backend/device, assets and blocked-sample state; the other compiled consumers of the
descriptor table; non-session and per-family range; playback handles; independent destroy; network; pitch and
amplitude; physical time; and the Task49/50 services. No malformed/reload/all-file conditions were run. CPU
floating-mode preservation is not inferred from unchanged fingerprints. A later extraction is not part of this folder.

## Reproduction and retained private paths

Private evidence lives under `R:/d3-evidence/` (large inputs and closed snapshots have since moved to
`D:/EECH-Migration-Evidence`, original junction paths preserved; the failed all-file compression audit of the old
sources hit ENOSPC and no all-file before/after hash success is claimed). Key groups: `metadata-default-controls-7c-native-0960-lua-20261101`,
`metadata-public-authority-0960-20261101`, `metadata-public-authority-remaining-0960-20261101`,
`metadata-linux-final-overlay-0960-20261101`, `metadata-linux-test-lints-overlay-0e25-20261101`,
`metadata-windows-tests-0960-20261101`, and the runner/audit scripts `run-host-sound-metadata-control-0960-20261101.py`
(SHA256 `72ed01bef9dff4e2df6cc69b0d7d57d9af0da56a48cfe877ba0130db77aad009`) and
`audit-metadata-default-controls-0960-20261101.py` (SHA256 `9d0a843c91aad88ca8f28517bfb7a8146309af74db68f1f9d26c07ff6891ff41`).
To re-run a case: boot the frozen 7c native with the 0960 `lua/env-sound-data-host-witness.lua` (delegate
`lua/env-sound-init-host.lua`; the ORIGINAL `lua/env-sound-init.lua` as delegate for the old 7574 capability red) through the runner. No raw recordings, audio
assets, scenario roots or binaries are stored in Git.

## Source map (literal byte copies, no normalization)

The JSON files are byte copies of the private receipts; three of them (`default-comparison.json`, `linux-final-checks.json`, `windows-world-retest.json`) have CRLF line endings, which `.gitattributes` (`*.json -text`) preserves in Git, so `git diff --check` reports trailing whitespace for them by design. The Git blob SHA256 of each equals the value below.

| Private source (under R:/d3-evidence/) | Published alias | Bytes | SHA256 |
|---|---|---:|---|
| `independent-environment-host-sound-metadata-final-actual-review-native7c-lua0960-20261101.json` | `actual-review.json` | 31368 | `44f1440fc6a0ff89b3a2b09454159a4a2e755c526614d67891a36e4fa833af38` |
| `metadata-default-controls-7c-native-0960-lua-20261101/comparison.json` | `default-comparison.json` | 3792 | `ea826f86b2cd784c2858e84420394dfbd4e918ccabba422ee85e5d01cf9844eb` |
| `metadata-authority-execution-bookkeeping-correction-20261101.json` | `authority-bookkeeping-correction.json` | 1735 | `ba1264ae3521c93203c15397433d3f97b82f72ee678a6263a2ce5925859fee97` |
| `independent-environment-host-sound-metadata-corrected-source-review-7c85100ef-20261101.json` | `production-source-review.json` | 15275 | `ce3ca963fb7c3b265e1cdb1e1825d70989f38a71d496a0767ea7052966463e47` |
| `independent-environment-host-sound-metadata-witness-delta-review-7130828b2-20261101.json` | `witness-totals-review.json` | 8225 | `59504eb8d95291c73fb0e2e2d06e970d973f93d534422565a1754045317b8c11` |
| `independent-environment-host-sound-metadata-witness-delta-review-0960e22d1-20261101.json` | `witness-final-review.json` | 6327 | `1958962e08ddb2fe0c9bf5e19ba352147e7effd996f568d24878aa4503819241` |
| `independent-environment-host-sound-metadata-test-lint-source-review-0e25c5e00-20261101.json` | `test-lint-review.json` | 4530 | `1e4691bdafbaa80ae4cd25af5db77859f47acd23e527bedf0deaff09f59b31d0` |
| `current-CI-0e25c5e00-20261101.json` | `ci-0e25.json` | 14458 | `cf62dbe8daaee295c8b4eb606115d9d040af5f15510a7c520f50d7b413ce7ac9` |
| `metadata-linux-test-lints-overlay-0e25-20261101/controller.json` | `linux-final-checks.json` | 4487 | `49ba9b7f4067d5b3a3fc7cff1aabc433bddd513dcae45c914c250adcee2880a4` |
| `metadata-windows-world-source-context-retest-0960-20261101/controller.json` | `windows-world-retest.json` | 1055 | `902a9c5a4f2e3acb50be6d2bbb4ba17532cd5efbd47b76acf07c072b97ca78df` |

The complete private recordings, sidecars and per-case controllers named in `actual-review.json` (`actual_controls`)
are retained privately with their SHA256 values listed there.
