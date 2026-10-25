# D2 combat and perception — 25 October 2026

D1 / EEDCS-15 is complete at integration a7934aa7d1bcffc2ea5057bb3f1b0af332606c61.
All 15 D1 tasks are Done. D2 / EEDCS-18 is now In Progress; first extraction
EEDCS-42 moves AI line-of-sight observations to the host while keeping target
selection and engagement policy in EECH_DC. Live authority: M5 goal 11534337 v3
and migration ledger 14548994 v26. All five epic report macros are preserved.

Final evidence: native 40/40, full Tacview and observations unchanged, metrics
equal; nine CI checks and independent exact-head approval before PR100 merge.
Coverage limits are preserved in the acceptance record and closure audit.

Workspace: codex/d2-perception-los from verified integration a7934aa7, plus the
documentation-only handover cherry-pick 756ed6326. Local regression bundle:
C:/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/eech-regression-20261025.
All 3,581 payload hashes pass; Georgia 1,174 and Lebanon/repeat 1,179 input files match
their checked-in manifests exactly. Read docs/handover/HOST-READINESS-20261025.md.

Now: one Claude CLI 2.1.310 implementation context
ae34a3d7-0931-4638-b06a-a2d399277929, exact claude-sonnet-5-5 High via -p, is
characterizing the existing LOS path before structural changes. No product code
has moved and no D2 Tacview comparison has run. D1 experiments are not repeated
for readiness. One independent Codex lead review follows the concrete candidate.

Build prerequisite: existing Docker Desktop first-run Subscription Service
Agreement awaits user handling; Linux daemon is not ready. Windows 11 x64 and
Python 3.12.14 are available. No numerical toolchain substitution or authentication
change made. Both GitHub connections identify fd-starscream-bot; normal delivery
also needs a distinct authorized PR-author identity, without self-approval.

Next: exercise original-behavior characterization with documented toolchain,
then bounded LOS extraction, focused boundary evidence, clean shipping build,
established deterministic pack and complete Tacview/observation/metric comparison,
independent review and normal merge. EEDCS-42 and D2 remain incomplete.

Later: remaining D2 work, then D3-D5 and M6 live DCS operation. Call-map PR83 is
unfinished historical inventory and needs a current-source refresh. Historical
ee-dcs TypeScript-port work is unrelated preservation, not the current roadmap.
