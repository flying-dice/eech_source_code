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
| `retained-dll-check-report.json` | `check` on the real DLL: verdict FAIL (1378 W/E-class functions defined), CLASS_RULES v1 | `1846c6ff4551b5917713721776448e3fe8e28ee23fd37148012e8fa00d2bd01a` |
| `selftest-output.txt` | Selftest run, 80/80 as expected (74 synthetic, 6 on the real DLL); a local temp path on one line is elided and trailing whitespace is stripped. The `retained-dll-head-as-build-commit` line names the repository HEAD of the run (`69110bcc`, the base of the sha256-header change) | `57f2fc96a458fbd5f49df32d62a175e177e2c6d5aea3d1bfd00453ba0a8bc4be` |
| `staging-baseline.json` | Level A per-file baseline (161 campaign files) at the build commit; a ratchet record, not acceptance. Do not re-record silently. | `885025eb42497cd6cdd2b17528670da68d7e75b89d26286d7d11c6c261d026b8` |

| `checkpoint-69110bcc/selftest-output.txt` | Old checkpoint, unchanged: the previous committed selftest run, 72/72 as expected (66 synthetic, 6 on the real DLL), before malformed `sha256` header records were rejected | `e9ee7b5480fe0092839fe132275940b671a6cec6fce1c41b3cce17974624ad57` |
| `checkpoint-cdffc799/retained-dll-check-report.json` | Old checkpoint, unchanged: the report of the tool at `cdffc799`. Same verdict and counts; older reason wording and key names | `70545dceb367ad51c5b193264f0ac2b127df28e53032aab22d8dc2941d950134` |
| `checkpoint-cdffc799/selftest-output.txt` | Old checkpoint, unchanged: 37/37 as expected at `cdffc799` | `5c89634b5b9e5ac3d1c5a35be3516a74b76f8ff9e1f571fe0b472d8219cf321d` |

Hashes are of the committed (LF) bytes. The retained report and the staging baseline (`885025eb…26b8`) are unchanged by the sha256-header change; the real DLL verdict is still FAIL (1378 W/E-class functions; E 707, W 671).
The retained report is Level B on known source definitions and objects only; it is not criterion 2 acceptance.
