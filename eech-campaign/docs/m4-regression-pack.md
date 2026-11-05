# M4: the accepted M1–M3 regression pack

This record is part of M4 ([#31](https://github.com/flying-dice/eech_source_code/issues/31)). It answers: **what accepted M1–M3 behaviour would we detect if a structural change broke it, what would merely look different, and what could currently regress without this evidence pack detecting it?**

**Answer, in short** (45 accepted claims; the full matrix is [`regression/pack.json`](../regression/pack.json), and each run's status is in its report):

- **Detected, and named: 17 claims, directly protected.** A check that asserts the behaviour fails.
  - M1: the campaign mechanics of Georgia and Lebanon; Lebanon's airbase resupply, which is where S3 shows; the six lifecycle and failure cases, including the characterised missing-map crash.
  - M2: input identity; repeatability; Tacview and the observations agreeing; observation not perturbing the campaign.
  - M3: a complete resupply chain whose deliveries equal the metrics; a SUPPLY aircraft loss followed by a response and a next assignment.
  - M4: the accepted checks still reject S3's retained mutant output.
- **Would merely look different: 12 claims, baseline-sensitive.** An exact baseline or a retained artifact stops matching, but nothing names the behaviour.
  - M1: the exact 3-hour wars; S2's removal; S1; the portability and load-time corrections.
  - M2: the retained recording; the characterised fidelity numbers.
  - M3: the retained chain (32 deliveries, 31 complete), what the campaign does with delivered stock, the duplicate-crate defect, and the specific Mi-17 loss.
- **Could regress without the pack detecting it: 16 claims.**
  - **Evidence only (8):** W1, H1, F1, the compile-only changes, the rebuild, the kernel conformance, the plain M2 observations, and the working-directory hazard.
  - **Not covered (8):** frame-time poisoning, threading and callbacks; N3, N1 and N2, whose faults do not occur within 3 hours; U1; the longer runs; captures in the evidence; a visual Tacview review; and adaptive compensation for lost supply.
  - **Blind spots:** outside those claims, the blind spots below say what even a passing check cannot see.

**Demonstration:**
- **On the accepted state,** the pack passes.
- **On #68's S3 mutant,** it fails. It names the broken behaviour: Lebanon's airbase ammo resupply. It shows the baseline-sensitive claims as differing, S1 and S2 among them although they did not change. It keeps the resupply-chain check green although every delivery was fuel, which is a blind spot shown live.

## The pack

| | |
|---|---|
| **Inventory** | [`regression/pack.json`](../regression/pack.json). Each claim has: where it was accepted, its path, its protection class, the checks that judge it, its sensitivity evidence, and its blind spots. The file also records the accepted candidate and scripts, the scenarios, the input manifests, and the hash and revision of every retained artifact, checker and runner the judgement depends on. |
| **Command** | `python tools\regression-pack.py --georgia <root> --lebanon <root> --lebanon-repeat <second root> --bin <build>` (about 25 minutes). `--retained-only` runs the first tier alone, in about a minute, with no retail data. |
| **Output** | `report.md` (claims, checks, blind spots), `report.json`, and every run's outputs, under `--out` (default `target/regression-pack/`) |

**How a claim is reported:**

| Status | Meaning |
|---|---|
| `PASS - directly protected` | Every check that asserts the behaviour passed. |
| `PASS - baseline-sensitive` | The retained baseline or artifact the claim is judged by is reproduced. |
| `FAIL - directly protected` | A check that asserts the behaviour failed. |
| `FAIL - differs from the baseline` | The evidence differs; the pack does not say which behaviour changed. |
| `EVIDENCE ONLY - not automatically checked` / `NOT COVERED` | Taken from the inventory, never from a run, so these can never become PASS. |
| `NOT RUN` | The check needs the runtime tier. |

The pack is `PASS` only when both tiers ran and no check failed.

**The retained tier** needs no retail data. Any failure here stops the pack, because a build cannot be judged against inconsistent evidence. It checks:
- that every retained artifact, checker and runner has its recorded hash, and each input manifest its recorded identity;
- that the checkers reproduce the retained reports from the retained evidence;
- that the checkers' own self-tests still detect their injected discrepancies. These were one-off runs in M2 and M3; the pack now runs them on the retained evidence;
- that the accepted checks still reject the S3 mutant's retained metrics (#68);
- that Georgia's expectations still miss S3. This is the documented blind spot, and the check fails if it changes.

**The runtime tier** runs the existing checks on the build under test:
1. It compares each root with its manifest.
2. It runs `tools/lifecycle-windows.ps1`.
3. It runs `tools/regress-windows.ps1 -Exact` and `tools/reference-windows.ps1 -ObserveSupply` in parallel, each root used by one engine at a time.
4. It runs `tools/loss-chain-check.py` on the reference run.
5. It compares the fresh evidence with the retained.

Each check's logic stays in the existing checkers. The pack parses their verdicts, and imports `campaign-expectations.py`'s own check to report the Lebanon resupply expectations apart from the other mechanics.

One judgement moved:
- **The problem:** `reference-windows.ps1` checks non-perturbation against the Lebanon baseline. On a changed build, that comparison fails whether or not observation perturbs anything.
- **The fix:** the pack checks `M2.unperturbed` against the same build's unobserved regression run. The runner's own baseline comparison becomes part of the baseline-sensitive `M2.reference_reproduced`.

`tools/lifecycle-windows.ps1` gains `-Out`, so that two packs never share an output directory. No campaign, engine, baseline, expectation or tolerance changed.

## Provenance

| What | Recorded in `regression/pack.json` |
|---|---|
| **Accepted candidate** | `0288ece2`: `eech_dc.dll` `da72f128…`, `eech-world.exe` `314fde3d…`, `lua.dll` `d0c799fe…` (#19) |
| **Accepted scripts** | `0ea49a4a`: `campaign.lua` `4b1fa47d…`, `observations.lua` `68527ae6…`, `metrics.lua`, `lifecycle.lua` (#27). They are unchanged up to `5bd8d11d`. |
| **Build under test** | Each file is hashed and compared with the above. A different build is reported, not failed: a structural change produces one by design. |
| **Scenarios** | Retail Georgia (map3) and Lebanon (map5), seed 1, 3 simulated hours, 100 ms frames, a checkpoint every hour. The reference adds 1 s sampling, Tacview and `observe_supply=1`. |
| **Inputs** | Lebanon: `reference/lebanon-3h/inputs.json` (1,179 files, `121a7890…`, M2). Georgia: `reference/m4-s3-mutation/inputs-georgia.json` (1,174 files, `d373bc42…`, #68). Georgia's inputs were first manifested in #68; that they are its baseline's inputs is inferred from the exact reproduction. |
| **Retained artifacts** | 17, each with its hash and the revisions that generated and committed it. Text is hashed as committed (LF), so the result does not depend on a checkout's line endings. The compressed recording and observations are also checked by their decompressed content. |
| **Checkers and runners** | 13, each with its hash and last revision, including `tools/regression-pack.py` itself, which classifies the claims and gives the verdict. Editing one without updating `pack.json` in the same reviewed change fails the pack. |
| **External data** | Retail Comanche vs Hokum (GOG) and Apache vs Havoc map3 (Steam), assembled by `tools/retail-cvh.sh` and `tools/retail-map3-installs.sh`. None of it is in the repository. |

**The pack fails loudly** (`reference/m4-regression-pack/fail-loudly.txt`), before any run, in each of these cases:

| Case | What the pack reported |
|---|---|
| One retained report altered by a byte | `evidence.artifacts: reference/m3-supply-loss/loss-chain.json has SHA-256 cbb48698…, recorded ed02be8b…` |
| A retained report missing | `reference/lebanon-3h/check.json is missing` |
| A checker edited | `tools/regress-compare.py differs from its recorded version (12e8f692)` |
| The pack's own runner edited | `tools/regression-pack.py differs from its recorded version (90acbacd …)` |

Each time it printed `RETAINED EVIDENCE INCONSISTENT: the pack cannot judge a build against it`, then `REGRESSION PACK: FAIL`, and exited with 1.

## Results (Windows 11, local, 2026-09-27, pack revision `90acbacd`)

Both runs are retained in [`reference/m4-regression-pack/`](../reference/m4-regression-pack/). They ran in parallel on separate roots, all with the accepted manifests.

**Revision note.** The reports say "with uncommitted changes". The only uncommitted file at the time was the draft of this record. The retained tier verified the pack's tools and artifacts by hash. `tools/regression-pack.py` itself is now recorded in `pack.json` at `90acbacd`, the version both runs used; the retained tier checks it too ([`retained-only/`](../reference/m4-regression-pack/retained-only/)).

### The accepted state: `PASS`

**What ran:**
- **Build:** `target/m3-reference-bin`: the accepted candidate's binaries (`0288ece2`) and the M3 scripts (`0ea49a4a`), byte for byte.
- **Roots:** Georgia (`d373bc42…`), and Lebanon's two M2 roots (`121a7890…`).

**Summary:** all 38 checks pass (13 retained, 25 runtime).

| Status | Claims |
|---|---|
| PASS, directly protected | 17 |
| PASS, baseline-sensitive | 12 |
| Evidence only | 8 |
| Not covered | 8 |

**Each accepted result is reproduced:**
- Georgia 30/30 and Lebanon 39/39, both IDENTICAL to their baselines;
- lifecycle: 5 PASS and 1 KNOWN DEFECT, unchanged;
- the two reference runs are byte-identical to each other, to M2's retained recording and to M3's retained observations;
- 31 complete chains of 32 deliveries;
- the Mi-17 87568 loss chain;
- every checker report equals its retained counterpart.

### The S3 mutant: `FAIL`, and the reason is named

**What ran:**
- **Build:** #68's mutant, `target/mutants/s3/mutant-bin`, not rebuilt. Its file hashes equal the retained [`mutant-BUILD-INFO.txt`](../reference/m4-s3-mutation/mutant-BUILD-INFO.txt): `56b164d7` with S3 removed, `eech_dc.dll` `fb113a37…`. Its scripts are the accepted ones.
- The pack reports it as a different build in `eech_dc.dll` and `eech-world.exe`.

**Summary:**

| Status | Claims |
|---|---|
| FAIL, directly protected | 2 |
| FAIL, differs from the baseline | 12 |
| PASS, directly protected | 15 |
| Evidence only | 8 |
| Not covered | 8 |

**What went red:**

| Claim | Status | The report's reason |
|---|---|---|
| `M1.lebanon.resupply`, `M1.corrections.S3` | **FAIL, directly protected** | `airbases resupplied with ammo (0)`. This names the behaviour. |
| `M1.georgia.war`, `M1.lebanon.war` | FAIL, differs from the baseline | Not identical: 11 (Georgia) and 13 (Lebanon) metrics outside tolerance. |
| `M1.corrections.S2_removed`, `.S1`, `.portability`, `.load_time_data` | FAIL, differs from the baseline | The same baseline differences. None of these corrections changed: **a baseline failure does not identify its cause.** |
| `M2.reference_reproduced` | FAIL, differs from the baseline | The recording differs (`d65f0ee7…` against `3bd17b64…`). The observed metrics leave tolerance, with 38/39 expectations. |
| `M2.fidelity_characterised` | FAIL, differs from the baseline | The observation report differs in counts, losses and deviations. Yet the check itself found 0 failures and the same −29.63 s drift: the war changed, not the fidelity. |
| `M3.resupply_reproduced`, `.feedback`, `.duplicate_crate` | FAIL, differs from the baseline | 32 → 30 deliveries; 31 → 27 complete chains. |
| `M3.supply_loss_reproduced` | FAIL, differs from the baseline | Only the list of *other* SUPPLY losses differs. The same Mi-17 87568 loss at 1,584.66 s occurs, but the three blue ground losses after 10,000 s do not. |

**What stayed green, and why:**

- **Correctly unaffected:**
  - `M1.lebanon.mechanics`: the other 37 mechanics still occur;
  - the six lifecycle cases;
  - `M2.inputs`, `M2.repeatable` (the mutant is deterministic) and `M2.tacview_observations` (0 failures). Observation does not depend on S3.
- **`M2.unperturbed`:** the mutant's observed run equals its own unobserved run. Judged against the baseline, as the runner does, it would have been reported as perturbing.
- **`M3.supply_loss`:** the same loss chain occurs in the mutant's war.
- **`M4.s3_detected`:** the retained tier.
- **Blind spot, as documented:** `M1.georgia.mechanics` passes, because Georgia's expectations do not assert resupply.
- **Blind spot, now demonstrated:** `M3.resupply_loop` passes although **all 30 of the mutant's deliveries are fuel**; in the accepted run, 10 of 32 were ammo ([`s3-mutant/chains.json`](../reference/m4-regression-pack/s3-mutant/chains.json)). The chain check cannot see what a task was for. Here only Lebanon's expectation and the baselines catch it.

So for a reintroduced S3:
- **Detected and named:** the Lebanon ammo-resupply expectation.
- **Looks different:** every exact baseline and retained artifact, including those of corrections that did not change.
- **Stays green:** the checks the defect does not touch, and one it does (the chain check).

## Blind spots

These are listed in `regression/pack.json`, and each report shows the claims each one limits.

| Blind spot | What it means for the pack |
|---|---|
| **Georgia resupply** | Georgia's expectations do not assert resupply. Without S3, all 30 still hold; only Georgia's exact baseline caught it (#68). The retained tier re-confirms this each run. |
| **Three hours** | Behaviour that emerges later cannot be protected: N3's fault at 39,327 s, a campaign's conclusion, the long-run supply balance. The longer runs are reported only. |
| **Unobservable supply** | Crates, a task's record (supplier, receiver, requested resource), request messages and group supply are not observed. Chains are attributed from position and supply changes, and a drop-off to a group is invisible. The chain checker pairs pick-up and delivery by resource, so it cannot tell an ammo task that delivered fuel from a fuel task (seen in the S3 demonstration below). |
| **Adaptive compensation** | M3's adverse evidence shows state damage and continued operation, not a response aimed at the lost supply. No keysite-bound loss, supplier or receiver loss, or capture-driven failure has been observed. |
| **Entity-index reuse** | Same-type weapons can merge into one track: 23 jumps and 12 launcher changes are lower bounds. The metrics' launched counts undercount by the same mechanism. |
| **Tacview limits** | Attitude is written only with a position or heading change: pitch up to 5.1° and roll up to 79.5° stale. A keysite's coalition is fixed at declaration, and its state and supply are not in Tacview. |
| **Clock, projection, sampling** | Session time drifts −29.63 s over 3 hours, and only about −4.6 s of that is explained. `ReferenceTime` is a label. The projection is an affine fit with a 1.1 km median residual, not re-verified. One-second sampling misses shorter events and their order. |
| **Reported only** | Most corrections, the longer runs, the M1 regen-disabled and lifecycle sensitivity checks, the rebuild check, and the kernel's i686 and Windows runs rest on code review or reported history, with no retained artifact. |
| **Retail data** | The runtime tier needs local retail data, so the complete pack is neither self-contained nor hosted-CI evidence. The retained tier runs without it. |
| **One war** | One seed per scenario. An exact comparison flags any change, harmless ones included: seed 2 moves 42 metrics beyond tolerance. A baseline difference alone does not classify the change. |
| **Coupled engine** | The campaign and the world are both inside `eech_dc.dll`. The pack cannot attribute a change to either, and says nothing about another host's world (M5). |
| **Captures** | None occur in 3 hours of Lebanon, so capture representation in Tacview and the observations is unchecked. Georgia's captures are seen only in its metrics. |
| **Plain observations** | The pack runs `observe_supply=1` only; the M2 settings' output is not re-run. |
| **Tacview visual review** | No recording has been viewed in Tacview; the pack compares bytes and structure. |
| **Linux** | The Linux baselines are stale and are not run. The repository CI is not evidence. |
| **Lifecycle, unverified** | Frame-time poisoning, threading and reentrancy, and host callbacks. |
| **Build identity** | A rebuild reproduces code and data but not file hashes, so a rebuild of the accepted source is reported as a different build. |

No blind spot needed fixing for the pack to state its confidence honestly, so none was fixed here.

## What this does not establish

- **Sensitivity for every claim.** Sensitivity is demonstrated for:
  - S3: both campaigns, and Lebanon's resupply expectation;
  - S2's removal: the Lebanon baseline;
  - the three chain and observation checkers: their self-tests.

  Elsewhere it is reported or not demonstrated, as each claim records.
- **A coverage figure.** The counts above describe the inventory, not EECH.
- **Automatic classification of a difference.** A red result names the claims and checks involved. Whether a change is a regression, a defect correction, an intentional change or expected variation remains a review decision.
- **CI or artifact distribution.** Both are out of scope; the pack runs locally.

## Reproduce

```powershell
# the retained tier only: no retail data, about a minute
python tools\regression-pack.py --retained-only
# the whole pack against a build: Georgia from tools/retail-map3-installs.sh, two Lebanon roots from tools/retail-cvh.sh
python tools\regression-pack.py --georgia <root> --lebanon <root> --lebanon-repeat <second root> --bin <build dir with lua\> --out target\regression-pack\<name>
```
