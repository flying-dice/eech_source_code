# EEDCS-43 extraction: evidence for the host-owned physical sensing of the AI target scans (26 Oct 2026)

**Development stacking, not accepted integration.** EEDCS-42 (PR101) is not merged; its final candidate full pack was still running when this evidence was taken. The account, boundary and limits are in
[`docs/m5-sensor-characterization.md`](../../../docs/m5-sensor-characterization.md) (Part 1). Nothing here is the final full pack, an independent review or a merge. No retail asset, installation root or binary is in Git;
the roots and binaries named here are private copies under `R:/d2-evidence/sensor-characterization-baseline-20261026/` (R: is a subst of the parent workspace).

## Identities

| | |
|---|---|
| Base / stack | integration `a7934aa7d1bcffc2ea5057bb3f1b0af332606c61` + reviewed frozen LOS head `a42f1cfad` (merge `1fc0cdc62`); the composed original build `1fc0cdc62` (DLL `917fe0f2...7e5c`) is the red subject and the oracle's origin |
| Extraction source | `bf176c1af` (the cut) -> `d588656e3` (bit-exact double transport) -> `37989fa5f` (clippy argument-count allows). **All results below are from `37989fa5f`** except the two retained first-cause runs |
| Fixture build | `37989fa5f183abbffa122fb47535923a83f6bfa9`, `tools/build-windows-sensor-fixture.sh`, cargo volume `eech-win-target-d2-sensor-fixture-2` (fresh), 0 uncommitted inputs, exit 0: `eech_dc.dll` `53b301ce5e8866efc2d5070c23d9641578ea15f0704a6f5aa951e67a17789b43` (`builds/fixture-37989fa5f-BUILD-INFO.txt`) |
| Shipping build | same commit, the unchanged `tools/build-windows.sh` and Dockerfile image, cargo volume `eech-win-target-d2-sensor-shipping-2` (fresh, never a LOS or frozen-42 volume), 0 uncommitted inputs, exit 0: `eech_dc.dll` `2c0815ad12b1ac451ad031aa3658b80d95b97e901d1c139a03e2c16147e5dab0` (`builds/shipping-37989fa5f-BUILD-INFO.txt`) |
| Toolchain | repository `tools/Dockerfile` image in WSL2 `EECH-D2-Build`; rustc/cargo 1.99.1, MinGW gcc 12-posix `12.2.0-14+deb12u1+25.2+b1`, Lua 5.1.5 |
| Inputs | private fresh copies of `eech-regression-20261025/roots/lebanon` (source manifest IDENTICAL to `reference/lebanon-3h/inputs.json`, `checks/source-root-manifest.log`; every used copy IDENTICAL after its run, `checks/roots-manifest-after.log`) |
| Machine load | one new native process at a time; the lead's full-pack processes ran throughout, so wall times are not performance data |

## Files

* `red/`: the red control. The composed ORIGINAL build booted with `sense-provider.lua mode=blocked` (300 s): provider consulted 0 times, 185 weapon births, 6 losses. `traces/witness-37989fa5f-final.jsonl`: the 600 s witness trace (LF text copy; the raw run file's SHA-256 is `bfb8fab3...e37a`, the LF copy's `dec1daf0...f753`:
  two representations of the same bytes, not to be confused).
* `checks/strict-comparison-original-vs-extraction.json`: the corrected comparator's ACTUAL Python exits for the final witness (0 on run, probe, run-at-probe) and for the two first-cause runs in `intermediate/` (1: the tolerance limit one ulp low, then a stale-C fixture hook), with the first differences.
* `providers/`: `reference`, `blocked`, `far` (300 s causal) and `missing`, `bad_*` (negative contracts) summaries and the driver lines; `controls/`: the 900 s ordinary-feature shipping candidate against the retained supplied-binary reference, and its `observation-check.py` report.
* `checks/`: fmt and four clippy configurations (`fmt-clippy-37989fa5f.log`), the full `cargo test` output, the f64 transport test (its `--nocapture` line gives the 4,010 of 60,000 one-ulp misreads under round-toward-zero), the symbol check, and the scripts.

## Commands (each: one native process, a fresh root copy)

```
witness:   tools/sensor-characterization-windows.ps1 -Bin <fixture build> -Root <copy> -Out <dir> -Scenario lebanon_retail -Seconds 600 -FrameMs 100 -ProbeAt 30,300,590      (native exit 0 after 763 s)
causal:    tools/sense-provider-windows.ps1 -Bin <shipping build> -Source <root> -Work <dir> -Modes reference,blocked,far -Seconds 300                                          (273, 260, 247 s; exits 0)
negative:  tools/sense-provider-windows.ps1 ... -Modes missing,bad_begin,bad_nil,bad_state,bad_reason,bad_accept_range,bad_draw_for_range,bad_error -Seconds 120               (exits 0; the failure is the engine's)
900 s:     eech-world.exe <bin>/lua/campaign.lua root=<copy> scenario=lebanon_retail hours=0.25 record_every=10 checkpoint_every=3600 observe_every=60 acmi= observe= metrics= observe_supply=0   (15:14:12-15:32:09, exit 0)
```

## Hazards found on the way (reported, not fixed outside this task)

* The existing `exact_f64s` reader (formats at 17 digits) is not exact under round-toward-zero: 4,010 of 60,000 sampled doubles, 3,199 of the 32,768 draws `k/32767`, come back one ulp low. It is used by D1 for `frand1 ()` draws; D1's accepted evidence is unchanged and untouched here. The sensing dialogue uses `exact_f64_bits`.
* `eech-engine-sys` reruns its build script only for `csrc/`, `compat/`, `build/` and `sources.txt`; a campaign-source (`aphavoc/`, `modules/`) edit alone leaves stale C in a reused cargo volume. This lane builds identities on fresh volumes.

## Not done here

No final full pack, no independent review, no merge, no Georgia, no player-controlled target in a campaign run, no visual Tacview inspection, no batching.
