# EEDCS-43 full deterministic validation

Runtime source `a2a041fecf52e18cbc16b33f3dc75a9c9b245c9d`, ordinary Windows x64
build. PR102 head before this evidence was `e7e46dcc2`; its runtime/tool trees
equal a2 exactly. Accepted LOS predecessor is integration `3ce17432e`.

All four established native campaigns completed. The separate completed checker
recovery passes the **same40 checks**; all **eight complete artifacts** equal the
verified destination baseline, and all three post-input manifests match.
Independent recovery review found no actionable issue. This directory does not
itself assert merge/epic completion; current CI and normal merge remain delivery
gates. See the delivery review and live PR/Jira state.

## Original failure and separate recovery

The original full invocation (exec89376) actually exited **1**. Its PowerShell
children selected the Windows Store Python alias. The preserved original report
has exactly seven affected failed checks: the two regression exact comparisons,
reference observation/baseline/supply checks and two retained checker reports.
It is not a passing original invocation.

The unchanged native-exit guards reached their postcheck blocks; no native-failure
exit occurred. Complete recorded bytes and explicit repo pre/post input checks
also match. The original reference summary's misleading input-difference message
came from failed interpreter commands; that original failure text is retained.

Using explicit Python3.12.14, separate invocations executed the **unchanged** repo
PowerShell post-native blocks, then the frozen pack's unchanged post-run body and
report writer. Reference-only recovery ran before Georgia finished; its historical
metadata says Georgia was then live. Final recovery ran after the observed original
exit1 and required precisely the seven known failed checks. It reused the original
passing retained/lifecycle/preflight stages and actual native outputs. It did not
override failed fields, rerun campaigns, alter check definitions or adopt a baseline.
The final recovery process exited **0**, with40/40 checks passing. Original reports,
raw files, logs, launcher and checker sources were rehashed unchanged.

`recover-reference-postchecks.py`, `complete-postcheck-recovery.py` and their
generated PowerShell files are **audit artifacts specific to this recovery**,
not alternate simulators or a generic new regression suite. They retain exact
source/block identities. The original launcher fixed cwd but missed Python command
resolution; future launches must preflight child PowerShell's interpreter and cwd.
The separately authored `eech-regression` agent skill documents that launch path.

## Evidence and limits

- `original-pack-report.*`: actual failed original report, preserved.
- `recovered-pack-report.*`: unchanged pack writer's separate40-check PASS.
- `*-postcheck-recovery.json` and console logs: executed/reused stages, actual
  checker returns and unchanged-original hashes.
- `complete-comparison.json`: all8 complete files, not prefixes. Each ACMI is
  117690861bytes/SHA3bd17b647665b558c7378206a3717c94e9cf771d4781d78e0b2aaae9a7d2d19b;
  each observation file85716447bytes/SHAda26bab01163aa4ab725f9f2ecb516d37a4558777fe3684222b8d885822e77cf.
- BUILD-INFO retains the exact tested binaries/scripts/toolchain. Before/after
  input logs retain repo manifest identity; all six processes exited0.
- `copy-manifest.json`: public copies' raw external source/hash/size. Scoped
  `.gitattributes` preserves these audit bytes across Git line-ending conversion
  and recognizes intentional CRLF. Two original failed-tool logs retain literal
  spaces after empty fields and are exempt from whitespace lint; other checks remain.

The baseline is the supplied-binary destination run87915, actual0/40PASS, already
equal to accepted D1 recordings. It was reused. No accepted bytes, tolerances,
lineages or blind-spot definitions changed. Recordings were generated, structurally
checked and byte-compared; no visual Tacview inspection is claimed.

Existing source/authority/refusal evidence remains in the parent dossier. Physical
sensing range/envelopes/tolerance/emitter calculations moved to the host; campaign
filtering/ranking/engagement/awareness stay core. Aiming, weapon execution and combat
resolution remain D2 work. Terrain/model lookup, mirrors, private RNG, effects and
sound remain declared coupling. D3 has not started.
