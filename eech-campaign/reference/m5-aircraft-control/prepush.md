# EEDCS-39 pre-push cleanup (fmt + clippy)

> Repository copy of the pre-push record. Written while the fmt/clippy changes were uncommitted; they are now commit `f1371dda`, so "uncommitted", "no commit" and "a follow-up lint commit is needed" below describe that moment, not the current state.

2026-10-23. Uncommitted on `codex/m5-aircraft-control` over `b721ae5a`. No commit, push or PR. The native pack was not touched (`native/pack.exit` was still absent at the last check). `target/aircraft-control` binaries are unchanged (sha256 of `eech_dc.dll`, `eech-world.exe` and `lua.dll` still match `native/BUILD-INFO.txt`).

## Result
- `cargo fmt --all --check`: clean (EXIT=0).
- `cargo clippy --locked --all-targets -- -D warnings` (the CI step): clean (EXIT=0, `logs/prepush-clippy2.log`).
- Production source is **not** literally unchanged. Apart from rustfmt layout, exactly one production expression changed: the range predicate in `eech-dc/src/module.rs` (below). It is behaviour-equivalent (parent-reviewed, including NaN, signed zero and infinities). Everything else is rustfmt layout or `#[cfg(test)]` code.

## Files changed (all within the EEDCS-39 commit's own files)
- `crates/eech-world/src/main.rs`, `crates/eech-world/src/movement.rs`: `cargo fmt` only (whitespace and line wrapping).
- `crates/eech-dc/src/module.rs`: `cargo fmt`, plus one clippy fix at line 327: `count >= 0.0 && count <= 1.0e6` became `(0.0..=1.0e6).contains(&count)` (clippy: "manual `RangeInclusive::contains` implementation"). This is a **production** change, in the `terrain_sample_count` `LuaValue::Number` arm. It is behaviour-equivalent: both forms reject NaN, and ±0.0 and ±infinity give the same result. The parent confirmed the equivalence in review.
- `crates/eech-engine/src/lib.rs`: `cargo fmt`, plus 20 clippy fixes, all new in `b721ae5a` (blame confirmed):
  - Production code: rustfmt layout only.
  - `#[cfg(test)] mod tests` only. The crate denies `unwrap_used` and `expect_used` (`Cargo.toml [lints.clippy]`), and `--all-targets` lints the tests.
    - 11 `expect_err("...")` calls became `refusal(x)`.
    - 4 `unwrap_err()` calls became `refusal(...)`.
    - 4 `unwrap()` calls became `accepted(...)` (2 `navigation_from`, 2 `CString::new`).
    - These use two small test-local helpers, `refusal` and `accepted`, added to the tests module. Each panics with a message on the wrong variant, as the old calls did. The existing tests already used the `let ... else { panic!() }` idiom, so I did not add an `#[allow]`.
    - 1 `redundant_closure` fix: `.then(|| plan_guidance())` became `.then(plan_guidance)`.
  - No lint was suppressed.

## Checks (full logs in `logs/`)
- `prepush-fmt.log`, `prepush-fmt-check.log`: EXIT=0 and EXIT=0 (local rustfmt 1.9.0-stable; the parent's failure was `final-fmt.log`, 85 diffs in these four files).
- `prepush-clippy1.log`: EXIT=101 on the unfixed tree. 21 errors, all new in `b721ae5a` (20 in the `eech-engine` tests, 1 in `eech-dc/module.rs`). The listing is retained.
- `prepush-clippy2.log`: EXIT=0 after the fixes (docker exit=0).
- `prepush-test.log`: `cargo test --offline --locked -p eech-world -p eech-engine -p eech-dc --no-fail-fast`, EXIT=0. The counts are identical to `r1-green.log`: eech-dc 3 and 2, eech-engine 28, eech-world lib 131, bin 18, host_callbacks 3, load_dc 2; 0 failed. No test was added, removed or loosened.
- Clippy ran with the cached Linux target volume `eech-d1-creation-target2` and the pinned image digest. The image has no clippy, so each run did `rustup component add clippy` inside a transient `--rm` container (`tools/clippy.sh`). The image, its tag and the native build volume were not touched. The component needed network access to download.

## Limitations
- The formatting and lint changes are in the working tree only. The committed `b721ae5a` (the native binaries' source) still fails `fmt --check` and clippy. A follow-up lint commit is needed before push. Relative to the native-built source, the lint commit changes rustfmt layout, test code, and the one equivalent range predicate in `module.rs`. The parent reviewed that predicate as behaviour-equivalent, so no native rebuild is requested; the native binaries are identified by `b721ae5a`, not by the lint commit. `git diff -w b721ae5a` shows the non-layout delta.
- The tests were run for the three changed packages only, not the full workspace. Clippy covered the workspace.
- The `i686` CI matrix leg and the C/aphavoc build were not re-run; no C file changed.
- Not a runtime or acceptance claim. Runtime and airborne coverage stay with the parent's pack supervision.
