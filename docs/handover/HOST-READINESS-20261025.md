# D2 host readiness — 25 October 2026

This is reproduced host setup evidence, not acceptance of D2 or a rerun of D1.

## Authority and source

Read live through the existing Atlassian connection: M5 goal 11534337 v3,
migration ledger 14548994 v25, and EEDCS-18. D2 was To Do with no children.
Created EEDCS-42 (AI line-of-sight observations); both task and epic are now
In Progress. Ledger v26 records the bounded D2 scope and start; a full HTML
read-back confirms all five `jirachart` extensions remain.

SSH clone failed with `Host key verification failed`. No host-key bypass,
authentication change or credential copy was made. Existing GitHub CLI HTTPS
authentication successfully cloned `flying-dice/eech_source_code`, selecting
`codex/host-handover-20261025`: HEAD
`cb9703fa56eaab814b241afd5f3979f836d166c1`.

Fetched and inspected `origin/claude/eech-campaign-native-spike-tjkj6g`:
`a7934aa7d1bcffc2ea5057bb3f1b0af332606c61`, still accepted D1. Created
`codex/d2-perception-los` from that exact integration state and cherry-picked
the handover-only commit as `756ed6326`. Its four documentation files do not
alter the accepted runtime. Read HOST-HANDOVER.md, STATUS.md, AGENT.md,
AGENTS.md, PROJECT.md, docs/governance/REVIEW.md, the PR template and
docs/handover/D1-CLOSURE-AUDIT.md. Historical pending statements in that audit
are superseded by its final acceptance and live ledger.

## Local regression bundle

Copied `\\192.168.8.1\fs1\eech-regression-20261025` to
`C:/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/eech-regression-20261025`
with robocopy `/E /COPY:DAT /DCOPY:DAT /R:1 /W:1 /NFL /NDL /NP`.
Copy summary: 3,582 files, no failures or mismatches. Read START-HERE.txt and
HANDOVER.md. Retail payloads remain outside the repository.

Bundled Python at
`C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe`
reports 3.12.14. Running the copied `verify-bundle.py` by its absolute path
returned `PASS: 3581 payload files match`. An initial relative-path invocation
used the wrong parent and failed to find the verifier; the absolute-path
invocation above corrected that setup error.

From `eech-campaign`, ran `tools/input-manifest.py` with that Python against
each local root and the checked-in reference manifest:

| Root | Reference | Actual result |
|---|---|---|
| georgia | reference/m4-s3-mutation/inputs-georgia.json | IDENTICAL; 1,174 files, 142,522,166 bytes; combined SHA-256 d373bc4262464da87532b8128d396358f0003bc2e9e51e8fa92cf0a6e2b50de2 |
| lebanon | reference/lebanon-3h/inputs.json | IDENTICAL; 1,179 files, 102,239,559 bytes; combined SHA-256 121a789003b68f9ec4d312f82afafdbb6c22be4a6c71720d66d4aeeaa4259c09 |
| lebanon-repeat | reference/lebanon-3h/inputs.json | IDENTICAL; same file count, byte count and combined hash as lebanon |

D1's final 40/40 pack, complete Tacview/observation equality and matching
campaign metrics are reused at their recorded provenance and coverage limits.
No D1 runtime experiment was repeated merely for this handover. No D2 candidate
pack or Tacview comparison has run yet.

## Toolchain and access

OS: Windows 11 Pro, 10.0.26200, 64-bit. Supplied Windows x64 binaries match
the destination OS. Git Bash exists at `C:/Program Files/Git/bin/bash.exe`.
Docker client 29.1.3 is installed; Docker Desktop 4.57.0 was stopped. Started
the existing application hidden. Its first-run Subscription Service Agreement
is pending user handling; the Linux daemon is not ready. WSL defaults to v2
and currently lists no installed distributions. No alternative numerical
toolchain has been substituted. The repository build path remains
`eech-campaign/tools/build-windows.sh` with `tools/Dockerfile`.

Atlassian read/write access works. GitHub CLI and connected GitHub both report
`fd-starscream-bot`. This is the mandatory lead-review identity, so a distinct
authorized author connection is required before creating the delivery PR under
the repository's normal review process. No self-approval or identity workaround
is authorized by this readiness record.

Claude CLI 2.1.310 authentication reports logged in using its existing Max
subscription. `--help` confirms `-p`, `--model`, `--effort high`,
`--permission-mode bypassPermissions`, and JSON output flags. A minimal real
request using `--model sonnet --effort high` succeeded; runtime initialization
and model usage both name `claude-sonnet-5-5`. The sole implementation lane
uses that exact model with High effort, `-p` and bypassPermissions. No old-host
session is resumed and no fallback model is configured. No Opus planning was
needed. The lead remains the single independent Codex reviewer of the candidate.

## Next bounded action

EEDCS-42 characterizes existing LOS before structural changes. Execute the
original-behavior witness with the documented toolchain when Docker is ready,
then move the physical LOS solver and return value into the host while keeping
target policy in EECH_DC. Terrain/object access is declared D3/D5 coupling.
Only after focused evidence: clean shipping build, established deterministic
pack/full-file comparison, one independent review, normal PR/merge and task
completion. D2 remains incomplete; D3 has not started.
