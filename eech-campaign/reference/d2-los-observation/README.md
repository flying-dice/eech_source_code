# EEDCS-42 line-of-sight evidence

Current runtime source `61e856645` has completed characterization, source review and
all40 shipping regression checks. All8 complete scenario artifacts match the
supplied baseline byte for byte. See [`extraction/final-pack/`](extraction/final-pack/README.md)
for reports, input identities and the explicit original exit1/report-only recovery0
distinction. Final CI/normal merge remain delivery gates. No baseline or tolerance changed.

## Historical first fixture build (compile only at that checkpoint)

Fixture source commit `da8b8c327008ce720c95e9f48971ff6b4aab0321` (PR101 head at build time), 0 uncommitted build inputs.
Built 26 Oct 2026 08:14-08:15 UTC in the lead's isolated WSL2 distro `EECH-D2-Build` (Alpine, Docker 29.9.2 on its own
socket) from the repository `tools/Dockerfile` image `eech-build` (manifest list `sha256:0eb7e80f...684d8`; local image id
`sha256:61999cc3...61c2`, created 08:10 before the build; not rebuilt by this run). Command, from `eech-campaign/`:
`sh tools/build-windows-los-fixture.sh target/los-fixture/baseline-20261026-wsl` (fresh cargo volume
`eech-win-target-los-fixture`, release, `x86_64-pc-windows-gnu`, `--features eech-dc/los-fixture`; no default flag changed).
Result: exit 0, `Finished release in 1m 11s`; no `error` lines in the cargo log. Toolchain recorded by the build: rustc/cargo
1.99.1, MinGW `gcc 12-posix`, `gcc-mingw-w64-x86-64-posix 12.2.0-14+deb12u1+25.2+b1`, Lua 5.1.5. This is a newer Rust than the
supplied baseline binaries were built with; `-w` hides C warnings, so no warning-clean claim.

Fixture binaries (FIXTURE BUILD, test-only, never shipping): `eech_dc.dll` `11294e6c...206e`, `eech-world.exe`
`67906e08...69e2`, `lua.dll` `fdd3325e...3a05` (full hashes in `fixture-BUILD-INFO.txt`). The DLL carries the `los_probe`,
`los_trace` and `los_trace_reset` method names and `eech_los_trace_record`; the supplied baseline DLL carries none.

Other checks (Linux image, disposable containers): `cargo fmt --all --check` clean; `cargo clippy --locked -p eech-engine
-p eech-dc --features eech-dc/los-fixture --all-targets -- -D warnings` finished without diagnostics (this also compiled the
fixture C with the Linux gcc; the logged `CLIPPY_EXIT=0` is the pipe's, the verdict is the absence of diagnostics before
`Finished`). `eech-world.exe lua/los-characterization.lua` with no arguments parses and stops at the `root=` assert (no root, no
campaign). Not run: any characterisation, any campaign, `cargo test`, the pack.

## Update 26 Oct 2026: clippy settled, baseline characterisation

- `fixture-clippy-settled.log`: the cached repeat, with cargo's own exit status captured without a pipeline:
  `CARGO_CLIPPY_EXIT=0`, `DOCKER_EXIT=0` (`cargo clippy --locked -p eech-engine -p eech-dc --features eech-dc/los-fixture
  --all-targets -- -D warnings`, `Finished` from cache). This settles the earlier pipe ambiguity.
- `fixture-B-BUILD-INFO.txt`: fixture B (commit `ee7f6f79b`, entity-kind counters), same image and toolchain.
- `characterization-20261026.md`, `traces/`, `controls/`: the characterisation of the unchanged solver, results and limits.

## Update 26 Oct 2026: extraction evidence

`original-transcript/` freezes the original solver's lookup transcript (source `1387331fd`); `extraction/` holds the candidate's results: traces
(40 s byte-identical to the frozen original; 600 s Lebanon and Georgia equal on every compared field), the provider causal and failure runs, the
900 s whole-campaign byte comparison against the supplied baseline binaries, the symbol check, build identities and check tails. The account, limits
and costs are in [`docs/m5-los-observation.md`](../../docs/m5-los-observation.md). Build identity note: BUILD-INFO's `image:` line varied between builds
(`16cfd029...`, `0522090f...`, `808f97a0...`) although the repository image in the WSL store was created once (08:10:12 UTC) and not rebuilt; the local store reports its id
inconsistently, so the identity is the Dockerfile and the toolchain versions recorded in each BUILD-INFO.
