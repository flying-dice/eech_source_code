# M5 SUPPLY arrival evidence

This directory retains the bounded evidence for the first host-driven campaign
arrival slice. It covers the four SUPPLY waypoint types only. The host receives
copied finite movement facts and an opaque host identity, evaluates the physical
arrival predicate outside the campaign DLL, and returns a synchronous decision.
Campaign policy and the resulting supplier, cargo, receiver, and task mutations
remain inside the campaign DLL.

The retained M4 and causal runs used the release binaries built from
`1578435b3557570635cccbcc416cf5fcb15d2076`; their exact hashes and build inputs
are in [M4-BUILD-INFO.txt](M4-BUILD-INFO.txt). The causal Lua test was finalized
at `13846570326de9b0a536db60cc40d143c9f7a75f` and exercised with integration head
`4028c5a96510d7ef72d2e09c3fe36f3e7038ae88`. A later exact-head build from
`a352c126793e3d1a375af06473c6ae656a95ab00` supplied the lifecycle and native
error controls recorded here. The parent-authored
[independent release-source equivalence record](independent-release-source-equivalence.json)
shows that the production host source at the lint-only head is equivalent to the
source that produced the retained M4 executable. That record is source analysis;
it does not claim that the later PE bytes ran the M4 pack.

The public causal run observed:

- reference callback counts of 4,131 PICK_UP, 11,161 PREPARE_FOR_DROP_OFF,
  5,768 DROP_OFF, and 2,843 FINISH_DROP_OFF calls;
- 101 withheld PICK_UP decisions over 12.2 seconds with physical movement
  continuing while cargo, supplier stock, receiver stock, and task state stayed
  unchanged, followed by cargo `0 -> 1` and supplier fuel
  `96.7999878 -> 86.7999878` after the first allowed arrival;
- 101 delayed DROP_OFF decisions over 10.2 seconds with the same campaign state
  unchanged, followed by receiver fuel `60 -> 100` and task advancement after
  the first allowed arrival. Cargo was `1 -> 1` in that first post-transition
  snapshot; this evidence does not characterize the later FINISH_DROP_OFF
  lifecycle.

The retained exact-head lifecycle summary covers missing callback, malformed
response, Lua error, and same-thread reentry rejection. The PUC Lua and copied-DCS
native-error controls each passed 18 of 18 isolated CLI attempts with exit zero
and no timeout. The M4 report records 40 of 40 passing checks across Georgia and
Lebanon, including the repeatable Lebanon case.

No PE binaries, retail campaign assets, host logs, or large observations are
retained here. Where an original observation was omitted, its SHA-256, exit code,
timeout result, and associated executable identity are retained in the bounded
summary. This slice does not implement non-SUPPLY arrivals, physical-world
separation, campaign loss reporting, a DCS host adapter, or M5 acceptance. The
staged DCS probe still fails boot until a real adapter supplies both mandatory
host callbacks; it has no fallback that manufactures host answers inside the DLL.

Run `python verify-hashes.py` to validate the retained bytes and their cross-file
identities. Run `python verify-hashes-selftest.py` to exercise malformed,
duplicate, missing, mismatched, path-escape, and truncated-digest failures.

## Retained file hashes

| File | SHA-256 |
|---|---|
| [causal-delay-drop-off.out](causal-delay-drop-off.out) | `35f24bea06de2fcb591607296355f95e06b1355b30305ac3413c512901a4488b` |
| [causal-reference.out](causal-reference.out) | `79842966b00385fe3ca4e4a1873bd9e8cc6c7f9b48c866285577536655c4f489` |
| [causal-withhold-pick-up.out](causal-withhold-pick-up.out) | `e0f5476162a26bc3147ebea1d536c841538d6bccd9b16181cc2c504d0782bec0` |
| [causal.summary.json](causal.summary.json) | `1266a5a84716c4354803558dfd572aa93b486b0c7de726b07f63fb550ff36251` |
| [controls-dcs.results.jsonl](controls-dcs.results.jsonl) | `c579bc28354d5c343d68a5f74b3a00d74cd45d4f1ad016e8a05de88442fab179` |
| [controls-dcs.summary.txt](controls-dcs.summary.txt) | `8b3c001a7c984cd9c11fd8b76187f138cba1c09d6ecd016ac0908c3568e2a1d2` |
| [controls-puc.results.jsonl](controls-puc.results.jsonl) | `c6e5bab633f6edaa5552dd79601a934deaef3b2651a821fa3ab5ee7ffd501c4f` |
| [controls-puc.summary.txt](controls-puc.summary.txt) | `95f70e0e85d2d3d989e3ca7bc0df41f3bb1173fc2f7a1f3c21adad6b3a2b2da2` |
| [EXACT-HEAD-BUILD-INFO.txt](EXACT-HEAD-BUILD-INFO.txt) | `38465b68a08ff6536a5f9936c43d93ad1f69e490c915c65054e5e50b9af72b15` |
| [exact-head-lifecycle.summary.json](exact-head-lifecycle.summary.json) | `a523e082622d3247e6fe4572b9d5889e42b423ee96f4459564c91958b7571ca1` |
| [independent-release-source-equivalence.json](independent-release-source-equivalence.json) | `51498b3efc3d91f223ffea8e564ddd24c7754f68aee587f2aaabf49f8546ab44` |
| [M4-BUILD-INFO.txt](M4-BUILD-INFO.txt) | `71180a2a3f9f1027503eafa4eb76f9a225576937308932dea14edf03e57933e3` |
| [m4-pack.report.json](m4-pack.report.json) | `71f34ec19cde14af23ee0da543a39c5a66eb35127447c8c7ab939b7fe4cf5281` |
| [m4-pack.report.md](m4-pack.report.md) | `ebde6a056c1d4dea1bf204f4f0dee8c6361282494efcafb091d38ea64e5bdb1f` |
| [M4-TEST-INFO.txt](M4-TEST-INFO.txt) | `6d878e316e3486b49513c6d4ec9253de7e0c32bc8ae73915b8ddb0d93feb295b` |
