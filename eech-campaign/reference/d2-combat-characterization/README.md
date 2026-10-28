# D2 combat-resolution characterization evidence (EEDCS-46)

Source checks of the read-only combat observer (design: [`docs/m5-combat-characterization.md`](../../docs/m5-combat-characterization.md)). These are NOT native results.

* `checks/syntax-check-46-2.log`: the four edited game sources, the observer and the engine glue, compiled for syntax and warnings with and without the fixture macro against the staged headers of the Task45 clean build with this branch's headers laid over them (a subset of the engine: `init.c` and the other callers are not in it).
* `checks/strict-preprocess-46-3.log`: with the fixture macro off the four edited sources preprocess identically to their `HEAD` blobs (PASS x4); `checks/strict-preprocess-46-control-feature-on-3.log` is the control with the macro on (DIFFERS x4, exit 1 as expected).
* `checks/clippy-46-2-default.log`, `clippy-46-2-fixture.log`, `cargo-test-46-1.log`, `fmt-46-1.log`: default and fixture Clippy, the engine, module and world tests (engine 48, world 197 library, 19 binary, 9 host/Lua), and rustfmt; actual Cargo and Docker exit 0 each.

An earlier attempt hooked `notify_local_entity` directly; the default Clippy then failed in the `eech-sys` crate, which extracts that function verbatim and compiles it without the hook macros. The hooks were moved to wrappers installed at message-response initialisation, which leaves the extracted function untouched.
