# EEDCS-40 author self-review (not an independent review)

Base `6e837440`. The parent is the sole independent reviewer; the technical lead has not approved.

Checked by reading the diff against the original handlers:

- Message order is unchanged: the setters still send before writing the parent position, so the
  host sees the parent's OLD position (what `resolve_dependent_parent` expects).
- `dependent_step.c` follows `hc_msgs.c`: offset from the parent's OLD position, rotation by
  `heading - en_heading`, position, then heading (attitude rebuilt), then velocity.
- An unhosted child, or a hosted child with an unhosted parent, runs the original code unchanged.
- `applying_host_result` depth counter: `++` on begin, guarded `--` on end, `> 0` checks in
  place, rotor_set and vehicle_vector_seed.
- No host success is fabricated: refusals return errors and mutate nothing (tested).

Risks for the reviewer:

1. The host-build DLL C compiled and linked under the pinned MinGW image (`-w`, so no warning-clean claim). The two 900 s baseline probes saw zero carries (non-acceptance). Carries through the real C handlers are observed only via the test-only fixture (doc section "Controlled fixture"): artificial `link` arrangement, 34 checks green, red old-guard build fails 3. The non-host (retail) build was not compiled.
2. A hosted child under an unhosted parent silently uses the original path (disclosed).
3. The position shape ignores `result.attitude` (the original does too).
4. The Rust carry tests were written with the implementation. A red run against the real C path exists only for the guard (old boolean guard, test-only build); the position/helicopter shapes have no red against pre-change C, only green fixture agreement. The witness's `parent_previously_carried` is history, not nesting depth.
5. rustfmt was applied after the last test run.
6. The `debug_fatal` for an unsupported hosted child type is a C path and is untested.
7. The fixture driver is test-only C compiled behind features; a reviewer should confirm it is absent from the default build (no `EECH_DEPENDENT_FIXTURE` define, no Lua method) and unreachable in a shipping module.

## Delta since the fixture (final candidate, `263b48a4`), author self-review

Only `eech-dc/Cargo.toml` changed in source since `c9cb51a3`: the `[features]` block moved above the
lints comment so the comment precedes `[lints.clippy]` again. No semantic change.

- The gated Rust and C (`dependent-fixture*`) were checked with clippy `-D warnings` and built
  by the build script under both features; default checks skip them.
- Features are only `eech-dc/dependent-fixture{,-old-guard}` -> engine -> sys; none is in a
  default feature set, and `build-candidate.sh` builds without them. The normal DLL does not expose
  `engine:dependent_fixture` (public Lua check, with a positive control against the fixture DLL).
- `dependent-fixture.lua` ships in `lua/` as a script; without the fixture DLL its driver call fails
  with the nil-method error, so it cannot move anything in a normal build.
- Not repeated: fixture green/red runs (nothing invalidated them). Not run: the established pack and
  Tacview evidence; the non-host (retail) C build. Hashes differ from the `7c61e13a` build; cause
  not isolated.
