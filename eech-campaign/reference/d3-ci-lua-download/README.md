# CI: verified pinned download of the Lua 5.1.5 archive (`ENV-CI-LUA-ARCHIVE-AVAILABILITY-1`)

Task EEDCS-49 (D3), 7 November 2026 (UTC). **UNREVIEWED; the finding stays OPEN until the SAME independent review AND an actual green CI of the corrected workflow (Root controls the rerun).** Source before: the checkpoint `04bd7e71b`
(finding registered OPEN, `blocks_cutover=false`) over the published head `1b89c0b0ab613d2f6a08ea02dd6612457efa8982`. A necessary CI reliability repair only, not a preparation batch (the count stays 2). It changes the workflow, one shell
helper and its selftest, one lesson and the records: **no C, Rust, Lua, model, physics, Cargo, Dockerfile or compiler/link input changed**, the frozen native `1b89` jobs and data are not relabelled, and nothing was built or launched here.

## The defect (actual, from the preserved failure records; `ci-failures-safe-summary.json`)

Exact CI run `42359965325` of `1b89c0b0a`: the `windows-cross` job `128366083321` failed in the step **Lua 5.1.5 as lua.dll**, before any compile or test: `curl` exit 28, connection to `www.lua.org:443` timed out after 135185 ms.
Root's single failed-only retry returned seven fresh jobs; `windows-cross` job `128369471075` failed at the same download after 136790 ms while every other job passed. No further blind rerun was made. The failure is one origin's availability;
the archive identity (SHA-256 `2640fc56a795f29d28ef15e13c34a47e223960b0240e8cb0a82d9b0738695333`, unchanged) is not in question.

## Inputs (`literal-bytes-check.json` compares the one literal copy to its ACTUAL Git index blob bytes)

- `sector-height-publication-1b89-CI-initial-download-failure-20261107.json` (SHA-256 `816bcd91...`, 52864 bytes) and `sector-height-publication-1b89-CI-retry-download-failure-20261107.json` (`52b2049c...`, 51783 bytes): Root's failure records. Each embeds the
  **whole raw job log** (runner provisioning, git fetch output), so they stay external and are **not copied**; `ci-failures-safe-summary.json` carries their identities, the jobs and conclusions, the failed job's steps and the three log lines of the failing
  step (the command, `curl: (28) Failed to connect to www.lua.org port 443 after 135185 ms` / the retry's own time, and `Process completed with exit code 28`).
- `lua-5.1.5-MacPorts-verification-20261107.json` (`be4adb3d...`): Root's actual HTTPS download of `https://distfiles.macports.org/lua/lua-5.1.5.tar.gz` (HTTP 200, 221213 bytes) with the exact existing hash. The archive file itself (`R:/d3-evidence/lua-5.1.5-MacPorts-verified-20261107.tar.gz`,
  re-hashed by `c0`) is used only for the focused checks and is **not copied, vendored or uploaded**.
- The SAME review `cbbd27d7...` of `e326a3572` is read in full but not recopied; it sits literally in `../d3-entity-collision/sector-height-proof-guards-reviewed/` together with the old normalized-stdout **dual-identity receipt**, which is unchanged
  (checked against `HEAD`; its hash is in `literal-bytes-check.json`). The three source findings there stay CLOSED_SOURCE/PLAN_ONLY and are not reopened for CI.
- Byte qualifications carried unchanged: **3 inherited Lua CRLF raw-versus-Git exceptions** and **4 Core filter-only qualifications**; no file was normalised.

## The change

- `eech-campaign/tools/fetch-pinned-archive.sh <dest> <sha256> <https-url>...`: tries the URLs in order, each with `--connect-timeout 15 --max-time 120 --retry 0`, `--proto =https --proto-redir =https`, no certificate bypass, no credentials. The transfer goes to an
  owned `mktemp` file in the destination directory (truncated before every attempt, removed on every exit); only bytes whose SHA-256 equals the pin are moved to `<dest>`, so a failed, corrupt, truncated or empty transfer can never become the archive.
  A valid existing `<dest>` is reused without a transfer; an existing invalid one is removed, never silently used. Exit 0 (pinned bytes at `<dest>`), 1 (no source delivered them; nothing at `<dest>`), 2 (usage).
- `.github/workflows/eech-campaign.yml`, job `windows-cross`, step **Lua 5.1.5 as lua.dll**: the bare `curl` and its check are replaced by the pinned variable, the helper with `https://www.lua.org/ftp/lua-5.1.5.tar.gz` first and
  `https://distfiles.macports.org/lua/lua-5.1.5.tar.gz` second, and the same `sha256sum -c` before `tar` (the pin now appears once, in the variable both use). Version, SHA-256, `-O2 -DLUA_BUILD_AS_DLL`, the link line and the matrix are byte-identical (`c6`).
  One added step `Pinned-archive downloader` in `evidence-selftest` runs the selftest (no network, no installation).
- `tools/Dockerfile` still downloads from lua.org alone: its native builders already built all three profiles with exit 0 and are Root's; not changed.

## Controls (actual helper, fake transport, the ACTUAL pinned archive)

`tools/fetch-pinned-archive-selftest.sh` puts a fake `curl` first on a process-local `PATH` and runs the real helper; every case runs in its own empty directory so a stray temporary file is visible. The matrix: the coherent positive (one transfer, bounded, https-only,
no `-k`/credential option, pinned bytes, nothing left over); the primary failing as timeout (28), HTTP error (22), a cut transfer (18), truncated bytes, wrong bytes or empty bytes, each followed by the fallback delivering the same pinned bytes;
seven all-sources-failing pairs (both sources are tried, exit 1, nothing at the destination, no temporary file; including wrong bytes followed by a silent exit 0, which must not reuse the stale transfer); an existing valid destination reused with zero
transfers; an existing invalid destination replaced by verified bytes, or removed when nothing delivers them; a destination with a space and a relative path; usage errors (no transfer, nothing created); a directory as destination. 81 checks.

| child (`runs/`) | what | result |
|---|---|---|
| `c0` | the archive file has the pinned SHA-256 (re-hashed, not trusted from the receipt) | exit 0 |
| `c1` | `bash --posix -n` of the helper and the selftest | exit 0 |
| `c2` | selftest over a generated payload (`sh`) | exit 0, 81 checks |
| `c3` | selftest over the ACTUAL pinned archive (`sh`) | exit 0, 81 checks |
| `c4` | the same under `bash --posix` | exit 0, 81 checks |
| `c5` | **mutation control**: in a temporary copy the SHA comparison is disabled; the selftest must fail | caught: 24 FAIL lines (exit 0 = caught) |
| `c6` | workflow structural check against the exact `1b89` text (only the intended hunks; flags, pin, matrix, Dockerfile identical) | exit 0 |
| `c7` | the REAL curl accepts the helper's option set: one source on a refused loopback port; the helper exits 1, curl exit 7, nothing left | exit 0 |
| `c8` | `git diff --check` | exit 0 |

The actual CI red is characterised by the two real failures; no artificial matrix was built. PRE `2026-11-07T10:17:42Z`, POST `2026-11-07T10:18:28Z` (recorded by `date -u` before each capture); the 3971-row input maps are equal and identical to the e326 final POST map.
An earlier PRE (`10:13:39Z`) / checks / POST (`10:14:27Z`) over the same source was discarded as a packet (its freeze reader copied the two raw-log-bearing failure records); the reader was corrected, the sequence repeated, and the first outputs are kept externally as `attempt1`. The source did not change in between.
`join-vs-e326-and-1b89.json` classifies every path changed against `1b89`: the workflow, the two tools, one prose file, the card and STATUS, and this packet; none is a compiled, tested or linked input (no `aphavoc/`, `modules/`, `eech-campaign/crates/`, Cargo file or Dockerfile).
The `0be1dd846` 16/49 matrix, the e326 affected runs and Root's native `1b89` evidence (ordinary and observer shorts, hosted scan) therefore keep their true identity and are reused unchanged. All audit readers were final before the PRE identity capture.

## Lesson recorded

`docs/governance/D3-FIVE-WHYS-20261102.md`: network availability is not source identity; a bounded mirror with a fixed cryptographic digest avoids blind job or build replays.

## Limits

The fake transport and the real archive prove the helper's decisions, not that lua.org or MacPorts is reachable from the runner, and not that the runner's `curl` and `sha256sum` behave as the local ones (the real curl option set was exercised only against a refused loopback
port). No YAML parser or actionlint exists locally: the workflow is checked structurally (`c6`); its real parse and the `windows-cross` result are the CI rerun, controlled by Root. If the MacPorts distfiles were also unavailable the job would fail closed, as it should.
Author only: no self-review, approval, merge or Task49/D3/native authority claim; `D3-CURRENT.json` was not edited. All ten D3 requirements, the M5 and M6 witnesses, `sector-height-host-move-open-facts`, the final current 40/all-8/final-4-inputs, CI, review, normal merge and
final actual census remain required. No D4/D5/TypeScript.
