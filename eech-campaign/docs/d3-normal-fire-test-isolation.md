# Normal-fire hosted control: process isolation correction

The unreviewed b2a380/a384527 source and nqt2 proof remain literal. Its exact
final command selected the new hosted query alone, so its actual3041 provider
entries/3030–3041 query interval are valid at that dedicated invocation's scope.
It did not prove an unfiltered two-test invocation: both tests occupied one Rust
executable while C's callback ordinal survives for the process and Lua's host is
fresh per test. Serial ordering or resetting production globals is insufficient.

`eech-engine-sys/tests/normal_fire_query.rs` is now its own Cargo integration-test
executable, required features `physical-flags-collision-check,normal-fire-check,
normal-fire-correlation`. It excludes collision-fixture/model-fixture, exactly
where the current build does not compile HOST_ENTITY. The legacy target contains
only its original test. The new function's control body is moved unchanged; the
shared test provider's unused helpers are allowed only at their test-support use.

The affected command runs BOTH full executables with no test-name filter:

```text
cargo test --locked -p eech-engine-sys --features physical-flags-collision-check,normal-fire-check,normal-fire-correlation --test normal_fire_query --test physical_flags_collision -- --nocapture
```

No C, production Rust, feature definition, Lua, controller, physical input or ABI
changes. Only test-target registration/harness changes. Exact raw source joins
reuse nqt2's real normal-fire/D2, own-hook FPU/RNG, portable13-session,67/2 census,
Python5/default compile and concrete controller evidence; they are not replayed.
The hosted query still has manual registration and ZERO normal-birth credit.

ONE combined SAME source/controller review remains pending on the corrected head,
then Root's distinct frozen observer build/materialized launch review. No native,
Windows build, root creation, cutover/removal/merge or all10 closure occurred.
GenericNative1/Task49/D3/prep2 and all fullpack/final-current gates remain OPEN.
