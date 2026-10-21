# M5 world-boundary guard: retained evidence

Produced by `tools/world-boundary-check.py` and `tools/world-boundary-check-selftest.py`
(see `docs/m5-world-boundary-guard.md`). Not M5 acceptance evidence.

Input (read only, not retained here; a Windows build from the `m5-host-identities` lane):
`eech_dc.dll` SHA-256 `60cff8d96b97dfa820043db6b4aca136105e5514e5e70aa3de23a876e75f2057`,
built at `5a3b0139ba6d66e4be6b778290025a3bff3dd6e8`; `BUILD-INFO.txt` SHA-256
`c812b53c3f1442925f30cdfece0c75e0ae38d640fb8c2de6c7b41a41b567f026`. No linker map or
object manifest exists for it, so object provenance is unavailable and unverified.

| File | What it is | SHA-256 |
| --- | --- | --- |
| `retained-dll-check-report.json` | `check` on the real DLL: verdict FAIL (1378 physical functions defined) | `70545dceb367ad51c5b193264f0ac2b127df28e53032aab22d8dc2941d950134` |
| `selftest-output.txt` | Selftest run, 37/37 as expected (31 synthetic, 6 on the real DLL); a local temp path on one line is elided | `5c89634b5b9e5ac3d1c5a35be3516a74b76f8ff9e1f571fe0b472d8219cf321d` |
| `staging-baseline.json` | Level A per-file baseline (161 campaign files) at the build commit; a ratchet record, not acceptance. Do not re-record silently. | `885025eb42497cd6cdd2b17528670da68d7e75b89d26286d7d11c6c261d026b8` |

Hashes are of the committed (LF) bytes.
