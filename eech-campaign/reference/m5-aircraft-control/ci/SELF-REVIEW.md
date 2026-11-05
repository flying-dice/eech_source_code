# PR98 windows-cross apt source fix — author self-review (Claude, not independent review)

Base: 6e837440 on codex/m5-aircraft-control. Only file changed: .github/workflows/eech-campaign.yml (Toolchain step of windows-cross).

Evidence (reported, from retained windows-ci-job120525266110.log / ...087727.log): apt-get update used
`file:/etc/apt/apt-mirrors.txt` (a `mirror+file:` source) which resolved to http://azure.archive.ubuntu.com/ubuntu;
package downloads ran ~1-10 min each and were cancelled at the 40 min timeout; the one automatic fallback to
https://archive.ubuntu.com/ubuntu (job ...087727, line 232) completed within a second.

Change: before `apt-get update`, rewrite `mirror+file:/etc/apt/apt-mirrors.txt` and
`http(s)://azure.archive.ubuntu.com/ubuntu[/]` to `https://archive.ubuntu.com/ubuntu` in /etc/apt/sources.list,
*.list and *.sources (depth <=2). Covers both deb822 (URIs:) and one-line (deb ...) forms.

Preserved: suites (incl. noble-security), Signed-By/signed-by, GPG verification (nothing disabled), other
sources (packages.microsoft.com, security.ubuntu.com, ...), packages, versions, update-alternatives, rustup,
cargo test command, timeout-minutes 40, other steps.

Checked (see validation.log): js-yaml parse of the whole workflow; `bash -n` on extracted step; the exact
find/sed run on a fixture tree (deb822 + one-line + unrelated sources) gives the expected diff and is idempotent.

Limits (not verified):
- The runner's real source file contents were not inspected (no runner access); the form is inferred from the
  `Get:1 file:/etc/apt/apt-mirrors.txt Mirrorlist` log line. If the form differs the sed is a silent no-op,
  and the job behaves as before.
- archive.ubuntu.com may itself be slow for a given run; only a new CI run can show the effect.
- noble-security now comes from archive.ubuntu.com (which serves it) rather than via the mirror list.
- No native rebuild / test rerun (workflow-only delta). GNU find/sed behaviour tested under Git Bash, not Ubuntu.
- No actionlint available locally.
