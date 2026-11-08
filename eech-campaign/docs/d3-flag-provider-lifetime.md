# Flag provider lifetime: supported ownership and one missing join (UNREVIEWED)

Task49 / D3, starting source `667ff97e5f619ec67d6a3e6c362d8c67181581a1`.
One finite characterization; generic NativeAcceptance1 and whole D3 remain open.

The safe Rust API consumes `Option<ModelProvider>` into thread-local `MODEL`.
The boxed `FnMut` has an owned, static environment. `BOOTED.swap(true)` rejects
every second attempt before it can replace MODEL, including after an argument
error or a failed C boot. MODEL is assigned before CString argument validation.
`Engine` carries `PhantomData<*const ()>` and is neither Send nor Sync: the normal
engine methods and provider run on the owning thread. TLS lives until thread
exit; this is not a transferable process-global Rust closure or support for
destroying/recreating the owner thread, Lua VM, host binding, or engine session.

`with_host` constructs a temporary `HostContext` for each call. It offers the
static `model_callback` only while MODEL has a provider; the model callback reads
MODEL rather than dereferencing that context's `user`. Real C `ENTER` refreshes
the callback, and `campaign_model_set_host` stores `user=NULL`. A NULL-host entry
keeps the callback. Neither normal LEAVE nor the fatal return clears it. Other
per-entry roster/movement callbacks are cleared. A fatal poisons the engine;
subsequent entries return before consulting any provider. These are source facts
and, for the finite join below, behavior exercised by the fixture.

The Lua module's `model_provider` owns its bridge LuaFunction in the boxed
closure; `LuaEngine` retains the other callback handles. The reference host
constructs one SharedRoster, whose SectorRoster owns FlagStore. Its `clear`
forgets entities and heights but preserves flags. The reference provider's
`flag_event` obtains this SharedRoster from the live Lua app data. The C group-key
table is process-lifetime and also survives roster resets. These are CURRENT
source dispositions, not measurements of Lua VM destruction, thread exit,
successful boot, arbitrary custom hosts, or callback/session replacement.

**Supported domain:** one persistent provider/host/store binding on the owning
thread, with one boot attempt per process. Replacement is unsupported/unproved.
The C table may contain `fN` keys that coincidentally exist in a fresh store and
alias another group; fresh-store refusal is not guaranteed. No epoch, reset,
replacement registry, cross-thread API or semantic correction is introduced.

**The new finite control** (`physical-flags-lifetime-check`): one engine UNIT
test uses its OWN MODEL, model_callback and with_host. The dev dependency world
is used only through `flag_answer(SectorRoster, phase, raw words, identity)`;
no second engine provider/enum type or copied validator crosses that join.
An actual safe Engine::boot attempt with a NUL map path fails in Rust argument
validation before ANY C/retail boot, after consuming the provider. A Weak handle
then demonstrates that the provider owns its real host store after the boot call
returns; a second boot attempt is rejected and its proposed provider is dropped.

A thin private C entry, wholly behind the feature, uses the EXACT production
ENTER/LEAVE macros and actual campaign_model_flag_set/read. It intentionally
permits preboot use: no entity pool or booted session is manufactured. A healthy
set/read makes nonzero real callback/store traffic. After that entry's borrowed
HostContext ends, a NULL-host entry reads the same key/word/store. An argument
error asks no provider and leaves a later NULL-host read working. The matched
negative withholds that same read's answer with `[fl667:withheld]`; it reaches
the real Rust callback, returns a named fatal, leaves the owned provider alive,
and the next poisoned entry makes zero callback calls. All cases share ONE
isolated test process; no BOOTED/poison/key/store reset is used to allow them.

This is a characterization that may pass before any move. It is NOT successful
Engine::boot, public campaign flag READ/changed-fact authority, message dispatch,
native/Windows execution, Lua bridge execution, store replacement, callback
reentrancy, thread/VM destruction or full lifecycle acceptance. The scripted
environmental input is the one withheld provider answer. Private preboot access
and one dedicated process are explicit limitations, not a new shipping API.

**Reuse and FPU limit:** Task50's [model authority record](m5-model-host-authority.md)
at the starting Git retains the original callback installation contract and the
measured native wrapper defect/repair (`4bee2cc8c` / `3833c822d`, normal and fatal
model entries, x87 control/status and MXCSR). That historical evidence and its
literal reference packet keep their own scope; no old test is replayed or
relabelled as a current flag-entry measurement. The new control does not measure
the caller's complete FPU state, global restoration, RNG, successful current
native callbacks, or Lua VM lifetime. Those gaps stay explicit. The accepted
motion SAME774419f8, Tacview SOURCE7a4/873c and supply43fe proofs are reused at
their exact finite scopes without rerun.

The C engine source changes by one test-header include; both manifests, the
feature build define, a gated unit module and source guards have new raw source
identities. The original NUL in eech_engine.c is retained. Default feature-off
compilation and the affected lint/checks require their own current results;
retained 3abf Windows binaries are not relabelled. All ten D3 requirements,
public native READ, FPU/lifetime/DCS gaps and current full shipping acceptance
remain open pending their own evidence and SAME review.
