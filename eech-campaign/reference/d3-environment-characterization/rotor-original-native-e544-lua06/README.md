# Original helicopter rotor sound computation: finite measured characterization

Scope: the ORIGINAL `update_local_helicopter_rotor_sounds` (`aphavoc/source/entity/mobile/aircraft/helicop/helicop.c`), measured atop the already hosted baseline (weather, wind, session, lifetime, catalogue and backend exchanges): never a whole-original DLL. It is a measurement of what the original does (inputs, ordered pause/resume requests, the levels and pitches written to the sound children, and the standard floating point rounding mode and exception flags around a controlled call). It is **not** host ownership, not a physical extraction, and not Tacview, audio, shipping, Task 51 or whole-D3 evidence. The independent final actual review has no findings and sets `original_rotor_characterization_closed` and `bounded_three_way_rotor_transparency_closed` to true. **Task 51 and D3 remain open; D4/D5 have not started.**

## Identities (three distinct things)

| Item | Identity |
|---|---|
| Native source (frozen) | `e54423b5c17b4386c099240ea02158ec525c05f7` (`e544`), two separately identified Windows x64 FIXTURE builds (test-only, not shipping): `env-observer` (image `sha256:e0834a9b2b04c92e8fcab4a03988cc24869ea16efe4ac6725093367e219687b1`, `eech_dc.dll` `c5311d7159004c143e1ae61eb35ab60bc44f669f0cdcb81bc83c41e8c30a61d1`) and `env-fixture` without the observer (image `sha256:5004a962a9af5e44a0d32f1ce80d0b5ba3b08e19cfc53d1554a9516b428986b9`, `eech_dc.dll` `b4ee63ccbfd65cf92bb3a5d4a7474ca459234c31507ea4ee1623312dc8a6ab90`), 67 payloads each, `eech_world.exe` `2da4f5f5c615abb9b33a56b9d0c93c202dc19f8f632fec867cabcf178760727f` |
| External Lua | `06de242ae9b4050873207925d79a7a2e58be0c87` (`06`): `env-rotor-sounds.lua` SHA256 `c02df5dbf4b951bc6c3ed1728325e8a0d29cc195ddc37ac5abf59e72f0f4519e`, unchanged for every control |
| Test artifacts | the exact `06` Linux overlay and corrected Windows `06` guard executable (guard SHA256 `a53752a75de38a3d6086ce88188f8e65666d6c5dab63a8142b1fbc143b6465fd`), built from the effective `06` source on `e544`: separate test artifacts, NOT runtime binaries and not relabelled |
| Publication | the commit that adds this folder; parent `06de242ae` |

## Results (actual)

- **8 distinct actual native controls, all exits 0**: the seven new ones (natural 120 s; the 12-input AI and 15-input flight-model sweeps each in the uninstrumented build, the observer build unarmed and the observer build armed) plus one reused completed unarmed legacy diagnostic. 16 full input checks 0 (1179 files each), 8 unique roots, 67 payloads per control, external Lua unchanged. No duplicate run.
- **Natural 120 s**: 1200 frames, 304394730 raw bytes, 208800 FULL contiguous phase-0 calls, SHA256 `e5593222ec73e84fe705e01fda0486eb103a5aa1a16db03cb75b788fd91c2591`, 174 AI helicopters with no flight model, boot frame 0 calls, frames 1..1200 174 calls each, no loss, pending, orphan, nesting or overflow, no seed-word change inside a call. The existing 7516-byte sound-initialisation report is exactly the retained `b426` one (constructor, seed and heap rows unchanged).
- **Three-way (uninstrumented / unarmed / armed)**: all 12 AI and 15 flight-model inputs print byte-identical input, output-word and standard-fenv rows (AI 36 rows, 8480 B, `ad0b14ab2bea33b0fe667055c35a1f91e98379519382a30fe1ad9c928ef35f52`; flight model 45 rows, 10675 B, `34f15b6910bf89810ca7877a3459d415fc8b922cd3cd994253be182fe423e394`) and every non-plan raw line is identical. Whole raw files are NOT equal, only in the plan line's `instrumented` / `probe_armed` flags (0/0, 1/0, 1/1); this is reported, not normalised.
- **Unassigned locals under the existing compile contract**: flight-model inputs 10 and 11 show zero TURBINE2 amplitude, 12 and 13 zero APU amplitude and 14 both zero, in all three builds. That is the measured behavior under the already accepted `-ftrivial-auto-var-init=zero` of the canonical whole-engine build (`zero-auto-init-build-contract.json`), not an assumption about raw C.
- **Floating point scope**: rounding mode before and after every call is 3072 (round-toward-zero) and the standard exception sets match across the three builds; this is `fegetround` / `FE_ALL_EXCEPT` evidence on Windows x64 scalar SSE, NOT the x87 control/status word, MXCSR, FTZ/DAZ, an i686 or arbitrary compiler or the float domain.
- **Legacy diagnostic**: the existing general-observer / queue / sound run with the rotor probe UNARMED is complete: 5191384 bytes, 17283 rows, SHA256 `48307ff0286cff2219a89d7d8b807a3624fa1cb673c440c4ed70d55a7b40d81d`, byte and line-ending identical to the retained `b426` original.
- **Builds, tests, CI**: both `e544` Windows profiles build 0; Windows base Engine 98 / DC 36 / World 292 and the earlier rotor guards 11 all 0; Linux `e544` default and observer DLLs and affected tests 0; the final `06` targeted 12 guards plus default and observer strict all 0; all nine exact-`06` CI jobs succeeded (`ci-06de.json`).

## Preserved failures and history

- `a2c5c46a8`: the malformed-parser, sentinel-helper (6/7, exit 101) and 4 success / 5 failure CI results are retained; the `a2c` builds were never used as measurements. The meaningful old-`a2c` missing-after-data red (exit 101) was actually run; the later `06` strict run did not repeat it.
- The first raw-language review of the unassigned locals missed the established `-ftrivial-auto-var-init=zero` contract; the `e544` review records that correction transparently. The `e544` and `99c2` strict-lint failures and their later separate `06` fixes stay distinct. No code, expectation or tolerance was changed to turn a physical mismatch green.

## Scope limits

Finite ORIGINAL characterization and the specified three-way transparency only. The numeric rotor solver still resides in the campaign. The natural domain is AI without a flight model; the flight-model cases are declared synthetic global-flight-model / LOCAL input variation on a live helicopter, not live player flight, non-SESSION backend or range delivery, device or audio, network or render proof. No arbitrary-input, platform or x87 claim. Required unmoved work: non-SESSION range, status and playback of the rotor children (the helicopter-parent pause and resume helpers, per-family playback), fixed-wing equivalents, independent destroy, network, loading and blocked state, physical time, and the Task 49/50 services; shipping 40 / all eight complete artifacts and normal merges remain.

## Publication safety

Literal byte copies of safe receipts only (`.gitattributes` `*.json -text` preserves CRLF where present). `complete-natural-audit.projection.json` is a PROJECTION, not the literal receipt: the private `rotor-complete-natural-native-e544-lua06-20261101.json` (SHA256 `03eb8f5f4a802469bb128b3bd1411a05b3ea1d5033c13e9e1f9bdc355415ca54`) also holds two `env-sndinit-fingerprint` lines with the raw seed word and entity-heap index words, which are withheld here; those two lines are replaced by a fixed text and nothing else changed (the original and candidate lines were verified identical). The seven-native controller contains one local interpreter path, literal as retained. No retail assets, 1179-file manifests, binaries, the 304 MB raw natural record or other private run data are stored in Git.

## Source map

| Private source | Published alias | Bytes | SHA256 | Kind |
|---|---|---:|---|---|
| `R:/d3-evidence/independent-environment-original-rotor-sounds-final-actual-review-native-e544-lua06-20261101.json` | `actual-review.json` | 33011 | `8fccf0860d11b03ade510b58b314cf21965a6db517719e208c4ebbbe9f11def8` | literal copy |
| `R:/d3-evidence/independent-environment-original-rotor-sounds-residual-source-review-99c2a64ae-20261101.json` | `residual-source-review-99c2.json` | 7961 | `5d35ebefa872d5a0be4b5938da76d115db5e1234bed5970fac03cede5e5d34e3` | literal copy |
| `R:/d3-evidence/independent-environment-original-rotor-sounds-test-lint-review-06de242ae-20261101.json` | `test-lint-review-06de.json` | 3545 | `3cb0655ba9291eb2bd1dc29ee2f9912e66f27a9eb0724b2e7d00a865b98a13f6` | literal copy |
| `R:/d3-evidence/independent-environment-original-rotor-sounds-correction-source-review-e54423b5c-20261101.json` | `correction-source-review-e544.json` | 14923 | `48924aa08f68661d4b6d272ee6a5b439fc1975ba63b67046ec0571984e2d69ed` | literal copy |
| `R:/d3-evidence/rotor-complete-natural-native-e544-lua06-20261101.json` | `complete-natural-audit.projection.json` | 4695 | `e33237df8eecedd449dcf44bbfe846e24263b6c44349e127fd3cf752c5300920` | PROJECTION of the private receipt (SHA256 03eb8f5f4a802469bb128b3bd1411a05b3ea1d5033c13e9e1f9bdc355415ca54) |
| `R:/d3-evidence/rotor-three-way-native-e544-lua06-20261101.json` | `three-way-comparison.json` | 7035 | `26ce32dbda84fd233692d0d4cc92154dd7417b40a7467ca3a572d189bb456bab` | literal copy |
| `R:/d3-evidence/rotor-unarmed-comparison-e54423b5c-20261101.json` | `unarmed-comparison.json` | 1181 | `12d37c3f32aaa3f7529ff7bf04fe3e023f36d05554c58fad0a1c8f27117017b6` | literal copy |
| `R:/d3-evidence/rotor-zero-auto-init-build-contract-a2c-7167-20261101.json` | `zero-auto-init-build-contract.json` | 2376 | `98846f4b532178ce83f732bb03d1bde9e0161db3e708ee9942afd684854eb7eb` | literal copy |
| `R:/d3-evidence/current-CI-06de242ae-20261101.json` | `ci-06de.json` | 14457 | `436450524707b03cf7a7b7ff8c9704ff8a3331c3d7152a102008832e8fca229e` | literal copy |
| `D:/EECH-Migration-Evidence/rotor-final-linux-overlay-06de242ae-20261101/controller.json` | `linux-overlay-06de.json` | 5839 | `33dae3f3cbff3d28d7fcb280e349f4667ec493385efd86f082bc2d33b14a1c77` | literal copy |
| `D:/EECH-Migration-Evidence/rotor-corrected-windows-guards-native-e544-driver-06de-20261101/controller.json` | `windows-guards-06de.json` | 3294 | `252b000e094943bf88967df51b7957740838e4c8dc8dc9df1fbbe57e86c9a973` | literal copy |
| `D:/EECH-Migration-Evidence/rotor-original-seven-native-e544-driver-06de-20261101/controller.json` | `seven-native-controller.json` | 12947 | `46b6ae2fd866341de3fa1e306fcebd2f140df861b75a7fedfb30c53eb50ad65b` | literal copy |
