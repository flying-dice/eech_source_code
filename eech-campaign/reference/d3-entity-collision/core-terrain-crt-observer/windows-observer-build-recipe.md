# Required Windows observer build recipe — UNREVIEWED, not executed

Post-POST documentation addendum. The sealed source/tests and original capture
plan bytes are unchanged. `tools/build-windows.sh` accepts only an output directory;
its cargo command does **not** pass features. Calling that script unchanged would
build ordinary binaries and cannot produce this observer profile.

After SAME source/plan clearance, Root must freeze a clean exact commit with
`core.longpaths=true` before checkout, then bind its actual absolute snapshot,
fresh dedicated Windows target-volume and new output directory in a wrapper.
The existing ordinary image is immutable:
`sha256:09b73ef12aa5202b2cf2869880d8ca4e17cf2783d637bc69a7ac5f32fd33f5e7`
(rustc/cargo1.99.1, gcc12-posix, Lua5.1.5). Root must verify those identities,
not rebuild/re-tag the image or substitute the Linux9d247 image/cache.

The wrapper must execute this exact cargo selection inside that image, working
at `/src/eech-campaign`, with the frozen repository mounted read-only at `/src`,
the new target volume at `/target`, the existing Windows cargo registry mounted
at `/usr/local/cargo/registry`, and the new output directory at `/evidence`:

```sh
set -eu
export CARGO_TARGET_DIR=/target LUA_LIB=/opt/lua-mingw LUA_LIB_NAME=lua
cargo build --locked --release --target x86_64-pc-windows-gnu \
  -p eech-dc -p eech-world --features eech-dc/core-terrain-crt-observer \
  > /target/build.log 2>&1
mkdir -p /evidence/lua
cp /target/x86_64-pc-windows-gnu/release/eech_dc.dll \
   /target/x86_64-pc-windows-gnu/release/eech-world.exe \
   /opt/lua-mingw/lua.dll /evidence/
cp crates/eech-world/lua/*.lua /evidence/lua/
```

No `--all-features`, `target-point-fixture`, Original/model/collision fixture,
`OGRE_EE` or inherited compiler-flag override is allowed. The source defines the
dedicated `EECH_CORE_TERRAIN_CRT_OBSERVER` gate; forwarding of that Cargo feature
through dc→engine→sys and the actual compiled staged-file identity are required
build evidence. The copy follows successful cargo exit only; no capture follows
merely from a successful copy/build.

Root's wrapper must preserve direct build/container exits and raw build log,
record its own literal hash/argv/PID/birth, exact commit/clean status, actual image,
tool versions, selected features/environment, compile commands/defines, both
staged source hashes, COFF identities and full new payload manifest. It must emit
new BUILD-INFO naming this observer profile, never copy ordinary BUILD-INFO as its
identity. The wrapper/container/paths/manifest are not yet materialized or verified.
The copied unchanged campaign.lua and the new observer Lua must each be hash-bound.
The finite two-map plan still requires a separate concrete capture controller and
matched ordinary records/inputs before launch. No Windows build, native run,
retail capture, source cutover or shipping acceptance is asserted here.
