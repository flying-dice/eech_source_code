# EEDCS-44 final ordinary Windows regression

The original runner completed at 2026-10-28 02:00:52 UTC with exit0; its launcher
also exited0. All40 accepted checks pass. The independent complete-file comparator
exited0: all8 complete Tacview, observation and metric artifacts equal the verified
destination baseline byte for byte. All3 post-run retail input manifests match.

The frozen clean ordinary source is52206fef322dbb2f4db18620d44da1a57408ad7f,
not a fixture build. BUILD-INFO identifies the toolchain and all30 exported files;
the post-build manifest verifies every hash. The launcher verifies unchanged clean
source/build/accepted-report identity. The accepted baseline is the completed
destination run (baseline-host-20261026-shortpath), whose actual exit0 and accepted
D1 evidence are reused. No baseline, duration, tolerance or checker changed.

This directory contains an explicit safe subset copied byte for byte from private
outputs, with copy-manifest.json recording originals and hashes. Full recordings,
retail assets, binaries, roots and private execution-state dumps remain outside Git.
The narrow .gitattributes preserves evidence bytes and excludes only raw logs from
whitespace lint. It changes no comparator or product code. Tacview equality is a
complete byte/hash comparison, not visual inspection.

The unchanged repository regression-pack.py ran the four established native
campaigns. launcher.py supplies only the verified environment, actual process/exit
records and post-input checks; compare-complete-packs.py independently checks the
accepted40-check set and all8 complete files. Original stderr is empty. No recovery
or campaign restart was needed for this final pack. Earlier failed setup, checker
and diagnostic attempts retain their own source identities and dispositions.

Campaign target/weapon/package selection, engagement policy and consequences stay
in EECH_DC. Physical aiming/readiness is host-owned. Weapon execution and combat
resolution/factual reports remain D2; general terrain/model data, entity mirrors,
shared RNG, effects and sound remain declared coupling. D3 has not started. Final
independent evidence review, current CI and normal merge are separate gates.
