# Corrections to original EECH behaviour in the M1 candidate

This review covers the M1 blocker in [#19](https://github.com/flying-dice/eech_source_code/issues/19): the behavioural corrections and compatibility decisions in the candidate must be "visible and reviewed rather than silently inherited".

- **The candidate** is the one described in [`m1-baseline.md`](m1-baseline.md). This review works on the integration branch at `4653982c`, where the engine code is the same as `5f07697b`.
- **Scope:** every source patch in `crates/eech-engine-sys/build/patches.rs` at `4653982c` (18 ids, 22 entries; S2 has since been removed), plus the compiler flags that give the original code a defined behaviour. P1 is not in the list the review was asked for, but it is in the candidate, so it is reviewed too.
- **Not reviewed here:** the platform layer's substitutions and the data (listed at the end).

No engine behaviour is changed by this review. It corrects documentation only.

## How each patch is classified

| Class | Meaning |
|---|---|
| **Defect correction** | The original text, compiled for this target, does something its own code shows to be wrong: undefined behaviour (an out-of-bounds access, a NaN used as an index, a truncated pointer) or a result that contradicts the evident intent of the surrounding code. The patch changes behaviour only where the defect occurs. *Portability* marks a defect that exists only on this target (x86-64, GCC); there, the patch restores what the original does on its own platform (32-bit MSVC). |
| **Intentional semantic change** | The patch replaces defined original behaviour with something the original does not do. It needs explicit acceptance as a compatibility decision. |
| **No behavioural change** | Compile-only. Listed for completeness. |

Evidence is marked as follows:
- **traced:** the original code was read for this review;
- **reported:** a commit message or document says so, but it was not reproduced here;
- **reproduced:** run for this review.

## Summary

| Id | Where | Class | Evidence |
|---|---|---|---|
| E1, E2 | `system/debug.h`, `ai/highlevl/highlevl.h` | No behavioural change | traced |
| E3 | `effect/explosn/xp_dbase.h` | No behavioural change | traced |
| P1 | `en_funcs/en_creat.c` | Defect correction (portability) | traced; kernel i686 equivalence reported |
| B2 | `ui_sys/ui_attrs/ui_attrs.c` | Defect correction (portability) | traced; crash reported |
| X1 | `system/fpu.h` | Defect correction (portability) | traced; wrong sectors reported |
| W1 | `weapon/wn_move.c` | Defect correction | traced; fault reported |
| S1 | `force/fc_msgs.c` | Defect correction | traced; reported |
| **S2** | `force/fc_msgs.c` | **Intentional semantic change**: rejected, removed in `0288ece2` | traced; necessity **not** reproduced (without S2, all 39 Lebanon expectations hold) |
| S3 | `mobile/mb_msgs.c` | Defect correction | traced; reproduced (without S3, no airbase gets ammo) |
| N1, N2 | `terrain/terrelev.c`, `smokelst/sl_move.c`, `sl_updt.c` | Defect correction (guards) | traced; reported |
| N3 | `terrain/terrelev.c` | Defect correction, with a chosen substitute value | traced; crash reported; no effect in 3 h reproduced |
| H1 | `campaign/ca_hist.c` | Defect correction | traced; AddressSanitizer reported |
| U1 | `options/op_real.c` | Defect correction | traced; AddressSanitizer reported |
| C1 | `3d/3dobjdb.c` | Defect correction (the trigger depends on the data) | traced; fault reported |
| **T1, T2** | `graphics/textuser.c`, `3d/3dobjid.c` | **Intentional semantic change** (headless data tolerance) | traced; reported |
| **F1** | compiler flag `-ftrivial-auto-var-init=zero` | **Intentional semantic change** (compatibility decision) | traced (one known site) |
| — | `-fwrapv`, `-fno-strict-aliasing`, `-fcommon` | No intended change: they keep the original compiler's treatment | not independently verified |

Three changes needed explicit decisions as compatibility decisions (see "M1 review decision" below):
- **S2**, which changes campaign logistics. This review did not reproduce its stated need. It is rejected.
- **T1 and T2**, which let an inconsistent data set load. They are accepted.
- **F1**, which defines what uninitialised locals read. It is accepted.

E1–E3 change nothing. All the others are defect corrections.

## M1 review decision

Recorded by the technical lead, @fd-starscream-bot, in the review of #62 (2026-09-25) for the M1 candidate:

| Change | Decision | Scope and limits |
|---|---|---|
| **S2** | **Rejected** | Its stated need is not reproduced once S3 is present, and S3 explains the original diagnostic. S2 replaces a defined EECH supplier rule, so the conservative M1 position is to restore the original rule. It was removed in a separate follow-up (`0288ece2`), with the Lebanon Windows baseline re-recorded; see "S2 removal" below. #62 itself changed no engine behaviour. |
| **T1, T2** | **Accepted** | Explicit headless/data compatibility decisions. They cover the mixed retail/community data profile only. They do not establish a general rendering or data policy. |
| **F1** | **Accepted** | An explicit determinism/compatibility decision. It gives a defined value to every read the original leaves indeterminate, across the whole engine, and only one concrete site has been traced. It is not a claim of historical EECH fidelity. |
| **N3's fallback value** | **Accepted** | The chosen recovery value within an otherwise valid defect correction. The local vertex height is a project compatibility choice, distinct from the sector maximum used by the original's commented-out fallback. |
| All other classifications | **Accepted as documented** | The defect corrections (P1, B2, X1, W1, S1, S3, N1, N2, N3, H1, U1, C1) and the unchanged E1–E3. |

## The patches

### E1, E2: logging macros (no behavioural change)

- **Original:** in a release build (`DEBUG` undefined) under `WIN32`, `debug_log` and `ai_log` are `#define debug_log();`, called with arguments. MSVC drops the extra arguments (warning C4002), so the call becomes `;` and its arguments are never evaluated. GCC rejects the call.
- **Patch:** GCC takes the original's own non-`WIN32` definitions, `do { } while(0);`, which also discard the arguments unevaluated.
- **Class:** identical behaviour.
- **Note:** the engine build does not define `DEBUG`, so every `debug_log` compiles to nothing. This matters for T1 and T2.

### E3: explosion database union (no behavioural change)

In `xp_dbase.h`, two anonymous structs of one union both declare `frequency` and `smoke_lifetime`. C rejects the repeated names. The two layouts are:
- first struct: `initial_speed`, `frequency`, `smoke_lifetime`, …
- second struct: `generator_lifetime`, `frequency`, `smoke_lifetime`, `initial_velocity`

So both names sit at offsets 4 and 8 in each struct. The patch renames the second struct's copies. Every access resolves to the same storage as before.

### P1: entity attribute lists on x86-64 (defect correction, portability)

- **Original:** `create_local_entity` and `create_client_server_entity` pass the variadic attribute list on as `(char *) pargs`. That reinterprets the `va_list` as the i386 argument stack. On x86-64 a `va_list` is a descriptor (System V) or has 8-byte slots (Win64), so the readers get garbage.
- **Patch:** `csrc/eech_attrs.c` walks the same attribute grammar with `va_arg` and writes the list `get_list_item` expects.
- **Evidence:** `docs/64-bit.md` (B1). On i686, the kernel's corpus passes with the original line and with the patch alike (`EECH_ORIGINAL_STACK_ATTRIBUTES=1`). That is reported, not rerun here. The engine's marshaller is the kernel's grammar walk; only its error reporting differs (`debug_fatal` instead of `eech_abort`). Every entity creation in every campaign run depends on it.

### B2: UI attribute pointers (defect correction, portability)

- **Original:** nine `ui_attrs.c` sites read a pointer argument as `va_arg (pargs, int)`: the five `ui_object *` attributes and the four graphic attributes. On i386 an int and a pointer are both 4 bytes. On x86-64 the pointer is truncated.
- **Patch:** reads the arguments as pointers.
- **Evidence:** the first UI screen built at boot crashed without it (reported, `c1d344e2`).

### X1: float-to-int conversion (defect correction, portability)

- **Original:** `fpu.h` has one branch per compiler.
  - The MSVC branch is `fld value; fistp dword ptr [edx]`: a 32-bit store in the current rounding mode.
  - The GNU branch (used by any GCC build) is `fistp (%1)`, which in AT&T syntax is the 16-bit store. Values of 32,768 or more saturate.
- **Rounding:** EECH sets round-toward-zero (`set_fpu_rounding_mode_zero` in the renderer, terrain, sound and network initialisation). No code in the tree sets another rounding mode, and the platform layer runs every entry under round-toward-zero.
- **Patch:** `(int) value`, which truncates. That is what the MSVC branch computes. Out-of-range values and NaN give `INT_MIN` both ways.
- **Evidence:** reported (`99b5987c`). World coordinates past 32.7 km fell into the wrong sector: on the synthetic Georgia map every keysite came out blue.

### W1: ballistic solution at point blank (defect correction)

- **Original:** `get_ballistic_pitch_deflection` computes `asin (height / range)` and `sqrt (range² − height²)`. The range it is given is the 3-D distance (or, for the player's triangulating range finder, an estimate). In the refinement loop, `weapon.c` then adds up to ±5 m of random jitter (`range += 5 * sfrand1 ()`, line 1058). Near a target almost directly above or below, the height can therefore exceed the range. Then the pitch and range are NaN. `range <= 0.1` and `bound` let NaN through, and `(int) (range / RANGE_STEP)` indexes the ballistics table at `INT_MIN`. The x87 conversion gives the same index, so the original has the same fault. A commented-out `ASSERT(range >= 0.0)` suggests the authors met it.
- **Patch:** returns FALSE ("no solution") unless `|height| < range`. FALSE is an existing result: callers already handle it for unknown weapons.
- **Difference:** only where `|height| >= range`. In the exact-equality case the original returned a ±90° aim at zero range; that case has measure zero.
- **Evidence:** fault reported (`231bc98c`).

### S1: a keysite is not its own supplier (defect correction)

- **Original:** `response_to_force_low_on_supplies` (`fc_msgs.c`) looks up the closest airbase with `get_closest_keysite (…, 10 * KILOMETRE, …, TRUE, NULL)`. The 10 km is an early-out range: the first in-use airbase within it is returned at once. With no exclusion, an airbase asking for supplies finds itself at about 0 km. A SUPPLY task from a keysite to itself is meaningless. The function's `exclude_keysite` argument exists for exactly this.
- **Patch:** passes the requester as the exclusion.
- **Evidence:** traced; reported (`1b47fa02`: no airbase was ever resupplied).

### S2: fall back to a producer that holds the cargo (intentional semantic change)

**Original rule.** A keysite low on ammo (or fuel) is supplied as follows:
1. The supplier is the nearest factory (refinery for fuel; the other if none), or the nearest airbase when that is nearer.
2. It then looks for a crate of the type at that supplier.
3. If there is none, it creates no task. The original handles this outcome explicitly, with the debug message "cannot locate cargo at factory". The keysite asks again later.

**Patch.** When the chosen supplier holds no crate of the type, S2 takes the nearest producer that does (a factory for ammo, a refinery for fuel, then the other).

**Why it is a semantic change.** It replaces an outcome the original defines with a new supplier rule.

**Why it was added.** Airbases hold crates in proportion to their own supply level (`update_keysite_cargo`). Under the original rule, a drained airbase is chosen as the supplier for another drained airbase whenever it is nearer than a factory.

**Evidence.**
- **Reported** (`12e8f692`): on retail Lebanon, 367 ammo requests in 2 simulated hours produced no ammo delivery to any airbase.
- **Not reproduced with S1 and S3 in place** (this review). With only S2 left out, 3 simulated hours of Lebanon still resupplied airbases:
  - with ammo 3 times (blue 0, red 3; the candidate: blue 1, red 2);
  - with fuel 8 times.

  All 39 Lebanon expectations hold. The war diverges from the baseline (23 metrics outside tolerance), as it does after any behaviour change. The reported 2-hour diagnostic predates S3. Without S3, ammo tasks often delivered fuel (see S3), which by itself explains airbases getting no ammo.

**For acceptance.** S2's necessity is not established: the S3 defect explains its stated motivation, and without S2 the candidate still resupplies airbases. There are two options:
- **Remove S2**, restoring the original rule. This is the conservative choice. It needs a follow-up PR with new Windows baselines.
- **Accept S2** as a compatibility decision (a supplier fallback the original does not have).

Only 3 hours were compared. Without S2, blue airbases got no ammo in that time, against once with it; a longer run would show whether that matters.

**Decision:** rejected for the M1 candidate ("M1 review decision" above). Removed in `0288ece2` (see "S2 removal").

### S3: pick up the crate the task was created for (defect correction)

- **Original:** `create_supply_task` starts the task at a specific crate of the requested type and records that type in the task (`FLOAT_TYPE_TASK_USER_DATA`). The duplicate guard in `fc_msgs.c` relies on that record. At the pick-up waypoint, though, `response_to_waypoint_pick_up_reached` (`mb_msgs.c`) takes the keysite's first cargo entity, whatever its type.
- **Effect:** the drop-off (`ks_msgs.c`) refills the receiver's ammo or fuel by the carried crate's type, and the pick-up deducts that type from the supplier. So an ammo task that loads fuel delivers fuel. While it is active, the duplicate guard blocks another ammo task.
- **Patch:** takes a crate of the task's type, and only failing that the first crate, as the original does.
- **Evidence:** traced. Reported (`12e8f692`): in 3 hours of Lebanon, no ammo crate was picked up at a factory.
- **Reproduced** (this review). With only S3 left out, 3 simulated hours of Lebanon:
  - resupplied no airbase with ammo (the candidate: 3 times);
  - resupplied airbases with fuel 17 times (the candidate: 10);
  - left airbase ammo at 38 (blue) and 30.7 (red), against the candidate's 49.9 and 45.5.

  The campaign expectation "airbases resupplied with ammo" fails (38 of 39 hold).

### N1, N2: NaN positions (defect correction, guards)

- **N1:** `get_3d_terrain_point_data` range-checks its position with `x < min || x > max`, which NaN passes. The sector index is then garbage and the lookup reads outside `terrain_sectors`. N1 reports the caller and uses the map corner.
- **N2:** reports a smoke point created with a NaN position or motion, and drops a moving one.

Both change behaviour only for NaN, where the original's behaviour is undefined. Neither has reported anything since N3 (reported, `5f07697b`: a 14-hour Lebanon run with no NaN report).

### N3: degenerate terrain faces (defect correction, with a chosen substitute value)

- **Original:** the face normal is the normalised cross product of two edges. For a degenerate triangle (collinear or coincident points) the cross product is zero, and `normalise_any_3d_vector` returns a zero vector (not NaN). The elevation formula then divides by the normal's y component: `dy = (nx·dx + nz·dz) / ny` is 0/0, which is NaN. A near-vertical face (`ny` about 0) gives a huge or infinite `dy` instead. Wind-drifted smoke carries the NaN into a terrain query (N1).
- **Patch:** when `!(normal.y > 0.00001)`, uses the up vector. The elevation is then the reference vertex's height.
- **The substitute value is a choice.** The original's own fallback for this case is commented out ("//DEBUG//"). It also uses the up vector and treats undercut faces (`ny < 0`) the same way, but it sets the elevation to the sector's maximum height. So does the original's active "failed to locate a terrain face" path. N3 keeps the local vertex height instead, which is accurate for a single bad face; the sector maximum can be hundreds of metres higher. `patches.rs` described the NaN mechanism wrongly ("normalises to NaN"); this is now corrected.
- **Not established:** whether the original 32-bit MSVC/x87 build reaches NaN on the same faces. Exactly coincident points give a zero cross product in any precision; nearly collinear ones might not under x87's extended precision.
- **Evidence:**
  - Reported (`5f07697b`): a deterministic access violation at 39,327 s of Windows Lebanon, reproduced three times; with N3 the same war runs 14 h with no NaN report.
  - Reproduced: the Windows baselines date from `12e8f692`, before N3, and the M1 candidate reproduces them exactly. N3 therefore changes nothing observable in the first 3 simulated hours of either campaign.

### H1: an empty campaign history (defect correction)

- **Original:** `maintain_campaign_history` (`ca_hist.c`) reads `campaign_history [num_campaign_history_items - 1]` without checking for an empty history. With none, it reads index −1. When the garbage it reads happens to match the displayed page, it calls `remove_campaign_history (-1)`, and the history count can go negative.
- **Where it matters:** `maintain_campaign_history` runs whenever a campaign item (a task, for example) is destroyed (`response_to_campaign_item_destroyed`). Headless, no screens are navigated, so the history is always empty and every such event takes this path. In a played game the history is usually populated.
- **Patch:** returns when the history is empty.
- **Evidence:** traced; AddressSanitizer global-buffer-overflow (reported, `5f07697b`).
- **Correction:** H1's note in `patches.rs` blamed it for the Lebanon crashes. The evidence points to N3; this is now corrected.

### U1: the co-pilot target-report button (defect correction)

- **Original:** `op_real.c` sizes the button from `option_cpg_report_targets_text`, a 2-entry array, with a count of 4. It reads two pointers past the array.
- **Patch:** uses 2.
- **Effect:** UI sizing only.
- **Evidence:** traced; AddressSanitizer (reported, `5f07697b`).

### C1: "no collision object" in the binary scene database (defect correction; the trigger depends on the data)

- **Original:** a scene's collision object 0 is the null object, meaning none. The `.EES` loading path maps 0 to −1, but the `3dobjdb.bin` path keeps 0. The consumers treat anything but −1 as a real object and read the null object's NULL surface list.
- **Patch:** maps 0 to −1 on the binary path too.
- **Trigger:** the candidate's retail Apache vs Havoc database, with the community objects over it, stores 0 (`RS_MANPAD`, for one). Whether a shipped game's database does is unknown.
- **Evidence:** traced; fault reported (`b58e56e5`).

### T1, T2: texture data inconsistencies (intentional semantic change, headless data tolerance)

- **Original:** texture registration treats a camouflage mismatch as fatal (T1), and so does a missing texture animation (T2).
- **Candidate's data:** it mixes the retail Apache vs Havoc texture set with the community objects, and they disagree. It is not a shipped configuration.
- **Patch:** continues instead of stopping (T2 uses animation 0).
- **Correction:** the patches said "logged". `debug_log` compiles to nothing in this build (E1), so the mismatches are ignored silently; this is now corrected.
- **Effect:** only texture camouflage flags and animation indices, which nothing headless uses for campaign behaviour. The objects' geometry, collision meshes and weapon mounts still load.

## Compiler flags

- **F1, `-ftrivial-auto-var-init=zero` (every engine unit):** an intentional semantic change, as a compatibility decision. A local read before it is written is 0; in the original MSVC release build its value is whatever the stack held.
  - One site is known: `create_supply_task` reads the uninitialised `prepare.y` and `finish.y`. There, the C reference and the TSTL port made the same decision (`docs/patches.md`, F1).
  - Other sites in the whole engine are not catalogued.
- **`-fwrapv`, `-fno-strict-aliasing`, `-fcommon`:** these make GCC treat signed overflow, type punning and tentative definitions the way the original compiler does. They are meant to preserve the original's behaviour, not change it; this is not independently verified.

## Reproduced for this review

Each variant is the candidate's source with one patch entry removed from `build/patches.rs`, built locally and not committed. `grep "EECH headless (S1)"`, `(S2)` and `(S3)` over each build's staged `fc_msgs.c` and `mb_msgs.c` confirmed what each variant contains.

```sh
# Git Bash, Docker; build/patches.rs with the one entry removed
EECH_WIN_TARGET=eech-win-target-review-a tools/build-windows.sh target/review-no-s2   # without S2
EECH_WIN_TARGET=eech-win-target-review-b tools/build-windows.sh target/review-no-s3   # without S3
```
```powershell
# each: retail Lebanon, seed 1, 3 simulated hours, as tools\regress-windows.ps1 runs it
target\review-no-s2\eech-world.exe target/review-no-s2/lua/campaign.lua root=<Lebanon root> scenario=lebanon_retail hours=3 record=0 checkpoint_every=3600 metrics=<abs path>/review-no-s2.json
python tools\campaign-expectations.py <metrics>
python tools\regress-compare.py regression\windows\lebanon_retail.json <metrics>
```

Lebanon after 3 simulated hours (Windows, 2026-09-26):

| | Candidate (S1, S2, S3) | Without S2 | Without S3 |
|---|---|---|---|
| Airbase ammo resupplies (blue / red) | 1 / 2 | 0 / 3 | **0 / 0** |
| Airbase fuel resupplies (blue / red) | 1 / 9 | 0 / 8 | 2 / 15 |
| Airbase ammo level (blue / red) | 49.9 / 45.5 | 54 / 43.3 | 38 / 30.7 |
| Airbase fuel level (blue / red) | 31.1 / 38.3 | 21 / 32.3 | 51.2 / 49 |
| SUPPLY sorties (blue / red) | 11 / 52 | 10 / 49 | 15 / 49 |
| Campaign expectations | 39 / 39 | 39 / 39 | **38 / 39** (no airbase ammo) |
| Metrics outside the baseline's tolerance | 0 (IDENTICAL) | 23 | 9 |

The candidate's column is `regression/windows/lebanon_retail.json`, which the M1 candidate reproduces exactly (#60). One run per variant: a different war can move these counts by chance.

## S2 removal

Following the M1 review decision, `0288ece2` removes S2 from `build/patches.rs`. `response_to_force_low_on_supplies` now compiles as the original EECH text: when the chosen supplier holds no crate of the type asked for, no SUPPLY task is created, and the keysite asks again later. No other patch changed.

**Commands** (Windows, 2026-09-26):

```sh
tools/build-windows.sh target/m1-no-s2      # commit 0288ece2, 0 uncommitted changes; eech_dc.dll da72f128…
```
```powershell
tools\lifecycle-windows.ps1 -Root <Lebanon root> -Bin target\m1-no-s2
tools\regress-windows.ps1 -Georgia <root> -Lebanon <root> -Bin target\m1-no-s2 -Exact
```

**Results:**
- The staged `fc_msgs.c` contains S1 and no S2; `mb_msgs.c` contains S3.
- Lifecycle: 5 PASS, 1 KNOWN DEFECT, unchanged.
- **Georgia: IDENTICAL** to its baseline. That is expected: retail Georgia has no factories or refineries, so S2's fallback never found a producer there.
- **Lebanon:** all 39 campaign expectations hold, but 23 metrics are outside the old baseline's tolerance.
- The new Lebanon metrics are **byte-identical** to the "without S2" run of this review (built separately from `4653982c`, whose engine code is the same). The new baseline has therefore been produced twice.
- `regression/windows/lebanon_retail.json` is re-recorded from this run. The old baseline stays in the history at `12e8f692`.

**Lebanon after 3 simulated hours, before and after:**

| | With S2 (`12e8f692` baseline) | Without S2 (new baseline) |
|---|---|---|
| Task sorties, blue / red | 163 / 227 | 150 / 223 |
| SUPPLY sorties, blue / red | 11 / 52 | 10 / 49 |
| Airbase ammo resupplies, blue / red | 1 / 2 | 0 / 3 |
| Airbase fuel resupplies, blue / red | 1 / 9 | 0 / 8 |
| Airbase ammo level, blue / red | 49.9 / 45.5 | 54 / 43.3 |
| Airbase fuel level, blue / red | 31.1 / 38.3 | 21 / 32.3 |
| Units lost, blue / red | 293 / 254 | 285 / 245 |
| Units regenerated or spawned, blue / red | 108 / 148 | 65 / 132 |
| Keysites captured, blue / red | 1 FARP / none | none / none |
| Campaign expectations | 39 / 39 | 39 / 39 |

**Why the difference is expected.** S2 only acts when the chosen supplier has no cargo of the type. Removing it means fewer SUPPLY tasks, from a different set of suppliers (blue 11 → 10, red 52 → 49), and so different deliveries. From there the war diverges, as it does after any behaviour change (a different seed moves 42 metrics; `regression/README.md`). The combat differences (losses, regen, the FARP capture) come from that divergence, not directly from S2. Every mechanic the expectations check still occurs, including airbase ammo and fuel resupply.

## Outside this review

These are compatibility decisions of the host layer and the data, not patches to EECH's text. They are described in `docs/engine.md` ("The platform layer", "Data") and are not classified here:
- the eight replaced source files;
- COM calls that fail, so there is no sound, input or 3D device;
- `debug_fatal` unwinding;
- simulated time;
- Windows text-mode `fopen`;
- 1×1 substitutes for missing artwork;
- no registry;
- round-toward-zero at every entry;
- the generated or retail-plus-community 3D data;
- the MinGW C runtime.
