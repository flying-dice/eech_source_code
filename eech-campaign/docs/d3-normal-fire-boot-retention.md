# Normal-fire boot retention correction (pending SAME review)

Required blocker: `ENV-NORMAL-FIRE-BOOT-RETENTION-1`, under the still-open
GenericNative1 parent. This corrects the diagnostic observer, not campaign policy.

## Actual failure and accepted evidence

Root's reviewed426 attempt ended with healthy **FAIL**, before frame1:
`normal-fire: undrained model entries`, in an actual boot `entity_scan` callback.
It is not UNREACHED. B was never prepared or launched. Root binding
`normal-fire-first-native-426de-Root-binding-20261108.json` (`31446aeb…`) retains
actual exits/absent original processes, unchanged97 payloads/10 artifacts and
four1179-file input checks. The preceding default native0 control has all three
COMPLETE outputs literal-equal to both retained3ab and80fd; reuse, do not replay.
Prior three source P2s are closed by3c9b/24a9; outer P2s by e887→60c9.
Exact426 all9 CI and compiled2fc build/profile remain their own accepted scopes.

## Source lifecycle and bounded correction

`mb_hostmodel.c::ask` calls `NF_MODEL_ENTRY` BEFORE each real provider callback.
`eech_wpn_fixture.c::eech_nf_model_entry` increments its global ordinal every time,
but emits a `normal_model` record only while `nf_step.query_active`. Boot's model
and entity-scan loops therefore consume ordinals without equivalent event storage.
The Lua collector wrongly allocated/budgeted one pending record for every call.
It also retired a preceding feed only at the next frame's feed, after that
next frame's callbacks had already happened. Boot and frame1 share frame/tick1/0.

The collector now retains exact ordered **same-op runs**, preserving every call
ordinal and op. The existing100000 retained-record resource budget is unchanged;
it now limits runs, not the number of callbacks in a repeated operation. This is
a bounded diagnostic memory policy, not a guarantee for arbitrary alternating
traffic or proof of a complete future retail boot. Alternating100001 runs still
fail and poison all continuation surfaces. Query before/after/call/op/sequence
checks remain exact; compressed ranges cannot supply a different operation.

`Engine::wpn_events` loops over the real C getter until `more==0`. Only AFTER that
complete public drain, successful `feed` and selection does the driver acknowledge
`capture:drained()`. It rejects an open interval or a callback after feed, then
retires old op storage before new callbacks. Partial feeds retain their op proof;
they do not acknowledge a complete drain. C buffer/overflow/sequence behavior is
unchanged (`NF_EVENTS8192`, overflow remains an explicit normal_error).

`eech_engine_boot` runs `boot(config)`; `eech_engine_frame` separately runs the
entity update loop. Weapon step/query hooks are on actual weapon movement/lookup.
This diagnostic explicitly requires no weapon query in its boot drain and fails
if one is observed; it never silently discards a boot query or credits it as a
moving witness. That restriction is checked, not asserted as universal boot lore.
Boot still delegates and validates scene/roster/flag effects exactly once.
It retains no query-answer dictionary. During frames only candidate and relevant
GROUP/READ answers are retained, then cleared after complete drain/selection.
All model op ordinals, including unretained answers, still participate in the join.

## Focused controls and remaining gate

OldLF RED101 uses a short real reference-provider positive first, then the exact
100001-call failure. GREEN uses200036 real reference FlagStore/provider calls:
short boot, long boot, long first-frame traffic before its feed, and two query
handoffs across frame1/2. Query framing here is synthetic over retained C physical
fields: no new C/retail/native boot claim. Further controls cover partial feeds,
early/late drain ACK, wrong compressed op, alternating overflow and storage reuse;
failures latch across six surfaces. Full synthetic driver healthy/B with100001
extra boot calls validates exact global ordinals; an observed boot query fails
before frame1. The intermediate nil-record correction failure is retained too.
Independent Python checks accept coherent global ordinals above100000, preserving
the full reference/controller gates. Existing semantic/prefix/reset controls stay.

Whole3816 C/Rust product/build/Cargo/static rows and two producer files remain
joined to compiled2fc. No new build, default/native replay, physical input change,
baseline/seed/window change or binary relabel. Corrected literals are separate.
STOP for SAME changed-material review, then Root owns exact materialization and
any later reviewed launch. All ten requirements/preparation2/GenericNative1,
physical cutover, fullpacks, Task49 and whole D3 remain OPEN.
