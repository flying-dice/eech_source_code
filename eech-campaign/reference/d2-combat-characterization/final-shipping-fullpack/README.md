# Final ordinary shipping D2 regression

The original established pack completed on 28 October 2026 at 23:53:56 UTC
with native source 64cf0f1c5551a2e59309a92917c32e6189d8ae60. Its original
controller exited 0, and its wrapper completed at 23:54:10 UTC. This closes
the final native validation gate; independent final review and normal delivery
are separate gates. D2/EEDCS-18 and EEDCS-46 are not declared accepted by these
at-time reports.

- The exact accepted 40-check set passed. No baseline or tolerance changed.
- All eight COMPLETE candidate artifacts equal the completed destination
  baseline byte for byte: both ACMIs, both observation files, both reference
  metrics and the Georgia/Lebanon regression metrics.
- All three required external post-input checks passed. The runner's fourth,
  internal Lebanon copy also passed its additional 1179-file post-input check.
- The shipping 42-entry file manifest and BUILD-INFO identity are unchanged.
  BUILD-INFO SHA-256 is
  3410ac4174a259e0ee211e4bb05361a819e1c1da54a300abe93968eff08e169d.
- Scenario settings remain seed 1, frame 100 ms, three simulated hours, with
  Georgia/Lebanon unrecorded and Lebanon twice recorded. Full original commands
  and roots are in pack-run.json and preflight.json.

The complete-eight-files.json report includes both baseline and candidate
lengths and SHA-256 values. Each ACMI is 117690861 bytes, SHA-256
3bd17b647665b558c7378206a3717c94e9cf771d4781d78e0b2aaae9a7d2d19b.
Each observation file is 85716447 bytes, SHA-256
da26bab01163aa4ab725f9f2ecb516d37a4558777fe3684222b8d885822e77cf.
This is structural/full-byte comparison, not a claim of visual Tacview inspection.

publication-manifest.json maps each exact copy to its retained private original.
The full recordings, observations, raw campaign metrics, retail roots and
shipping binaries remain under R:/d2-evidence; none is copied into this folder.
The installed eech-regression skill supplies the full Windows setup/run workflow;
the exact comparison helper used here is also retained.

The runtime build is never relabelled as a later metadata commit. At daf3960d8
seven coarse source frontiers match 64cf; the crates difference is only the
integration-test host_callbacks.rs. Production compilation inputs are unchanged.
ci-at-daf-all-nine.json is evidence for that exact head, not approval of future
publication commits.

Reuse the independently closed component, six public report-case and 9000-frame
cost evidence in the neighbouring folders, within their stated limits. The cost
probe measures loaded elapsed engine:frame time; it establishes neither a DCS
frame budget nor a causal parent performance comparison. Default recording
equality alone is not proof of replaceable host authority; the controlled host
and factual-report witnesses establish the characterized boundary.

The finite whole-D2 matrix is in docs/handover/D2-SCOPE-AUDIT.md. Campaign
selection, engagement, awareness, credit, accounting and route/repair policy
remain in EECH_DC. Terrain/model services, physical logistics, dormant code,
entity mirrors, effects/sounds and the shared random stream retain their
explicit scope/coupling dispositions. No D3 work is started.
