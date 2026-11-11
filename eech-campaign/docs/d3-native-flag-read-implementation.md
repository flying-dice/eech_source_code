# nr395 opt-in READ witness — source and offline controls, UNREVIEWED

Implements the [accepted plan](d3-native-flag-read-plan.md), SHA256
`1c472eec613c09041491617d692c91b957568a21d18d9137c316ea1cfebe7352`.
SAME plan review `dde3b645...` accepted feasibility only; this source, its
controller/literals and any actual native evidence still require review.

New files: `lua/physical-flags-read-witness.lua`,
`lua/physical-flags-read-transaction.lua`, `tools/native_flag_read.py` and
`tools/native_flag_read_controller.py`. The old30ad driver stays byte-identical.
The new driver derives its exact word helpers, flag grammar, transcript oracle,
complete positive-reference relations and immediate-refusal latch from it.
It supports only the seven accepted READ modes, requires the explicit strategy
argument, and never implements a model, movement solver, flag store or Core API.

Before selection, all44 request words are classified by integer patterns in
the current domain (raw sector-height word12 is the documented exception).
The canonical delegated answer is obtained once and checked completely:
result/state legality, name, pose presence/count/finiteness and no metatable.
Names are compared against already-C-accepted boot SCENE answers observed by
the same callback/normal-return acknowledgement; no extra scene query is made.
The real bind_mobile result and descriptor tuple are recorded, never invented.

The first eligible real candidate is latched through only optional GROUP,
validated issued key, the acknowledged selected READ, and the same immediate
44-word re-ask with only UPDATED changed. Entry/frame/scene/name/identity,
registration/member/group/bind and candidate ordinals, flag key/array/lane and
the complete pre-READ phase tuple must match the healthy reference. Any unrelated
callback, missing re-ask, changed record or normal entry return before completion
fails. There is no fabricated generation field, second actor or window search.

The real READ is delegated exactly once before each valid change or fault. The
traffic-derived W→W&M transition is recorded separately from the answered word.
Changed0/mask affects Core's carried fact, not a second store write; the recorded
post-word is the immediate selected READ's oracle result, not final world state.
`reads_consumable` counts eligible real provider READs, including a subsequently
altered/withheld answer; it is separate from the actual carried tuple and accepted
reply counters. The callback continuation latch covers all ten functions supplied
at boot; the two default sample-provider aliases use the wrapped environment
function. A caught expected refusal is diagnostic success, not healthy campaign
continuation. No getter/counter/close is called through a poisoned engine.

`pf-read-* schema1` is a new canonical report family. It binds the runtime,
accepted plan, both Lua files, bundle manifest and healthy-file hash; exact
key/order/count/oracle/selection/store/full request-and-reask relationships are
validated before child output/root preparation/boot/provider work. The Lua
driver checks structure and semantic identity; the controller independently
checks the ACTUAL healthy file SHA from a single byte capture before child work.
Lua identity arguments are trusted only when this reviewed controller has checked
the real staged files. Direct invocation with invented identity arguments is not
an authenticated native result. Python validation is independently exercised on
the actual Lua-produced offline reports for all seven modes.

Offline tests reuse the prior scripted campaign collaborator literal and real
reference flag provider/store. Boot, SCENE acceptance and candidate decisions
are scripted. They cover real store/neighbour effects, changed carrying, optional
GROUP acknowledgement, a meaningful prior-latch counterexample, malformed full
requests before delegation, bad positive name/pose, all ten callable surfaces,
five immediate refusals, false continuation/normal return, damaged references
before effects, independent Python schema and controller inventory/hash gates.
They are NOT native campaign, successful boot, real scene loading/candidate
solver or public-runtime acceptance. Failed developer fixture attempts remain
retained as fixture errors, not manufactured product RED.

## Root materialization and the controller gate

The runtime remains EXACT ordinary HOSTED3abf, not the implementation Git HEAD.
`tools/native-flag-read-runtime.json` records the expected91 old payload rows
from the sealed build receipt; the file itself does not verify a new copy.
Root materializes a NEW91-payload binary directory and full external manifest,
plus only the healthy retail root. Both active fullpacks remain untouched.
The separate `literal_dir` must contain exactly the two final reviewed Lua files.
Children and their output directories remain absent until complete healthy PASS.

Concrete configuration keys are `schema:1`,
`state:MATERIALIZED_FOR_REVIEW`, fixed `runtime`, `source_plan_sha256`, exact
seven `modes`, `binary_dir`, `bundle_manifest`, `bundle_manifest_sha256`,
`literal_dir`, absolute `lua`/`protocol`, `pristine`, `healthy_root`,
`input_tool`, `input_reference`, UNUSED absolute `root_base` and `out_base`,
and `hashes` for controller/validator/binding/lua/protocol/input_tool/input_reference.
The bundle manifest is OUTSIDE the bundle, `{runtime,files:[{path,bytes,sha256}]}`.
The existing input tool's full immutable input rows are compared before/after
each process; engine-written files remain explicitly separated by that unchanged
tool and are retained in the full raw manifests.

Default invocation is READ-ONLY preflight. `--execute` additionally requires a
SAME clearance binding the exact materialized plan and controller SHA and both
Lua/validator/binding SHAs, with `native_controller_launch_clearance:true`.
That later review hashes the plan; the plan hashes code. No source hashes itself.
No unmaterialized placeholder or source-plan acceptance is launch clearance.

On actual Root execution, every child records real PID/creation time from its
Windows process HANDLE (GetProcessTimes), exact argv/cwd, direct exit and raw
stdout/stderr hashes. No process is restarted. A healthy UNREACHED exhausts the
fixed35-frame cap, records all six children NOT_LAUNCHED_UNREACHED, and stops.
Controller0 can therefore mean a correctly bounded UNREACHED run, never allPASS.
The fixture/controller test data are labelled synthetic and never executed as
a shipping/native program. Current source work runs no Windows/retail/native
campaign, creates no real child root, and changes no C/Rust production semantics.

Actual public READ/changed fact/refusal acceptance, natural moving integration,
all needed consumers, FPU/lifetime/DCS, wholeheight/sharedterrain/RNG/hierarchy/
services/final census/current40all8/fourinputs/currentCI/formalreview/merge and
all10/NativeAcceptance1/Task49/D3 remain OPEN until their own evidence and review.
