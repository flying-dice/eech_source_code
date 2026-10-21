# M5 native-error ABI candidate: retained evidence

This record supports the narrow correction in [the ABI note](../../docs/m5-dcs-lua-abi.md), under criterion 5 of Confluence 11534337 version 1. It does not establish DCS mission operation or M5 acceptance. The original microprobe and disposable product experiment remain at `E:/eech-runs/dcs-probe/abi-microprobe/` on the authoring PC; this directory retains the small outputs needed to review the committed candidate.

## Authorship and adoption

The executable product change remains commit `59a97bcabf132d7bd6be173267d33711ef9404bb`, authored by `fd-shockwave-bot <shockwave@flying-dice.com>` with `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>` recorded in that commit. Claude prepared the partially staged documentation corrections and evidence retention. An OpenAI Codex GPT agent finished and adopted that partial work by checking every retained file against its original, narrowing the measurement claims, and committing only documentation, retained output, and comments. This finish does not change the executable product inputs or transfer authorship of the product experiment.

## Identity and source boundary

- Product source: clean `git archive` of commit `59a97bcabf132d7bd6be173267d33711ef9404bb`, branch `codex/m5-dcs-lua-abi`. [candidate-BUILD-INFO.txt](candidate-BUILD-INFO.txt) records zero uncommitted build inputs, the pinned image `sha256:0c60c75db7b035e830c4834b1e15e096483631915bafc22c1af6aae02032bab3`, Rust 1.98.1, MinGW GCC 12-posix, and release `x86_64-pc-windows-gnu`.
- Candidate DLL: `c9d6a5d67eda9892e19e03ff2deb4edc56f365c9ab262796fe96c79ca187aa51`; host: `02c524cd28915742b2adbb22ec8a5df47b0961eb247a01dfdf43960adf6331e5`. Both hashes appear in the build record and each candidate control's `results.jsonl`; the `lua.dll` hashes are also in each `summary.txt`. The PUC Lua 5.1.5 DLL is `d0c799fe4e1e615d335d18504d28a7b0d7c78d7f97d1754bb885b1097ac60350`; the measured DCS Lua DLL is `8a228b5437520bc8dda3e77aac7bb7f81c10d23f0747adbcbafe95647533b755`.
- Stock red comparison: a disposable clean `65892de17bf5c4bb9ab6e9f8af70779e35b0106b` build with registry mlua-sys 0.6.8, in [stock-BUILD-INFO.txt](stock-BUILD-INFO.txt). Its DLL is `61e38ef21f4610995570147c6bfa2566d819799bca347d6f8170a65fc085d750`; host is `ae54d32fcbabb7155a355143919d3489dcfc81a0158fbf487edb4128ccd36c59`. This establishes the red symptom at the product's public Lua path, but it is an earlier source revision. The candidate's focused layout test separately fails against stock mlua-sys and passes with the patch; see [linux-tests-final.log](linux-tests-final.log).
- A recursive comparison (`diff -r`) of the `cargo vendor` copy of mlua-sys 0.6.8 (`build/vendor/mlua-sys-0.6.8` in the original experiment directory) with `patches/mlua-sys-0.6.8` finds only `src/lua51/lua.rs` changed among shared files. The vendor copy's `.cargo-checksum.json` is not kept and `PATCH.md` is added. The source change is `LUA_IDSIZE` 60 to 256, with a one-line explanatory comment.

## Measured layout

The same `lua-debug-layout.exe` (`9abbee772003faa5a8c72833b405c0cfe89b36f081c93845a32b62061a6bf633`) measured **316 bytes written** by DCS Lua and **120 bytes written** by PUC Lua: [layout-dcs.txt](layout-dcs.txt), [layout-puc.txt](layout-puc.txt). DCS disassembly puts `i_ci` at offset `0x138`, so the extent written is 0x138 + 4 = 316 bytes; the patched mlua-sys record is **320 bytes allocated** on x86_64 (316 rounded up to the struct's 8-byte alignment). The unchanged mlua-sys record was 120 bytes. The disassembly itself is not retained here; the written-extent measurement is. The tool has a finite 64 KiB canary buffer: a write reaching its last byte is reported as unmeasured, and a host writing beyond it is outside the tool's safe measurement range. These are one DCS DLL and one PUC DLL, not universal Lua ABI measurements, and not a bound on every Lua 5.1 record.

## Public-path controls

Run command: `tools/native-error-controls/run-controls.ps1 -Bin <build/out> -LuaDll <selected lua.dll> -Out <new result directory> -Reps 3`. Each control runs in a fresh hidden process with a 60-second bound. The runner records the script, host, module, and Lua hashes for every attempt. The retained `.summary.txt` and `.results.jsonl` files below are copied from the original runs; the example `.stdout.txt` files show the error value, traceback and collection markers.

| Product / Lua DLL | Result | Retained details |
|---|---|---|
| Stock / DCS | 9/18 pass: `empty`, `plain-error`, `load-only` pass; `first-error`, `host-log`, `lua-compat` each fault `0xc0000005` on all three repetitions | [summary](controls-stock-dcs.summary.txt), [per-run hashes](controls-stock-dcs.results.jsonl), [first error](stock-dcs.first-error.stdout.txt), [host log](stock-dcs.host-log.stdout.txt) |
| Stock / PUC | 18/18 pass | [summary](controls-stock-puc.summary.txt), [per-run hashes](controls-stock-puc.results.jsonl) |
| Candidate / DCS | 18/18 pass, including 100 callback errors, correct error text, traceback, GC and teardown | [summary](controls-candidate-dcs.summary.txt), [per-run hashes](controls-candidate-dcs.results.jsonl), [first error](candidate-dcs.first-error.stdout.txt), [host log](candidate-dcs.host-log.stdout.txt), [100-error control](candidate-dcs.lua-compat.stdout.txt) |
| Candidate / PUC | 18/18 pass with the same controls | [summary](controls-candidate-puc.summary.txt), [per-run hashes](controls-candidate-puc.results.jsonl), [first error](candidate-puc.first-error.stdout.txt), [host log](candidate-puc.host-log.stdout.txt), [100-error control](candidate-puc.lua-compat.stdout.txt) |

The stock comparison and candidate differ in source revision and binaries; the stock red run isolates the observed public-path crash, while the focused layout red test isolates the `lua_Debug` mismatch. Do not infer exact binary equality across those revisions. The earlier disposable patched build, `34ad0c2b…` DLL and `d4190538…` host, passed standalone controls and the M4 pack; its records remain under `product/` in the original experiment directory. This candidate has its own build and controls above.

## Linux checks and limits

[linux-tests-final.log](linux-tests-final.log) records the targeted stock-layout red test, then `cargo test --locked --no-fail-fast`, `cargo fmt --check`, and `cargo clippy --locked --all-targets -- -D warnings` passing on the candidate. The new callback tests run against normal PUC Lua in CI. No DCS Lua binary is in the repository, so the DCS controls are local evidence.

The change covers mlua-owned `lua_Debug` allocations used by traceback, stack inspection and function info.

**Borrowed PUC hook record: unsupported, not claimed safe.** A hook callback receives a record allocated by the host Lua; the patched Rust type borrows a 256-byte `short_src` from a stock host's 120-byte hook record, which outruns that allocation. The product never calls `set_hook`, so mlua's `hook_proc` is never installed and nothing here exercises it. Hook use, other DCS `luaconf` differences, other DCS builds, and in-mission behaviour remain outside this correction's proved scope.

## M4 pack on the candidate

[pack-candidate.report.md](pack-candidate.report.md) and [pack-candidate.report.json](pack-candidate.report.json) are the `tools/regression-pack.py` report for the candidate build (run 2026-10-21 09:40): **PASS**, `windows-retail v1` unchanged, 29 protected or baseline-sensitive passes, 8 evidence-only, 8 not covered. The report labels the pack revision `59a97bca... (with uncommitted changes)`; its own build section and [candidate-BUILD-INFO.txt](candidate-BUILD-INFO.txt) identify the build inputs as a clean `git archive` of `59a97bca` with 0 uncommitted files. The "dirty" label describes the runner checkout, not the built product, and is recorded here as reported. Only the two report files are retained; the run's recordings and copied roots stay local. The report notes the build differs from the accepted one in `eech_dc.dll` and `eech-world.exe`; the scenarios, roots and accepted baseline were unchanged, and no baseline was updated.

## Files, sources and hashes

Sources are under `E:/eech-runs/dcs-probe/abi-microprobe/` on the authoring PC (`product-fix/` unless stated). Every retained file was compared with its source by SHA-256 and is byte-identical. There is no separate manifest: the other reference directories list their files in a README, and this table is that list.

| File | Source | SHA-256 | What it shows |
|---|---|---|---|
| [candidate-BUILD-INFO.txt](candidate-BUILD-INFO.txt) | `target-59a97bca/out/BUILD-INFO.txt` | `b4354ae23be74d0d86ea2c30ef7fa967a0e560548085ebea22e9172451094cb0` | Candidate identity: clean git archive of 59a97bca, pinned image, toolchain, binary hashes. |
| [stock-BUILD-INFO.txt](stock-BUILD-INFO.txt) | `product/target-stock/out/BUILD-INFO.txt` | `0268ed94d9dabe7a8e5b463d5e8c01370d37ba380d5d460203236c6633cb20d9` | Stock red build identity: git archive of 65892de1, registry mlua-sys 0.6.8, binary hashes. |
| [layout-dcs.txt](layout-dcs.txt) | `layout-dcs/output.txt` | `6497ef96c791b27321fac2e813a8b16b092f709eb468034d2518c54629e1512e` | The DCS lua.dll writes 316 bytes of lua_Debug (one DLL). |
| [layout-puc.txt](layout-puc.txt) | `layout-candidate-mingw/output.txt` | `510aaf11ce260b3400b9e0b7d4b7840fc58ca0a3f709e10674059c35ba9d2eca` | The PUC Lua 5.1.5 lua.dll writes 120 bytes (one DLL). |
| [controls-stock-dcs.summary.txt](controls-stock-dcs.summary.txt) | `controls-stock-dcs/summary.txt` | `f47009870a1a39339eef142965162f7f31082a85b7b602d55a2be3d293132808` | Stock product under DCS Lua: first-error, host-log and lua-compat fault 0xc0000005 on 3/3 reps; 9/18 pass overall. |
| [controls-stock-dcs.results.jsonl](controls-stock-dcs.results.jsonl) | `controls-stock-dcs/results.jsonl` | `ab3efacbdfbf848da530db07f8e880a9d0f50ec0f2c6b6087d7e1c41ec9f4f3a` | Per-attempt exit code and script, host, module and Lua hashes, stock/DCS. |
| [controls-stock-puc.summary.txt](controls-stock-puc.summary.txt) | `controls-stock-puc/summary.txt` | `c54828330193d803b00ae2c2d0ddd275477e59e22be4a51bc0ff9b4a1bec97d0` | Stock product under PUC Lua: 18/18 pass (the defect needs the DCS record size). |
| [controls-stock-puc.results.jsonl](controls-stock-puc.results.jsonl) | `controls-stock-puc/results.jsonl` | `4f6b32c3913b5a54b1e556aca8d0148bf3a08e12231f3dbe50652d7c5aa15dd6` | Per-attempt hashes, stock/PUC. |
| [controls-candidate-dcs.summary.txt](controls-candidate-dcs.summary.txt) | `controls-candidate-dcs/summary.txt` | `93062e045e82f12cf7b0c7f7b68cc27169f7cbfd6debd368c02ded4467c4e4d6` | Candidate under DCS Lua: 18/18 pass. |
| [controls-candidate-dcs.results.jsonl](controls-candidate-dcs.results.jsonl) | `controls-candidate-dcs/results.jsonl` | `f87b3318fe9e3932999ad80f87b67bbb39b4c69c294c20edfcc22c305107ea07` | Per-attempt hashes, candidate/DCS (module c9d6a5d6..., host 02c524cd...). |
| [controls-candidate-puc.summary.txt](controls-candidate-puc.summary.txt) | `controls-candidate-puc/summary.txt` | `ea833631f8ab9f4787d553fac9af2cd875e16aa25f496f166ed8630e434000bd` | Candidate under PUC Lua: 18/18 pass. |
| [controls-candidate-puc.results.jsonl](controls-candidate-puc.results.jsonl) | `controls-candidate-puc/results.jsonl` | `520319750a982bcba8f46c15b5944ec6f7924b7981a1ebdb1d12293fee97ef91` | Per-attempt hashes, candidate/PUC. |
| [stock-dcs.first-error.stdout.txt](stock-dcs.first-error.stdout.txt) | `controls-stock-dcs/first-error-1/stdout.txt` | `965267c7010ff5cfa80c8c837dce00813e874edb50173544c33ac6a2be0aae0a` | Stock/DCS output stops at BEFORE_BAD_ARGUMENT (rep 1). |
| [stock-dcs.host-log.stdout.txt](stock-dcs.host-log.stdout.txt) | `controls-stock-dcs/host-log-1/stdout.txt` | `8ce124aee14d177937894843aa3252831003a79006ca70962180dbc5fad8ac5e` | Stock/DCS output stops at BEFORE_HOST_LOG_BAD (rep 1). |
| [stock-dcs.lua-compat.stdout.txt](stock-dcs.lua-compat.stdout.txt) | `controls-stock-dcs/lua-compat-1/stdout.txt` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | Stock/DCS lua-compat printed nothing before the fault (an empty file, rep 1). |
| [candidate-dcs.first-error.stdout.txt](candidate-dcs.first-error.stdout.txt) | `controls-candidate-dcs/first-error-1/stdout.txt` | `b88f4a10483284fbd38042f7958a69e967d3ef9c7dd2af794c2abad63462373e` | Candidate/DCS: error value, traceback and AFTER_GC for the first bad argument (rep 1). |
| [candidate-dcs.host-log.stdout.txt](candidate-dcs.host-log.stdout.txt) | `controls-candidate-dcs/host-log-1/stdout.txt` | `6a9305010c6635711fe142f59e704a603935943dc91f993f930eea5455e652a2` | Candidate/DCS: host.log({}) error returned to pcall (rep 1). |
| [candidate-dcs.lua-compat.stdout.txt](candidate-dcs.lua-compat.stdout.txt) | `controls-candidate-dcs/lua-compat-1/stdout.txt` | `1a86534822ce7ea849ceeb713b3cf2378d9c70badeba638b22aeaf76d30cff48` | Candidate/DCS: 100 failing callbacks, COMPAT_100_ERROR_CALLBACKS=PASS (rep 1). |
| [candidate-puc.first-error.stdout.txt](candidate-puc.first-error.stdout.txt) | `controls-candidate-puc/first-error-1/stdout.txt` | `a36b91979c708a270dabbb0933dc3af8c0183846e23417f2d79bb511a3a1228f` | Candidate/PUC first-error output (rep 1). |
| [candidate-puc.host-log.stdout.txt](candidate-puc.host-log.stdout.txt) | `controls-candidate-puc/host-log-1/stdout.txt` | `7cbd7d4d678fb20337812d92e29420274756ee59483e71a2c7f1e800ed0c3841` | Candidate/PUC host-log output (rep 1). |
| [candidate-puc.lua-compat.stdout.txt](candidate-puc.lua-compat.stdout.txt) | `controls-candidate-puc/lua-compat-1/stdout.txt` | `0577bed6fb835f98e41de79a02e159f69b637febf2f9d61f0d62db0953dea59a` | Candidate/PUC 100-error control output (rep 1). |
| [linux-tests-final.log](linux-tests-final.log) | `linux-tests-final.log` | `7545d7af10b2b06ab067d08203a531af4bc63c61c8fc24ac46c9e003f591060f` | Red: layout test fails against stock mlua-sys (120 < 316). Green: locked test run, fmt and clippy pass on the candidate. |
| [pack-candidate.report.md](pack-candidate.report.md) | `pack-product-59a97bca/report.md` | `0610655a9e1e50a5dddf815c1439825885d7ffcf1b5e2a3e78471454e0ebaf7a` | M4 pack PASS on the candidate build; windows-retail v1 unchanged. |
| [pack-candidate.report.json](pack-candidate.report.json) | `pack-product-59a97bca/report.json` | `35b2e0b4c73ee68698d57dfbdc07372d26e6a033633a7f954e4f5635a294858a` | Machine-readable form of the same report. |
