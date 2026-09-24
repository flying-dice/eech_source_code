# Campaign regression test

A test that the campaign still behaves the same after a refactoring. It runs
the retail Lebanon and Georgia campaigns for three simulated hours each, in
parallel, and compares what the war did with the baselines here. It takes
about 20 minutes of wall time.

```sh
# Windows (Git Bash) or any Docker host: builds the toolchain image, the engine
# and the retail roots (once, in Docker volumes), then runs the test
tools/regress-docker.sh "D:/Program Files (x86)/GOG Galaxy/Games/Comanche vs Hokum" \
    "D:/SteamLibrary/steamapps/common/Enemy Engaged Apache vs Havoc"

# Linux, with the engine built and the roots assembled
tools/retail-map3-installs.sh <cvh install> <avh install> /tmp/georgia
tools/retail-cvh.sh <cvh install> map5 /tmp/lebanon
tools/regress.sh /tmp/georgia /tmp/lebanon
```

Natively on Windows, with `eech-world.exe` and `eech_dc.dll`
(`tools/build-windows.sh`, a MinGW-w64 cross-build in Docker). Windows keeps
its own baselines in `regression/windows/`, because its C runtime's `rand ()`
and maths aren't glibc's (`docs/engine.md`, "Windows"):

```powershell
tools\regress-windows.ps1 -Georgia <georgia root> -Lebanon <lebanon root> [-Build] [-Exact] [-Update]
```

Options:

- `--exact` fails unless the runs reproduce the baselines exactly.
- `--update` makes the runs the new baselines.
- `--rebuild-roots` (Docker only) re-assembles the retail roots.

The runs' metrics and logs are left in `target/regress/`.

## What is compared

The runs don't record Tacview. `campaign.lua metrics=<file>` (`lua/metrics.lua`)
samples the world every simulated second. It keeps cumulative counts and takes
a snapshot of the state at the start and every simulated hour.

The first minute is the campaign building its starting world, so units created
then don't count as spawned.

| Metric | Per | Mechanics covered |
|---|---|---|
| sorties | side, kind, task | task generation and assignment, air and ground (ADVANCE, RETREAT, troop movement) |
| aircraft sorties | side, airframe | which aircraft fly |
| weapons launched | side, weapon | combat: air-to-air, air-to-ground, SAMs, artillery, ships |
| lost | side, kind | attrition |
| spawned | side, kind | regen from reserves, reinforcement, troop insertion |
| captures | side, keysite type | the front: FARPs, airbases, factories |
| keysites held and their states | side, type, state | damage, repair, "out of action" |
| supply | side, keysite type: mean ammo and fuel | production (factories, refineries), consumption, SUPPLY deliveries |
| alive | side, kind | the forces |

The two campaigns cover different mechanics:

- **Lebanon (map5)** has the complete retail economy: 25 factories, 11 refineries and 15 ports. It covers production and supply, regen every 600 s, the air war, and naval combat.
- **Georgia (map3)** has blue's only airbase, Batumi, struck out of action at 2:55 and FARPs changing hands from 1:11. It covers captures and the OCA strike.

## Pass and fail

The runs are deterministic: the same build, data and seed reproduce every
number. An unchanged engine therefore reports **IDENTICAL**.

When a change moves individual events, the test compares aggregates at every
checkpoint instead, with tolerances (`tools/regress-compare.py`):

- **Counts:** within 20%, or 3, whichever is larger.
- **Supply:** within 15 percentage points.

Out-of-tolerance metrics are listed as `FAIL` with the baseline and current
values, and in-tolerance differences as `ok`.

A pure refactoring should be IDENTICAL; run with `--exact` to enforce that.
Anything that changes the random stream makes a different war, which the
tolerances don't absorb (below). A deliberate behaviour change should
therefore come with a deliberately updated baseline (`--update`), committed
with the change and with the comparison in its description.

### How the test was checked

- **Reproducible:** a second full run reproduced both baselines exactly: IDENTICAL, IDENTICAL.
- **Detects a broken mechanic:** Lebanon with regen disabled (`REGEN_FREQUENCY 60000`) fails.
  - Every regenerated type drops to 0: blue jets 10 → 0, helicopters 29 → 0 and vehicles 20 → 0; red jets 17 → 0, helicopters 40 → 0 and vehicles 51 → 0.
  - The forces alive fall: blue jets 39 → 14, red helicopters 96 → 66.
- **A different seed is a different war:** Lebanon with `seed=2` has 42 of 244 changed metrics outside tolerance. For example, jets lost go from 11 to 22 for blue and from 17 to 28 for red.
- **Time:** a full run took 19 m 56 s of wall time, including the Docker build and the root setup. The two campaigns ran in 1,110–1,304 s, in parallel, on a machine that was busy with other runs.

## What would change the numbers without a bug

- **The compiler or its flags:** the toolchain is pinned by `tools/Dockerfile`, Debian bookworm's GCC 12 at the build's own flags.
- **The retail data:** the GOG Comanche vs Hokum and Steam Apache vs Havoc installs.
- **The seed or frame length:** `seed=1` and `frame_ms=100` are recorded in each baseline's `run`.
- **The sampling:** `record_every=10` frames, one sample per simulated second.

The observation itself (`engine:objects ()`) only reads state.
