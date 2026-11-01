# Host-owned sound backend availability at initialization: finite evidence

Scope: the **initialization availability decision** only (EEDCS-51, D3, PR111). The campaign still applies its own flag test (`!no_sound && !dedicated`) and call order; behind it, it asks the host once whether a physical sound backend is available and takes the answer. Only the availability decision and the flag query moved: not backend start, the sample loader, blocked-sample state, a device, or playback. This folder is the independently reviewed evidence for that finite slice (`finite_host_sound_backend_availability_slice_closed: true`, no findings). **Task51 and D3 remain open; D4/D5 have not started.** It is not Tacview, visual, audible, shipping or normal-delivery evidence.

## Identities (read this before trusting any run)

| Item | Identity |
|---|---|
| Native production source (immutable) | `b4265e529cfa2bd6113993d404f60b5819a01b86` (`b426`); Windows image `361634d294f2a5f441b6b0cd08a6b5ff45874a98bf1a9ecb59c68faeff477a93` from its BUILD_INFO; 66 new payloads, against 64 old-metadata (`7c`) and 62 old-original (`7574`) payloads for comparison. The older stale-image-ID setup failure (exit 125) is retained and ran no compilation. |
| External witness Lua (frozen) | `1d41509047977863768ab4e52a6202c2fec44a78` (`1d415`), `env-sound-backend-host-witness.lua` SHA256 `af1803cc2d40c3259b40ac7074616e4f1e905670d81365ec81e5a6284fd60035` |
| Delegates | backend driver `env-sound-init-backend.lua` unchanged since `b426` (`c2b15803…afc7b`); old metadata delegate `d7db76cb…c91c7`; original delegate `a40bd591…4182` |
| Test and lint evidence | built/evaluated from the five-file effective `1d415` overlay on the clean `b426` source; these are separate **test artifacts** and are not frozen Windows native executions |
| Publication | the commit that adds this folder; parent `1d4150904`. Runtime code (production C/header/Rust/ABI/observer) is unchanged from `b426`/`1d415` |

**Windows runtime controls used the immutable `b426` `eech-world.exe` and `eech_dc.dll`** with the `1d415` witness script. Nothing is rebuilt or relabelled as a later SHA.

## Results (actual)

- **11 distinct native processes**: 3 default controls (probe, observation-off, header-present) + 8 unique authority cases (healthy absent, healthy control, available, withheld, malformed, old-metadata capability, original capability, flag-skipped). Duplicates add no coverage. All 22 pre/post input manifest checks exit 0 (22 complete input manifests). 66 new, 64 old-metadata and 62 old-original payloads verified.
- **Observation-off**: complete and byte/ending identical to the original: 5191384 bytes, 17283 rows, SHA256 `48307ff0286cff2219a89d7d8b807a3624fa1cb673c440c4ed70d55a7b40d81d`.
- **Metadata comparison** (`default-comparison.json`): the probe has 94 rows (raw 7516 B) and the header-present boot 79 rows (raw 6444 B) against 93/78 in the `7c` metadata reports; all other 89/74 complete lines are byte-exact. Whole-report equality is **false by design**: the new host-backend mirror row (`answers=1 available=0`), the campaign's own `DirectSoundCreate` calls 1 with `E_FAIL` `80004005` becoming 0 with `00000000` (the other three COM rows are unchanged), the derived and result branch label/provenance, and the raw path/byte count. No Core device call is synthesised.
- **Authority**: healthy absent and healthy control, and `available` (unsupported), `withheld` and `malformed` each ended with witness exit 0, a named cause and checked, readable, empty artifacts. `available = true` remains a REFUSAL, never positive hardware or audio. The old `7c` metadata native and the `7574` original each fail the capability control (native exit 1, delegate completed, attempted gate, zero host requests), a genuine red.
- **Flag variation** (`flag-config-variation.json`, whose historical `UNUSED` label is the preparation state, not current usage): one private configuration byte (`no_sound` 0 to 1 in `EECH.INI`, which the 1179-file input manifest excludes, so it is hashed and declared separately). The engine completes with 0 host requests, gate `no_sound=1 backend=0 skipped=1`, and the witness classifies it setup/not-applicable (witness exit 1), not a missing capability and not ownership. Totals: 8 of the 11 processes exit 0 and 3 expected exit 1 (the two legacy capability reds and this not-applicable case); a witness process exiting 0 on a negative case means the engine stop was caught, not a healthy positive backend.

## Preserved failures

- Linux `b426` default affected tests 101 and Windows `env_sound_init` 101 (23/25) on a malformed fixture helper; the three P2 groups of review `7693…64e2fa` were corrected at `1d415` (`correction-source-review.json`, `fd40a244…c95e`). The Windows `b426` aggregate exit 1 (`windows-base-tests.json`) is retained: only the two new `env_sound_init` fixture tests failed and the other 8 executables exit 0, including World 292/292 run normally with `D:/src` absent (the `include_bytes!` portability fix). The earlier metadata source-fixture failure and its history keep their identities.
- The first authority orchestrator aggregate exited 1 because Root's wrapper looked for an underscore verdict in stderr while the actual stderr is hyphenated; a read-only assessment (exit 0) verifies the existing legacy native exit 1, JSON verdict and gate, without a rerun, and only the two unstarted cases were resumed. The initial default auditor exited 1 for omitting a `result.branch` label; a separate classified audit exited 0 after source and diff inspection, with no raw rewrites.

## Scope and what stays open

Only the availability decision and the flag query moved; the flag short circuit is unchanged. Per the reviewed technical-lead disposition, the campaign-reachable catalogue, duration, state, status, range, resume and per-family playback consequences remain required D3, as do the loader and blocked state, physical time and Task 49/50. Pure menu, draw and application-shutdown code with no supported public route are exclusion/removal candidates, not a full EECH UI product and not globally dormant. The one-boot-per-process contract and the exact M6 DCS unload/reload/second-mission check are recorded and unverified. No shipping, Tacview, visual, audible, Task51 or whole-D3 closure is claimed.

## Reproduction and retained private paths

Private evidence is under `R:/d3-evidence/` and `D:/EECH-Migration-Evidence/` (the Windows build `D:/EECH-Migration-Evidence/be-b4265e529/…`, controllers named in `actual-review.json`). No raw recordings, retail assets, scenario roots, binaries or credential files are stored in Git, and the reports published here carry no coordinates, entity IDs, seeds or retail file data. `flag-config-variation.json` contains one local interpreter path (literal, as retained).

## Source map (literal byte copies, no normalization)

The JSON files are byte copies of the private receipts; six of the ten have CRLF line endings, which `.gitattributes` (`*.json -text`) preserves in Git, so `git diff --check` reports trailing whitespace for them by design. Each Git blob SHA256 equals the value below.

| Private source | Published alias | Bytes | SHA256 |
|---|---|---:|---|
| `R:/d3-evidence/independent-environment-host-sound-backend-final-actual-review-native-b426-lua-1d415-20261101.json` | `actual-review.json` | 40560 | `32cd26ce6be8227ecfc80e0e328ec476972b3ebdf7e18cc13d31bd465a4625c3` |
| `R:/d3-evidence/independent-environment-host-sound-backend-source-review-b4265e529-20261101.json` | `source-review.json` | 23330 | `7693ab3a16ab8a4d73a10aa0dcaebeca597ed2d1ed511a79b5979b25ab64e2fa` |
| `R:/d3-evidence/independent-environment-host-sound-backend-correction-source-review-1d4150904-20261101.json` | `correction-source-review.json` | 9112 | `fd40a244a3be3ea7aab384b5b6a5d806779a06f57b37ce876799fb1438f8c95e` |
| `R:/d3-evidence/backend-default-comparison-classified-b4265e529-20261101.json` | `default-comparison.json` | 3707 | `c6d2f798186bd3685d40fc5d4fd8220914178d31c4b7018451890ff775741990` |
| `R:/d3-evidence/backend-old-metadata-capability-assessment-20261101.json` | `legacy-capability-assessment.json` | 2436 | `92113a066cbb359de569bd673f53b8687d5cf2e17484bf316249a059ca37f2fa` |
| `R:/d3-evidence/backend-flag-skipped-preparation-20261101-0648/preparation.json` | `flag-config-variation.json` | 1810 | `e222faae442bc1967be1777d3f81f5b98fb7b530dbef2f2dcedd4dc92abdd1aa` |
| `R:/d3-evidence/current-CI-1d4150904-20261101.json` | `ci-1d415.json` | 14458 | `7c426a625899729f3c61cbd462ddbcfbbb74b8b9b8b5dc72aa22849afe9288fa` |
| `D:/EECH-Migration-Evidence/backend-final-overlay-1d4150904-20261101/controller.json` | `linux-final-checks.json` | 3222 | `1d563c1304884215627a3a23b1ef0ec83e4b9461f2d03adce9a379558838c9c7` |
| `D:/EECH-Migration-Evidence/backend-corrected-windows-guards-1d4150904-20261101/controller.json` | `windows-corrected-guards.json` | 4088 | `614c9b1969e6280da070f09da272310c5c9ba3f36657f06b9cb6a9b26f02b04c` |
| `D:/EECH-Migration-Evidence/backend-windows-tests-b4265e529-build-image-recovery-20261101/controller.json` | `windows-base-tests.json` | 4429 | `b8f64ccce5f478841d1f096a8039dd0e895f55e203da8b91bf644cb5c6680357` |

The complete private recordings, sidecars and per-case controllers named in `actual-review.json` are retained privately with their SHA256 values listed there.
