# D2 combat-resolution characterization evidence (EEDCS-46)

Later runtime investigation: an empty ASAN program reproduced the diagnostic flood
independently of observer code; test-only non-PIE completed8/8. The bounded
non-PIE real-observer guard then passed24 checks. See
[probe evidence and retention limits](c-checks/runtime-probe/README.md). The first
full flood log was not retained, and its identical cause is not asserted. No
shipping flags or campaign data changed. Source correction review and native
observational-equivalence gates remain separate.

Source checks of the read-only combat observer (design: [`docs/m5-combat-characterization.md`](../../docs/m5-combat-characterization.md)). These are NOT native results.

* `checks/syntax-check-46-4.log`: the twelve edited game sources, the observer and the engine glue, compiled for syntax and warnings with and without the fixture macro against the staged headers of the Task45 clean build with this branch's headers laid over them (a subset of the engine: `init.c` and the other callers are not in it).
* `checks/strict-preprocess-46-5.log`: with the fixture macro off the twelve edited sources preprocess identically to their blobs before the hooks (`de991bac3`; PASS x12); `checks/strict-preprocess-46-control-feature-on-5.log` is the control with the macro on (DIFFERS x12, exit 1 as expected).
* `checks/clippy-46-3-default.log`, `clippy-46-3-fixture.log`, `cargo-test-46-2.log`, `fmt-46-2.log`: default and fixture Clippy, the engine, module and world tests (engine 48, world 197 library, 19 binary, 9 host/Lua), and rustfmt; actual Cargo and Docker exit 0 each.
* `c-checks/observer_checks.c`, `c-checks/run-observer-checks.sh`, `c-checks/stub/`, `checks/observer-checks-2.log` to `-4.log`: the observer's own safety checks against the REAL `eech_cmb_fixture.c` over a stand-in engine (24 checks; compile 0, run 0, Docker 0 each; AddressSanitizer and UndefinedBehaviorSanitizer). `checks/observer-checks-1.head.log` is the first 4 KB of the first attempt, which spun in the sanitizer's handler before any output reached the log and was stopped (a 7.6 GB log; not reproduced in three later runs nor in a run of the original, buffered configuration; cause not isolated; the check now prints unbuffered and runs under a timeout).

Two earlier attempts are why the hooks look as they do. Hooking `notify_local_entity` directly made the default Clippy fail in the `eech-sys` crate, which extracts that function verbatim and compiles it without the hook macros; the hooks moved to wrappers installed at message-response initialisation. The first independent review of the observer then found five defects (a variadic read for a bridge notified with no fourth argument and a false fatal for kinds with no damage level, a nesting overflow, six unobserved timed health paths, blast and receiver identity, and unbalanced damage counters); all are corrected in this source. The review record is `combat-characterization-20261028/independent-observer-review-1-20261028.json` in the private evidence tree.

Earlier check logs for the superseded observer (`syntax-check-46-2.log`, `strict-preprocess-46-3.log`, `clippy-46-2-*.log`, `cargo-test-46-1.log`) were replaced by the ones above in this commit; they remain in git history at `47f880f1e`.
