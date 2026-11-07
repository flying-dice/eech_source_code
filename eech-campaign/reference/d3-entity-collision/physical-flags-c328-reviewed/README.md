# Physical-flags characterisation c328: the ONE consolidated review and Root's bindings (literal records)

Task EEDCS-49 (D3), 7 November 2026 (UTC). Exact bytes of sealed records (`* -text`); aggregate records only: no raw retail data, actor, sector or pose tuple is in Git.

- **The review `CHANGES_REQUIRED_PROSE_ONLY`** of the characterisation `c328d8bfb` (published at `69f400522`): the measured 199 expectations are VERIFIED and no executable correction is required; one P2, `ENV-PHYSICAL-FLAGS-HIGH-INDEX-PERSISTENCE-1`: the contract and the control's comments call a high-index (>= 131072) UPDATED bit permanent/for-life. It SURVIVES the quarter-memset reset, but the same `&=` getter clears high-index neighbours, so it persists only until a destructive neighbour read or another overwrite. The qualification of the supply-descriptor explanation follows. No production repair, boolean normalisation or replay is asked for the prose.
  The review's `smallest_required_next_consumer_control` names the real immediate consumer (`fw_msgs.c` `response_to_check_mobile_reached_guide`, through `message_responses[FIXED_WING][CHECK_MOBILE_REACHED_GUIDE]`); `immediate_consumer_gate_met` is false and no production flag cutover, native launch, layout/ABI/store decision or supply-descriptor fix is cleared.
- **Root's c328 binding** and **Root's 69f publication record**: the independent verification of the freeze, the maps and the packet, and of the publication commit. The immutable original freeze, readers, failed attempts and the erratum remain as published.

Not a review of any later change, an approval or an acceptance of whole height, Task49 or D3.
