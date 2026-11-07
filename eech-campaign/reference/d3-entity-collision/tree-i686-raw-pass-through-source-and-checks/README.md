# D3 Task49: tree no-hit raw-word pass-through on i686 (UNREVIEWED CODE_FROZEN)

Code `0a7330f860c19a45a415f8a6cbf77513a9db84be` over metadata `d45221d958a70afb78778b1a9ab3aae260469607` over code `cd97e0b723e4f6710622ba9c6c19a16516fa4174`. Source and Linux checks only: **no native actual, no Windows/Wine run, no independent review, no SourceClearance, no acceptance, no CI success.**
The machine-readable account is [tree-i686-raw-pass-through-CODE-FROZEN-20261106.json](tree-i686-raw-pass-through-CODE-FROZEN-20261106.json); every run record is in `direct-records/`, the image and build record in `env/`, the CI snapshot in `ci/`, hashes in `manifest.json`. The earlier `cd97e0b72` subnormal pack and report (`62445b90...`), the `4fb81c673` pack and the sealed review `d6b5df9f...` are preserved and not rewritten.

## The failure and its cause

Current CI at `870a243fb` and `d45221d95` fails both i686 profiles at `tree_host::tests::the_host_asks_for_the_old_end_then_the_new_end_and_a_map_without_trees_is_an_answer_that_leaves_the_callers_words` (`tree_host.rs:342`): the caller's signalling-NaN point word `0x7fa00001` comes back as `0x7fe00001`.

- **Reproduced first** on the unmodified `d45221d95` with the real `i686-unknown-linux-gnu` debug target and the existing test (`red2-focused-d452`, exit 101, words 2145386497 against 2141192193, identical to CI). `red1-focused-d452` (also exit 101) is an environment failure, not the reproduction: the first derivative image had no 32-bit gcc runtime.
- **Cause:** `tree_step.c` `begin_body` and `finish` copied the caller's intercept point and face normal with float assignments. `gcc -m32` compiles each to an x87 `flds`/`fstps` pair, and loading a signalling NaN into the x87 register makes it quiet. Signed zero and subnormals survive such a copy; only a signalling NaN changes.
- **Attribution, not guesswork:** a new kernel-level test that never calls `f32::from_bits` fails on i686 with the unchanged C kernel (`diag1`); with only the C change both that test and the existing host test pass (`diag2`), so the Rust word-to-float conversions in `tree_host.rs` did not alter the word in this profile and no Rust production change is made. Generated 32-bit code: `begin_body` has 32 x87 float moves at `-O1` and `-O2` before the fix and 20 after, exactly the six loads and six stores of the point and normal.

## The change

Two files. `tree_step.c`: `memcpy` of the twelve bytes of the point and of the normal in `begin_body` and in `finish`, plus a compile-time `sizeof(vec3d)` check. `trees.rs`: tests only (two reinterpret helpers, `run_words` and `a_miss_gives_the_callers_point_and_normal_back_bit_for_bit_whatever_they_hold`, three miss exits times three raw-word cases: signalling and quiet NaNs, signed zero, subnormals, extreme finite words). The kernel arithmetic, FP guard, hit geometry, identity, RNG and the old-end-then-new-end terrain order are unchanged; the existing host test still asserts the order and the words.

## Local i686 environment

A task-local derivative of the immutable base image (`4a5618cb...`) adds the CI i686 Toolchain step's content: `gcc-multilib`, `gcc-12-multilib`, `liblua5.1-0-dev:i386`, the i686 Rust target (rust 1.99.1, gcc-12 12.2.0). Three attempts are preserved in `env/` (a bare image id in `FROM` is rejected by buildkit; the second image lacked the 32-bit gcc runtime). One cache owner, one Cargo child at a time through `env/run-tree-i686-direct.py`; the base image and accepted baselines are untouched.

## Checks and their qualification

- Committed clean tree, maps literal equal before and after (3953 files, 3950 raw-equal to the committed blobs, three inherited CRLF Lua files qualified, none outside the map roots): whole-workspace i686 debug (79 result rows, 1391 passed, 1 failed: the second finding below, not the tree), `cargo fmt --check`, and the workflow's workspace `cargo clippy --locked --all-targets -- -D warnings`.
- Working tree before the code commit (no pre-map, none claimed): focused GREEN, whole-workspace i686 default and with `EECH_ORIGINAL_STACK_ATTRIBUTES=1` (same single failure), whole-workspace x86_64 default and `EECH_C_OPT=2` (79 rows, 1412 passed, 0 failed). The i686 runs execute 21 fewer tests than x86_64; the cause (the cfg gates of the earlier i686 test gate) is inferred, not measured.
- Root deliberately blocked the three duplicate final runs of those profiles (`env/ROOT-UNSTARTED-DUPLICATES-20261106.json`, `blocked_before_launch`, no Cargo child); they are not claimed.
- **Contamination, preserved:** `final-i686-focused` FAILED on the committed tree with the old words. This lane's own pristine-d452 control had last built the `-p eech-world` build-script output from older sources, and cargo's mtime-based `rerun-if-changed` never recompiled the newer-timestamp fixed C file (different test binary, 1.95 s build). After an mtime-only touch (SHA-256 unchanged) the rerun rebuilt the C object and passed (`final-i686-focused-c-rebuilt`, run after the after-map). It is a build-cache artefact of this lane, not a defect of the fix.

## Second finding, not fixed here

`ENV-I686-ENTITY-CANDIDATE-NARROW-ULP-1` (OPEN): `tests/entity_candidate_host.rs:907`, world 1: five of the eighteen `Narrow(6, 2000, ...)` words differ by one unit in the last place on i686; x86_64 passes. Reproduced on the pristine `d45221d95` and on the fixed source with identical text. CI stops at the first failing test binary, so it has not seen this yet: after the tree fix both i686 jobs are expected to fail here next. Not diagnosed; outside tree transport.

## Other CI state

The `windows-cross` job failed at `curl: (35) connection reset` fetching Lua from lua.org (infrastructure; reported only). The d452 i686 default job was still running at the snapshot. See `ci/ci-snapshot-d452-20261106.json`; the private decoded logs stay outside Git with their hashes recorded.

## Unchanged and open

The four `fixed_rule_*` fields stay declared in every mode. `REQUIRED_CITY_BOX_LIFECYCLE_RETENTION` stays OPEN and is the mandatory next CITY cut after the finite flags. Default shape/clutter and the six draw-file align readers need the needed-caller/DCS disposition; Core keeps campaign policy. All ten D3 requirements, sector height, UPDATED terrain, raw TREE/FOREST/CLOUD RNG, hierarchy services, DCS mapping, natural moving, final 40/all8, current CI, normal merge and the final census stay open; the f734 ordinary pack and every consumed native root are untouched. Other C kernels were not audited for the same float-copy pattern.

## Pack accounting

83 listed manifest records plus `manifest.json` itself = 84 Git files in this pack.
