# 64-bit and platform findings

The C reference in eech-core-ts is a 32-bit x86 build (`-m32`), because it
depends on the i386 calling convention (`c-reference/README.md`, "Platform").
This document records what actually happens when the same campaign closure is
built for the native targets. The findings come from compiling and running the
code, not from reading it for old-looking idioms.

**Targets exercised**

| Target | Data model | Compiler | How it is run |
|---|---|---|---|
| x86_64-unknown-linux-gnu | LP64: `long` 8, pointer 8 | gcc 13, clang | native |
| i686-unknown-linux-gnu | ILP32 | gcc 13 (`-msse2 -mfpmath=sse`) | native |
| x86_64-pc-windows-gnu | LLP64: `long` 4, pointer 8 (the DCS platform) | MinGW-w64 gcc 13 | Wine 9 |

**Method**

- Every original unit compiles with `-Werror=pointer-to-int-cast`,
  `-Werror=int-to-pointer-cast`, `-Werror=int-conversion`,
  `-Werror=incompatible-pointer-types` and
  `-Werror=implicit-function-declaration`.
- The whole test suite runs on every target:
  - the corpus: 8,401 scenarios and 33,000 float operations, compared with the
    32-bit C reference's output;
  - the semantic scenarios, compared with results recorded on x86-64;
  - the findings tests.

On every target, native output == C reference output, for every scenario.

## Classification

| Id | Finding | Class | Evidence |
|---|---|---|---|
| **B1** | `en_creat.c` (`create_local_entity`, `create_client_server_entity`): `pargs_buffer = (char *) pargs` treats the `va_list` as a pointer into the i386 argument stack. The attribute readers (`en_attrs.c`, `get_list_item`) then walk it with `sizeof (TYPE)` steps. | **actual blocker** | `crates/eech-sys/tests/findings.rs` `b1_...`. On x86-64 System V a `va_list` is a 24-byte register-save descriptor, so the list reads back as `16, 48`: the descriptor's `gp_offset` and `fp_offset`, not the attributes. On Win64 `va_list` is a `char *`, but every variadic slot is 8 bytes, so reads of 4-byte items drift. Every creation goes through this code (supply crates, supply tasks, sectors), so nothing in the slice works without a fix. |
| B1 fix | Patch P1 (`docs/patches.md`): a marshaller walks the same attribute grammar with `va_arg` and writes the list `get_list_item` expects, into storage in the creating frame. | resolved | The corpus passes on all three targets. On i686 the whole suite also passes with the **original** line (`EECH_ORIGINAL_STACK_ATTRIBUTES=1`), so P1 is behaviour-neutral where the original is correct. |
| **T1** | `ai/highlevl/highlevl.h`: the release `ai_log` for `WIN32` is `#define ai_log();` and is called with arguments (`keysite.c`). Only MSVC's preprocessor accepts that; it gives warning C4002. GCC-family compilers reject it. | portability issue (toolchain) | MinGW build without the fix: 9 errors in `keysite.c`. GCC-family Windows builds pass `-UWIN32`, which selects EECH's own non-WIN32 GCC definition (`do { } while (0)`). No other `WIN32` use is in the closure. |
| T2 | `get_local_entity_index (EN)` is `(EN) - entities`, a `ptrdiff_t`: `long` on LP64, `long long` on LLP64. `keysite.c :: dump_keysite_info` prints it with `%d`. | portability issue, not reached (debug dump) | gcc `-Wformat` on x86-64. The native host layer made the same mistake (inherited from the harness); `-Wall -Werror` caught it and it is now cast. |
| T3 | The C reference observes `create_supply_task` with `ld --wrap`, which is GNU-ld only. | portability issue (harness) | The native build uses a compile definition on `fc_msgs.c` instead (`spec.rs` `UNIT_DEFINES`). |
| T4 | NULL-page faults: the C reference reports EECH's unguarded NULL dereference from a `SIGSEGV` handler. | portability issue (harness) | The native replay process uses `sigaction` on Linux and a vectored exception handler on Windows. The 24 recorded and 101 fresh NULL-dereference scenarios match on both. This is never installed in a library host (`docs/dcs.md`). |
| — | `sizeof (long)`: the only `long`s in the closure are two statics in `modules/system/time.c` that no extracted function uses. | not reached | closure scan |
| — | Pointer/int conversions | none in the closure | the build treats them as errors on all three targets |
| — | Serialisation of pointer-sized data: the attribute list holds `entity *`, but only in memory. The network and save-game packers write entities as safe indices (`en_attrs.c :: pack_entity_attributes` → `pack_entity_safe_ptr`). | harmless (in memory only) | source read; the pack/unpack paths are fail-loud stubs in the kernel |
| — | Structure layout: pointers are 8 bytes, so raw entity structs are larger. Everything is accessed by field. Bitfields are `unsigned int` (e.g. `group.member_count : 6`); their layout differs between MSVC and GCC but their semantics do not, and nothing depends on the layout. | harmless legacy assumption | corpus identical on ILP32/LP64/LLP64 |
| — | `#pragma pack (1)` in `3d/terrain/terrdata.h`, `3d/3denv.h` | not reached (types only named) | closure scan |
| — | `va_list` passed to message responses (`notify_local_entity` → `message_responses [..] (.., pargs)`), `va_arg` over route nodes in `create_task` | harmless (standard C) | corpus |
| H1 | `sc_seccreat.c :: create_local_sector_entities` allocates `sizeof (entity) * N` for an array of `entity *`. It over-allocates, more so on 64-bit. | harmless legacy assumption | source read |
| H2 | `time.c :: set_manual_delta_time`: `x = (++ x) % N` is undefined behaviour (unsequenced modification). Every compiler tested computes `(x + 1) % N`. | harmless legacy assumption (reached every update pass) | gcc `-Wsequence-point`; corpus identical across gcc/clang/-O0..-O3 |
| H3 | `up_update.c`: `memset (updated_entities, 0, MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8))` clears bytes, not words: a quarter of the bitset. `mb_int.c`'s getter is `updated_entities [index] &= mask`, which clears the word's other bits on every read. | behavioural (an original bug, platform-independent), not reached by the slice | gcc `-Wmemset-elt-size`. The bitsets serve mobiles' `INT_TYPE_UPDATED`/`MOVED`/`ROTATED`, and no mobile updates in the slice. It stays as written: the module preserves EECH behaviour, bugs included. |
| — | `__WATCOMC__` branch of `en_creat.c` | not reached (dead) | — |
| F | Floating point: see `docs/fpu.md` | no mismatch | — |

## What was not found

On the three targets, no behavioural difference appears anywhere the slice
reaches. That includes NaN payloads: the native module reproduces x86's
negative default NaN bits exactly, which the TSTL port deliberately does not
model.

The closure touched here is about 40k lines of the campaign core. The 64-bit
risk of the rest of the campaign (roughly 95k lines in eech-core-ts's kernel inventory)
is unmeasured. The method is in place for it: the pointer/int cast errors, the
closure scan, the corpus, and the per-target CI jobs.
