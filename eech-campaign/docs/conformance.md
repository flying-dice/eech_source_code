# Conformance: native module, C reference, TSTL

The hierarchy is unchanged:

1. the original EECH C's behaviour;
2. the executed C reference (eech-core-ts `c-reference/`, 32-bit);
3. the TSTL port, as an independent implementation.

The native module is checked against 2, and 3 is recorded alongside.

```
                    same scenario (C reference scenario language)
                              │
          ┌───────────────────┼────────────────────┐
          ▼                   ▼                    ▼
   C reference (i386)   TSTL port (JS)      native module (x86-64 / i686 / Win64)
          │                   │                    │
          ▼                   ▼                    ▼
       output ──── ts: "agrees" ────┐           output
          └──────────────── byte-identical ────────┘
```

## The corpus

`tools/corpus/export.sh` runs in eech-core-ts's toolchain (Node, vitest, the
canonical C reference build). It writes `corpus/*.jsonl.gz`. Each line holds
one scenario:

```json
{ "family": "supply-task-assignment", "id": "…", "input": "<harness scenario text>", "c": ["<C reference output lines>"], "ts": "agrees" }
```

| File | Content |
|---|---|
| `scenarios.jsonl.gz` (committed) | every hand-derived matrix case and every recorded random fixture of eech-core-ts slices 1–6a: 1,401 scenarios |
| `float32.jsonl.gz` (committed) | the recorded float fixtures: 3,000 float operations, 1,000 crate-row steps and 1,000 double sums, and the compiled aircraft database |
| `scenarios-fresh.jsonl.gz`, `float32-fresh.jsonl.gz` (not committed) | `EECH_CORPUS_FRESH=N`: N fresh random scenarios per generator (7 generators), N×28 float operations |

The scenarios are consumed **directly**: the corpus stores the exact text the
C reference reads, produced by eech-core-ts's own serialisers, so nothing is
re-interpreted. The `c` output is the canonical oracle's, executed at export
time. `ts` is the TSTL port's verdict on the same spec:
- lifecycle scenarios: the port's lines equal the C's;
- slice 1 and timelines: its structured outcome equals the C's parsed outcome;
- float operations: its bits equal the C's, NaN payloads aside, as eech-core-ts
  itself compares them.

## The replay

`eech_campaign::conformance::replay` runs a scenario through **the same kernel
`Campaign` uses**, with these adaptations:
- the scenario's physical state (aircraft positions, object bounds) is
  declared into a `ScenarioWorld`;
- the kernel reads that state back through the `World` boundary;
- events pass through the same translation as `step`'s.

The interpreter (`csrc/eech_legacy.c`) is a 64-bit port of the harness's main
loop:
- `create` lines become real variadic calls, marshalled by P1;
- output goes through a host callback;
- operations abort through the kernel's abort points.

The replay is behind the `conformance` feature: the scenario language
addresses internals (labels, raw fields, single operations) and is not an API.

A scenario whose C output is `result null-dereference` runs in a dedicated
process (`eech-harness replay`), which reports the NULL-page fault as the
harness does. Every other scenario runs in process, one after another. That
doubles as the test of sequential campaigns: the corpus reuses one process for
thousands of kernel opens.

## Results

| | Scenarios | native == C | TS == C | NULL dereference, via a dedicated process |
|---|---|---|---|---|
| assess-group-supplies | 291 + 1000 fresh | all | all | 24 + 101 |
| update-timeline | 172 + 1000 | all | all | — |
| entity-lifecycle | 194 + 1000 | all | all | — |
| keysite-cargo | 182 + 1000 | all | all | — |
| force-low-on-supplies | 187 + 1000 | all | all | — |
| supply-task-construction | 189 + 1000 | all | all | — |
| supply-task-assignment | 186 + 1000 | all | all | — |
| float32 / double sum / crate row | 5,000 + 28,000 fresh operations | all | all (NaN payloads aside; crate row not modelled in TS) | — |
| aircraft database | 33 entries | all | (checked in eech-core-ts) | — |

These results hold on x86-64 Linux, i686 Linux (with and without P1) and x86-64
Windows (Wine), and with gcc, clang, and `-O0` to `-O3`.

## Compatibility decisions carried over

| Decision | Made by | Native module |
|---|---|---|
| F1: `create_supply_task`'s uninitialised route heights are 0.0 | eech-core-ts slice 5b (the C reference zero-initialises that unit) | the same flag on the same extract. It is a shared decision, not EECH behaviour, and both implementations are pinned to it. |
| NaN payloads not modelled | TSTL | the native module reproduces x86's bits, as the C reference does |
| `convert_float_to_int` truncates (fistp under RTZ) | C reference and TSTL | the C `(int)` cast |
| `member_count` restored, not maintained (`gp_msgs.c` LINK_CHILD for members is not ported) | eech-core-ts 6a | `Campaign::new` keeps it equal to the members it restores |

No TSTL-only behaviour was adopted: where the corpus had a TSTL verdict, it
agreed with the C.
