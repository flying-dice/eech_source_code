# EEDCS-40 author self-review (not an independent review)

Base `6e837440`. The parent is the sole independent reviewer; the technical lead has not approved.

Checked by reading the diff against the original handlers:

- Message order is unchanged: the setters still send before writing the parent position, so the
  host sees the parent's OLD position (what `resolve_dependent_parent` expects).
- `dependent_step.c` follows `hc_msgs.c`: offset from the parent's OLD position, rotation by
  `heading - en_heading`, position, then heading (attitude rebuilt), then velocity.
- An unhosted child, or a hosted child with an unhosted parent, runs the original code unchanged.
- `applying_host_result` depth counter: `++` on begin, guarded `--` on end, `> 0` checks in
  place, rotor_set and vehicle_vector_seed.
- No host success is fabricated: refusals return errors and mutate nothing (tested).

Risks for the reviewer:

1. The DLL C is not compiled; the MinGW build and the Windows pack are the first real check.
2. A hosted child under an unhosted parent silently uses the original path (disclosed).
3. The position shape ignores `result.attitude` (the original does too).
4. The carry tests were written with the implementation; only the base witness is red.
5. rustfmt was applied after the last test run.
6. The `debug_fatal` for an unsupported hosted child type is a C path and is untested.
