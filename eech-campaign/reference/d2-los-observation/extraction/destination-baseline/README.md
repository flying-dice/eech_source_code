# Destination baseline completion — 26 October 2026

The supplied unchanged Windows x64 binaries completed the established pack on this host:
runner exit 0, all 40 checks PASS, the same check set as accepted D1. This is baseline
reproduction, not LOS candidate acceptance. The candidate pack remains running.

`report.json`/`report.md` are the unedited official reports. Their pack revision identifies
the wrapper checkout, while their build hashes identify the actual supplied D1 binaries
and scripts; those hashes match the accepted D1 report exactly. The command is retained
in the report. Seed 1, 100 ms frames and the full nominal 3-hour scenarios were unchanged.

`destination-baseline-completion.json` records the independently checked completion and
actual outer runner exit. `destination-baseline-full-reference-comparison.json` records
sizes and full SHA-256 for all 6 COMPLETE reference artifacts against accepted D1.
Each Tacview file is 117,690,861 bytes; each observation file is 85,716,447 bytes;
each metric file is 35,942 bytes. Each pair has identical accepted hashes.
This is full-file equality, not a visual Tacview review. Existing blind spots and claim
limits remain in the official report.

The 3 `baseline-post-run-inputs-*.log` records also match the repository manifests after
the run; each command's actual Python exit was 0. Their completion-record fingerprints
identify the external raw logs; Git retains text copies. Private full raw outputs remain under
`R:/d2-evidence/baseline-host-20261026-shortpath`, and retail assets/binaries stay outside Git.
No accepted baseline or tolerance was changed. Final LOS candidate/full-file comparison,
current CI and normal merge remain delivery gates.
