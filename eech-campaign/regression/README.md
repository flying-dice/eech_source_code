# Campaign regression test

A test, run on Windows, that the original EECH dynamic campaign still plays
out correctly after a change. It runs the retail Lebanon and Georgia campaigns
natively (`eech-world.exe` and `eech_dc.dll`) for three simulated hours each,
in parallel. It takes about 20 minutes of wall time.

```powershell
# the binaries: target\windows\ (a MinGW-w64 cross-build in Docker, tools/build-windows.sh)
tools\regress-windows.ps1 -Georgia <georgia root> -Lebanon <lebanon root> [-Build] [-Exact] [-Update]
```

The retail roots are assembled from the installs, which aren't in the repository:

- `tools/retail-map3-installs.sh <cvh> <avh> <root>`: Georgia, from GOG Comanche vs Hokum plus the Steam Apache vs Havoc map3 roads and terrain.
- `tools/retail-cvh.sh <cvh> map5 <root>`: Lebanon.

`tools/regress-docker.sh` builds both into its `eech-regress` Docker volume.

Options:

- `-Build` runs `tools/build-windows.sh` first.
- `-Exact` fails unless the runs reproduce the baselines exactly.
- `-Update` makes the runs the new baselines, in `regression/windows/`.

The runs' metrics and logs are left in `target/regress-windows/`.

## What each run must show

### 1. The campaign plays out as EECH's should

`tools/campaign-expectations.py` checks each run on its own, with no
reference, for the mechanics every correct run of the scenario shows.

| Mechanic | Lebanon (map5) | Georgia (map3) |
|---|---|---|
| Air tasks flown | CAP, CAS, BAI, SEAD, OCA strike and sweep, ground strike, escort, recon, BDA, repair, supply, transfers | the same without OCA strike, plus troop insertion |
| Ground tasks | advance, retreat, patrol | the same, plus insert-and-capture |
| Combat | both sides launch weapons and lose helicopters and vehicles | the same |
| Supply | both sides fly SUPPLY, and airbases consume ammo and fuel | red flies SUPPLY, and airbases consume |
| Regen from reserves | both sides rebuild jets and helicopters (every 600 s) | none: the retail regen interval is 16.7 h |
| Production | factories and refineries producing on both sides | none: map3 has no producers |
| Captures | not required in 3 h | keysites change hands |

Lebanon has the complete retail economy: 25 factories, 11 refineries and 15
ports, three blue airbases and three carriers. Georgia has blue's only
airbase and FARPs changing hands within two hours.

### 2. The run matches its baseline

`campaign.lua metrics=<file>` (`lua/metrics.lua`) samples the world every
simulated second. It keeps cumulative counts and takes a snapshot at the
start and every simulated hour.

The first minute is the campaign building its starting world, so units
created then don't count as spawned.

| Metric | Per |
|---|---|
| sorties | side, kind, task |
| aircraft sorties | side, airframe |
| weapons launched | side, weapon |
| lost, spawned | side, kind |
| captures | side, keysite type |
| keysites held and their states | side, type, state |
| supply | side, keysite type: mean ammo and fuel |
| alive | side, kind |

Runs are deterministic: the same build, data and seed reproduce every
number, so an unchanged engine reports **IDENTICAL**. Otherwise
`tools/regress-compare.py` compares aggregates at every checkpoint, with
tolerances:

- **Counts:** within 20%, or 3, whichever is larger.
- **Supply:** within 15 percentage points.

Anything that changes the random stream fights a different war (see the
seed check below), and the tolerances don't absorb that.

A pure refactoring must be IDENTICAL; `-Exact` enforces it. A deliberate
behaviour change must keep every campaign expectation. It comes with a
deliberately updated baseline (`-Update`), committed with the change and with
the comparison in its description.

## How the test was checked

- **Expectations hold:** the Windows baselines meet every campaign expectation, 30 of 30 for Georgia and 39 of 39 for Lebanon. Lebanon had 37 checks before `12e8f692` added its airbase resupply checks.
- **A broken mechanic fails:** with Lebanon's regen disabled (`REGEN_FREQUENCY 60000`), both checks fail.
  - The expectations fail on all four regen items (blue and red jets and helicopters rebuilt: 0).
  - The baseline comparison fails on every spawned count and on the forces alive: blue jets 39 → 14, red helicopters 96 → 66.
- **A different seed is a different war:** `seed=2` has 42 of 244 changed metrics outside tolerance, while every campaign expectation still holds.

## Lifecycle and failure checks

`tools\lifecycle-windows.ps1 -Root <Lebanon root> [-Bin <dir>]` takes a few seconds and runs no campaign. It boots the module through the same public path, one process per case, and checks:

- one boot per process;
- argument errors and installation errors;
- that the engine can't be reused.

The expected results are the behaviour characterised in [`docs/m1-baseline.md`](../docs/m1-baseline.md). The missing-map crash is checked as a known defect, so the check reports it if that behaviour changes.

## Does it detect a real regression?

`tools\mutation-windows.ps1 -Georgia <root> -Lebanon <root> -Patch <id> [-RepeatMutant]` builds a
test-only mutant with one source patch removed, and a control. Both come from the same clean commit, each in a
temporary worktree under `target/mutants/`, and the normal build is never touched. It runs this regression on
both and reports whether the mutant is detected, and which metrics changed. With S3 removed, both campaigns go
red while the control passes ([`docs/m4-s3-mutation.md`](../docs/m4-s3-mutation.md)).

## The Docker/Linux runner

`tools/regress-docker.sh <cvh> <avh>` and `tools/regress.sh` run the same
test in the Linux container, against the Linux baselines in
`regression/*.json`. That is where the engine was first ported.

The C runtime's `rand ()` and maths library differ between Linux and
Windows, so the two platforms fight different wars from the same seed. Each
platform is deterministic against its own baselines. The Windows baselines
are the ones that matter.
