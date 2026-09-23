# Fidelity spike: EECH x87 rounding and precision semantics (issue #7)

Status: **decision C accepted; round-toward-zero migrated (§9).** Sections 1–8 are the spike's evidence as recorded before the migration. In them, "canonical" means the round-to-nearest oracle of the time, which is now the `sse-rn` variant. Since the migration, the canonical oracle and the port round toward zero at declared float type. The x87 intermediate-precision question is open and gated on a runtime trace (§9.3).

Everything else is investigation-only:

| Artefact | What it is |
|---|---|
| `c-reference/fpu-variants.mjs` | The oracle variants. |
| `buildHarnessVariant` in `c-reference/build.mjs` | Builds a variant into its own directory. |
| `HARNESS_FPU_VARIANT` / `HARNESS_X87_CW` / `HARNESS_MXCSR_RC` / `HARNESS_FISTP` in `c-reference/harness.c` | Compile-time switches a variant build sets. |
| `test/fpu-spike/` | The differential runs. |
| `c-reference/fpu-probes/` | The targeted probes and their assembly. |
| `npm run spike:fpu` | Runs all of it. |

Nothing is part of `verify`, and nothing writes to `test/scenarios/generated`.

Evidence labels used below: **proven** means shown by source, documentation or executed code. **Inferred** means reasoned from proven facts. **Needs a binary** means it can only be settled by the shipped executable or a runtime trace of it.

---

## 1. FPU environment (Investigation 1)

### 1.1 Control-word writes in the source

Every write to the x87 control word, found by searching for `_control87`, `set_fpu_*`, `fldcw`, `_fpreset` and `_controlfp`:

| Where | What | Thread (from the call path) |
|---|---|---|
| `modules/system/fpu.c:158` | `__8087cw = IC_AFFINE \| RC_CHOP \| PC_53 \| 0x7F` under `#pragma aux __8087cw "*"`. The Watcom runtime loads this at start-up and on `_fpreset`, so it applies to the **Watcom build only**. The comment says PC_53 was chosen "to generate reproducible floating-point results regardless of the level of optimization". | main (Watcom CRT init) |
| `modules/system/startup.c:396` | `set_fpu_rounding_mode_zero ()`: "The graphics / 3d / 2d systems need the maths fpu to round to zero". | main / system (message loop) |
| `modules/system/startup.c:802` | `set_fpu_rounding_mode_zero ()` at the start of `start_application`, before `application_main`. | **application** (`CreateThread`) |
| `aphavoc/source/init.c:884` | After `initialise_application_3d_system (); initialise_terrain_database ();`. | application |
| `modules/graphics/render.c:79` | After `initialise_direct_3d ()`. | application |
| `modules/graphics/dirdraw.c:157` | After `ddraw_initialise ()`: "Reset the fpu rounding modes!" | application |
| `modules/graphics/d3d.c:588` | `recreate_d3d`, after device-lost recovery: "Set the FPU to zero rounding!" | application |
| `modules/sound/dirsound.c:117` | After `dsound_initialise ()`. | application |
| `modules/multi/directp.c:219` | After DirectPlay enumeration: "Reset FPU mode". | application |
| `modules/3d/terrain/terrdata.c:1525` | Inside `get_terrain_3d_sector`, immediately before its `convert_float_to_int` calls. | application |
| `3dtrans.c`, `terrgeom.c`, `3denv.c`, `3dexplos.c`, `3dhoriz.c`, `clouds/3dclouds.c`, `clouds/3dtrump.c` | `set_fpu_precision_mode_single ()` (PC_24) at the start of each render section and `set_fpu_precision_mode_double ()` (PC_53) at its end. These come in matched pairs; the `3drain.c` pairs are commented out. | application (rendering) |

The helper implementations:

- `set_fpu_rounding_mode_zero` is `_control87 (RC_CHOP, MCW_RC)`, which changes the rounding field only.
- The precision helpers change the precision field only.
- `set_fpu_exceptions` and `set_fpu_mode_default` are empty (their bodies are commented out).
- `convert_float_to_int` calls `asm_convert_float_to_int` (`fpu.h`). That is `fistp dword ptr` under Watcom and MSVC, so it rounds **by the current control word**. The `set_fpu_rounding_mode_zero ()` inside it is commented out.

### 1.2 Direct3D and other libraries

- **D3D9 device flags.** `d3d.c :: direct_3d_device_create` creates the D3D9 device with `D3DCREATE_SOFTWARE_VERTEXPROCESSING`, plus `D3DCREATE_MULTITHREADED` in DEBUG builds and `D3DCREATE_ADAPTERGROUP_DEVICE` for multihead. It **never passes `D3DCREATE_FPU_PRESERVE`**. The DX7 path (`dresol.c`) has no `DDSCL_FPUSETUP` or `DDSCL_FPUPRESERVE` either.
- **What Direct3D does without that flag.** Microsoft documents that Direct3D then "defaults to single-precision round-to-nearest mode" ([D3DCREATE](https://github.com/MicrosoftDocs/win32/blob/docs/desktop-src/direct3d9/d3dcreate.md)). The change happens when the device is created ([Direct3D and the FPU](https://learn.microsoft.com/en-us/archive/blogs/tmiller/direct3d-and-the-fpu)).
- **Which thread that affects (proven).** The device is created, reset and released through `system_thread_function` (`d3d.c:309`, `:577`, `:614`). That function **`SendMessage`s the call to the system (message-loop) thread** (`sysmess.c:144`). The x87 control word is per-thread state, so the documented switch to PC_24/RN lands on the system thread, not the application thread that runs the campaign.
- **What EECH does afterwards (proven).** EECH re-asserts RTZ after every graphics, sound and network initialisation. It never re-asserts precision, except through the render-section pairs, which always end in PC_53.
- **Per-call behaviour (needs a binary).** The documentation is silent on whether the D3D9 runtime also rewrites the control word of *other* threads on per-frame calls (`BeginScene`, `Present`, `Draw*`), which EECH makes from the application thread. The source proves no per-frame re-assertion of RTZ on the application thread outside `get_terrain_3d_sector`. That function re-asserts RTZ on every call, which suggests the authors had seen the rounding mode drift.

### 1.3 Ordering relative to the campaign update

- **Loop order (proven).** `flight.c` runs each frame in this order: `receive_comms_data ()`, then `update_client_server_entities ()` once per time-acceleration step, then `draw_view ()`, then `ddraw_flip_surface ()`. The campaign update therefore runs on the application thread, after the previous frame's rendering, which ended every render section with `set_fpu_precision_mode_double ()`.
- **Initial state of the application thread.** The thread comes from `CreateThread`, not a CRT thread start, so neither `__8087cw` nor `_fpreset` applies to it (proven). It starts with the Windows per-thread default: round to nearest, PC_53 (inferred; this matches the documented MSVC default environment, [/fp](https://learn.microsoft.com/en-us/cpp/build/reference/fp-specify-floating-point-behavior)). `start_application` then sets RTZ (proven).

### 1.4 Conclusion for the campaign thread

| Property | Finding | Evidence |
|---|---|---|
| Rounding | **Toward zero.** Set at `start_application` and re-set after every library initialisation that could disturb it. | Proven by source. Stability through D3D9 per-frame calls needs a binary. |
| Precision | **PC_53** at the campaign update, unless Direct3D changes the application thread's control word per call. In that case it would be PC_24, with RTZ restored only by EECH's re-assertions. | PC_53 is inferred from source (render pairs end in PC_53; the thread's default is PC_53). The PC_24 alternative needs a binary. |
| Exceptions | All masked. `set_fpu_exceptions` is empty. | Proven. |
| Harness assumption "EECH rounds toward zero" | Legitimate. It is what the harness's `convert_float_to_int` comment already assumes. | Proven. |
| Harness assumption "EECH arithmetic rounds to nearest" | **Not supported.** It holds for no plausible historical environment of the campaign thread. | Proven for rounding; see §5. |

## 2. Compiler and evaluation semantics (Investigation 2)

### 2.1 Toolchains

| Toolchain | Evidence | Floating-point semantics | Confidence |
|---|---|---|---|
| Watcom C 11.0c (original Razorworks build) | `aphavoc/readme.txt` ("EECH was developed using Watcom C"); `aphavoc/makefile` (`wcc386`); `cmake.bat` (`wmake optimise=1 commercial=1`); `setup/cohokum/watcom_compile.txt` | x87 code. Startup word `__8087cw` (RTZ, PC_53). `(int)` casts truncate through the runtime's chop routine, whatever the rounding control. Whether a value is re-rounded to `float` at each assignment depends on register allocation: Watcom has no documented `/fp:precise`-style guarantee. | Proven that this toolchain was used; register behaviour needs a binary. |
| MSVC 2008 (`EECH-MSVC.vcproj`, `Version="9,00"`, links `d3d9.lib`) | This is the project that matches the **current** D3D9 source tree. Release settings: `/O2`, no `FloatingPointModel` (so the default `/fp:precise`), no `EnableEnhancedInstructionSet` (so the VS2008 default `/arch:IA32`, i.e. x87) | x87 arithmetic "at machine precision" (the control word's precision), rounded to source precision "at assignments, typecasts, when floating-point arguments get passed to a function call, and when a function call returns" ([/fp](https://learn.microsoft.com/en-us/cpp/build/reference/fp-specify-floating-point-behavior), [/arch](https://learn.microsoft.com/en-us/cpp/build/reference/arch-x86)). `(int)` truncates (not `/QIfist`). `__8087cw` is an ordinary unused global under MSVC. | Settings proven from the project file. Whether released binaries used it needs a binary. |
| GCC (autotools, `unix_startup.c`) | `Makefile.am`, `fpu.c`'s `__GNUC__` branch | x87 or SSE depending on target. On SSE targets `_control87` (`fldcw`) does not affect SSE arithmetic at all. **Defect:** the `__GNUC__` `asm_convert_float_to_int` (`fpu.h:88`) writes `"fistp (%1)"`, which GNU as assembles as the **16-bit** `fistps` (opcode `DF /3`, with an assembler warning). The probe shows it writes 16 bits of a 32-bit `int`: `70000.7` gives `0x7ead8000`, with the high half left unchanged. | Proven by probe. It does not affect the Windows builds. |

### 2.2 Which build shipped

This **needs a binary**. The current source tree is D3D9, and only the MSVC project links `d3d9.lib`. The Watcom documentation describes DirectX 7. That points to MSVC 2008 x87 `/fp:precise` for community releases built from this tree (inferred). Both toolchains agree on the facts that matter most here:

- the arithmetic is **x87**;
- the campaign thread rounds **toward zero** and runs at **PC_53**;
- `(int)` casts **truncate**;
- `convert_float_to_int` is `fistp`, which under RTZ also **truncates**.

### 2.3 Is the GCC 32-bit oracle equivalent?

No, and the gap is measured in §5.

- The canonical oracle is `-m32 -msse2 -mfpmath=sse` with the default MXCSR, so every `float` operation is evaluated at declared type (`FLT_EVAL_METHOD 0`) and rounded to nearest.
- That equals a historical environment only in its integer, list and pointer behaviour, and in float operations whose result is exact.
- `-mfpmath=387 -fexcess-precision=standard` with an explicit control word reproduces the MSVC `/fp:precise` rounding points. These are the investigation variants.
- An `-O2` variant gave the same results as `-O0` on every corpus. So under `-fexcess-precision=standard` the results are free of optimisation artefacts; Watcom's behaviour is still unknown (§2.1).

## 3. Sensitivity inventory of Slices 1–3 (Investigation 3)

Every floating-point operation in the frozen slices, from the original C. The TS column is the current port.

**Classes:**

- **exact:** the result is representable, so no environment changes it.
- **conversion:** a float-to-int conversion.
- **stored bits:** the rounding changes stored or emitted float bits.
- **branch:** it can change a comparison.
- **accumulation:** the error compounds across frames.

| # | Original expression (file) | TS | Class | Environment dependence (evidence §5) |
|---|---|---|---|---|
| 1 | `required = 100.0 - (level * AMMO/FUEL_USAGE_ACCELERATOR)` (`group.c:696/738`; accelerator 1.0) | `toFloat32(100 - level)` | stored bits | The `double` difference is exact; its narrowing to `float` depends on the rounding direction. |
| 2 | `required = bound (required, 0.0, level)` | `bound` | branch | Comparisons are exact, but their inputs come from #1. |
| 3 | `level -= required` (keysite) | `toFloat32(level - required)` | stored bits | Rounding direction. |
| 4 | `set_..._float_value (en, ..., level + required)` (argument rounding) | `toFloat32` in `en_values` | stored bits, then **branch** on the next assess (`< 100.0`) | RTZ gives `99.99999` where RN gives `100.0`, so the group is refuelled again. Probe `supply 0.00001 100`; generator corpus 168/1500. |
| 5 | `get_approx_2d_range`: `((d*4.0) + e) * (1.0/4.0)` evaluated in `double` and narrowed (`range.c`) | `toFloat32((d*4 + e) * 0.25)` | stored bits, **branch** (`range <= min_range`, `range < best_range`) | Rounding direction of the `double` add and of the narrowing. |
| 6 | `get_2d_range`: `dx = ...; range = sqrt ((dx*dx) + (dz*dz))` | float ops, then `Math.sqrt`, then `toFloat32` | stored bits, branch (reported `actual_range`) | Rounding direction, **and precision**: x87 does not round `dx*dx + dz*dz` to `float` before `sqrt`. The only precision-sensitive site in the corpora (2/1500 under RN; 15/1500 between `x87-rtz-pc53` and RTZ at declared type, measured directly after the migration). |
| 7 | `raw->sleep -= get_delta_time ()`; `max (sleep, 0.0f)`; `sleep == 0.0` (`gp_updt.c:92-104`), same for `assist_timer` | `toFloat32(t - dt)` | stored bits, **accumulation**, **branch** (removal from the update list) | Subtraction rounding direction. The sign is never affected (an x−y that is zero is exact), but the expiry *frame* shifts: probe `timer 1.0 0.01` expires in 101 frames under RN and 100 under RTZ. Timeline corpus 430/1000. |
| 8 | `iterations = (int) (get_delta_time () * frame_rate + 1.0)` (`up_update.c:219`) | `toCInt(toFloat32(dt * rate) + 1.0)` | conversion, **branch** (the loop count) | **Precision**: SSE rounds `dt*rate` to `float` first, x87 does not (see §6 assembly). `0.7 × 10` gives 8 iterations on SSE and 7 on x87 (either rounding). The `(int)` truncates in every build. At the default rate of 2 (`cmndline.c:110`) the product is exact, so this is environment-independent there. |
| 9 | `entity_update_delta_time = get_delta_time () / iterations` | `toFloat32(dt / it)` | stored bits, accumulation (it is the delta every timer uses) | Rounding direction when `iterations` is not a power of two. |
| 10 | `set_manual_delta_time`, `system_delta_time` stores | `toFloat32` | exact (they store a value that is already a float) | none |
| 11 | `max_map_x = (float) (n * side) - 1.0` (`en_world.c:113`) | `toFloat32(toFloat32(n*side) - 1)` | exact for every map whose extent is at most 2^24 m | Needs `side ≥ 2^22` m to differ (adversarial map `2 2 16777216`). Unreachable with real map data. |
| 12 | `mid = min + ((max - min) * 0.5)` | as C | exact whenever #11 is | as #11 |
| 13 | `get_x_sector`: `convert_float_to_int (x, &s); s /= SECTOR_SIDE_LENGTH` (`en_world.h:141`) | `cIntDivide(toCInt(x), side)` | conversion, **branch** (sector membership) | `fistp` follows the control word: RTZ truncates, which is what the port does; RN rounds half to even. `511.5` falls in sector 0 under RTZ and sector 1 under RN. Differs only in the RN-with-fistp variants (274/1000). |
| 14 | Attribute narrowing: `va_arg (double)` to a `float` field, and `vec3d` components (`en_attrs.c`) | `toFloat32` | exact when the caller passes a promoted `float` (all campaign callers) | Rounding direction only for callers passing a non-float `double` |
| 15 | Bit-fields fed from ints (`alive:1`, `side:2`, sector `x/z:8`) | `storeUnsignedBitfield` | exact (integer) | none |
| 16 | Keysite and group vec3d position copies (`mb_vec3d.c`, `ks_vec3d.c`) | `toFloat32` | exact | none |
| 17 | `sleep > 0.0`, `level < 100.0`, and similar comparisons | direct | branch, exact comparisons | Depend only on their inputs |

## 4. Oracle variants (Investigation 4)

Built by `buildHarnessVariant` into `build/c-reference-fpu/<name>/`:

| Variant | Code generation | x87 control word | MXCSR rounding | `convert_float_to_int` | Models |
|---|---|---|---|---|---|
| `sse-rn` | SSE (canonical flags) | 0x037F | nearest | `(int)` | control: must equal canonical |
| `sse-rtz` | SSE | 0x0F7F | toward zero | fistp | rounding direction alone |
| `x87-rn-pc64` | x87, `-fexcess-precision=standard` | 0x037F | – | `(int)` | Linux default word |
| `x87-rn-pc53` | x87 | 0x027F | – | `(int)` | MSVC default, without EECH's RTZ |
| `x87-rn-pc53-fistp` | x87 | 0x027F | – | fistp | as above, with EECH's conversion |
| `x87-rtz-pc53` | x87 | 0x0E7F | – | fistp | **EECH application thread per source** |
| `x87-rtz-pc53-O2` | x87 `-O2` | 0x0E7F | – | fistp | compiler-artefact check |
| `x87-rn-pc24` | x87 | 0x007F | – | fistp | D3D9 default, if it reached the campaign thread |
| `x87-rtz-pc24` | x87 | 0x0C7F | – | fistp | D3D9 precision plus EECH's RTZ re-assertion |

Instruction model:

- `objdump` of `gp_updt.c.o` shows `movss`/`subss`/`comiss` in the canonical build and `flds`/`fsubrp`/`fcomip` in the x87 variants.
- Each variant reports its environment with the harness `fpu` command, for example `fpu cw 0e7f mxcsr-rc 0 flt-eval-method 2`.
- It then checks the environment before every scenario line and at exit, and aborts with status 4 on drift. No drift occurred.

Scenario input is parsed and narrowed under round to nearest in every variant (`next_double` / `next_float`). So the starting state is identical and only original code runs under the variant environment. For the canonical build, `next_float` performs the same `(float) strtod` narrowing the implicit assignments did, so its output is bit-identical (the recorded fixtures still pass).

## 5. Differential results (Investigation 5)

The inputs are the generators at the seeds and counts used by `differential.cref.test.ts`. TS equals the canonical oracle on these corpora, so each count below is also a TS-versus-variant count. The counts are scenarios whose complete output differs: branches, messages, entity graph, float bits and exit status. Brackets name the differing output line kinds.

| Corpus | sse-rtz | x87-rn-pc64 | x87-rn-pc53 | x87-rn-pc53-fistp | **x87-rtz-pc53** | x87-rtz-pc53-O2 | x87-rn-pc24 | x87-rtz-pc24 |
|---|---|---|---|---|---|---|---|---|
| Slice 1 scenarios (1500) | 188 | 2 | 2 | 2 | **181** [transmit 168, final 168, closest 13] | 181 | 0 | 188 |
| Slice 2 timelines (1000) | 430 | 6 | 6 | 6 | **430** [timer 430, step 1] | 430 | 0 | 430 |
| Slice 3 lifecycles (1000) | 0 | 0 | 0 | 274 [cargo, sector] | **0** | 0 | 274 | 0 |
| Adversarial Slice 1 (167) | 117 | 10 | 10 | 10 | **108** [transmit 26, range 66, closest 16] | 108 | 0 | 117 |
| Adversarial Slice 2 (70) | 69 | 60 | 60 | 60 | **69** [timer, step] | 69 | 0 | 69 |
| Adversarial Slice 3 (79) | 5 | 0 | 0 | 30 | **5** [cargo 5, sector 1] | 5 | 30 | 5 |

The `sse-rn` control variant differed in 0 scenarios in every corpus.

### 5.1 Difference classes, each reduced to its arithmetic path

1. **Rounding direction of stored results** (every RTZ variant). This is inventory #1, #3–#7 and #9.
   - Reduced example (Slice 1, a group with fuel `34.227` assessed against a keysite): `required = 100.0 - 34.227f` narrows to `65.773` rounded down, and `34.227f + required` comes to `0x42c7ffff` (99.99999) instead of `0x42c80000` (100.0). The transmit and final lines differ, and the next assess refuels again.
   - Timelines: the `sleep -= delta` bits drift by 1–6 ULP, and the removal frame can shift (probe `timer 1.0 0.01`: 101 frames against 100).
   - This class **dominates**: 12% of Slice 1 and 43% of Slice 2.
2. **Intermediate precision** (x87 against declared type, either rounding).
   - `get_2d_range`: the `sqrt` argument keeps 53 bits. Reduced example: keysite `(13255.324, 17468.32)` against position `(1415.038, -17891.939)` gives `46d8d608` against `46d8d609`. This is the only precision class in the generator corpora (2/1500).
   - `set_entity_update_frame_rate`: `dt*rate` is not rounded before `+ 1.0`. Reduced example: rate 100, delta `0.04` gives 5 iterations on SSE and 4 on x87, so the timers move with sub-delta `0x3c03126e` against `0x3c23d70a`. These are the 6/1000 timeline differences and the 60 adversarial ones. None occurs at the default rate of 2.
   - Pairwise runs over all 3,500 generator scenarios confirm that 24-bit x87 at each operation equals `float` at declared type. `x87-rtz-pc24` produces output identical to `sse-rtz`. `x87-rn-pc24` equals canonical in Slices 1–2; its Slice 3 differences are the fistp class below. It differs from `x87-rn-pc53-fistp` in exactly 8 scenarios, the 2 + 6 above.
3. **fistp under round-to-nearest** (`*-fistp` and `x87-rn-pc24`). Inventory #13: sector membership changes for positions with fractional part ≥ .5 (Slice 3, 274/1000). Every RTZ variant truncates, which matches the port.
4. **Map extent beyond 2^24 m** (RTZ, adversarial only). Inventory #11/#12: unreachable with real maps.
5. **Compiler artefacts:** none. `-O2` output is identical to `-O0` output on all 3,500 generator scenarios, and gives the same counts on the adversarial corpora. The canonical build's `(int)` and the x87 build's cast (`fnstcw`/`or $0x0c00`/`fldcw`/`fistpl`, a truncating chop) agree.

## 6. Targeted probes (Investigation 6)

`c-reference/fpu-probes/probes.c` isolates the original expressions verbatim. `run-probes.mjs` builds them for SSE, x87 and x87 `-O2`, runs them under each control word, and writes the assembly to `build/fpu-probes/`. The results are float bits, plus the frame count, iteration count or branch where relevant.

| probe | sse RN | sse RTZ | x87 RN PC53 | **x87 RTZ PC53** | x87 RN PC24 | x87 RTZ PC24 |
|---|---|---|---|---|---|---|
| timer 1.0 − 0.01 until 0 | 101 frames | 100 | 101 | **100** | 101 | 100 |
| subdivide 0.7 × 10 | 8, `3db33333` | 7, `3dcccccc` | 7, `3dcccccd` | **7, `3dcccccc`** | 8, `3db33333` | 7, `3dcccccc` |
| subdivide 0.1 × 10, 0.617 × 2 | 2, 2 | same | same | same | same | same |
| supply 0.00001 against 100 (required, keysite, group, `< 100`) | `42c7ffff 37000000 42c80000 0` | `42c7fffe 37800000 42c7ffff 1` | as sse RN | **as sse RTZ** | as sse RN | as sse RTZ |
| range (2d, approx) −7027.003,362.58 : 1414.948,−6811.288 | `462d1998 461fedac` | `462d1997 461fedab` | `462d1999 461fedac` | **`462d1997 461fedab`** | `462d1998 461fedac` | `462d1997 461fedab` |
| range 12345.678,−9876.543 : 1.1,2.2 | `46770acb 4667790e` | `46770ac8 4667790d` | `46770acb 4667790e` | **`46770ac9 4667790d`** | `46770acb 4667790e` | `46770ac8 4667790d` |
| map 64 × 4096 (max, mid) | `487fffc0 47ffffc0` | same | same | same | same | same |
| sector 511.5 / 512 (fistp) | 1 | 0 | 1 | **0** | 1 | 0 |

(The full table, with PC64 and `-O2` columns and more inputs, is printed by `npm run spike:fpu`.)

Assembly for `set_entity_update_frame_rate` (`probes-*.s`) shows the precision class directly:

```
SSE: cvtsi2ssl rate; mulss dt      -> product rounded to float
     cvtss2sd; addsd 1.0; cvttsd2si
x87: flds dt; fildl rate; fstps/flds (rate as float, exact); fmulp  -> product kept at PC precision
     fld1; faddp; fnstcw; or $0x0c00; fldcw; fistpl; fldcw         -> (int) chops, whatever the RC
```

What each probe isolates:

- **Rounding direction:** supply, timer, range (sse RN against sse RTZ).
- **Precision:** subdivide and `get_2d_range` (sse against x87 at the same rounding).
- **Float-to-int:** sector (fistp follows RC; a cast chops).
- **Float stores:** `-fexcess-precision=standard` rounds at assignments, casts and arguments, as MSVC `/fp:precise` does.
- **Modern-compiler artefacts:** `-O0` against `-O2`, identical.

## 7. Decision gate

**Outcome: C (mixed rule), with the precision component at D (insufficient evidence).** Accepted; implemented in §9.

- **Rounding direction: established, toward zero.**
  - EECH sets RTZ on the campaign thread at start-up and after every library initialisation (§1.1). This is proven for both toolchains.
  - It is observable in all the numerical code of Slices 1 and 2, in stored bits and in branches: supply refuel repetition, timer expiry frame, closest-keysite threshold (§5.1-1).
  - The current model, round to nearest, matches no plausible historical environment of the campaign thread.
  - Handling at integer-conversion boundaries alone (outcome A) is **rejected**. The conversion boundary (`fistp` for the sector index) is the one place where the current port is *already* right, because RTZ fistp truncates. The divergence is in ordinary float arithmetic.
- **Precision: not established.**
  - The source supports PC_53 on the campaign thread (§1.4).
  - The two candidate environments, `x87-rtz-pc53` and `x87-rtz-pc24` (the latter equal to "RTZ at declared type"), differ only in two expressions: the `sqrt` argument of `get_2d_range` (15/1500 scenarios, all `closest` lines; measured directly after the migration, §9.4), and `dt * rate` at non-default entity update rates (0 in the default configuration).
  - Deciding between them needs a runtime trace of the shipped binary. For example, log `get_fpu_control_word_value ()` (`fpu.c:188`, already present) inside `update_client_server_entities` on Windows with Direct3D active. Section 2.2 on the toolchain has the same dependency.
- **Float-to-int:** keep truncation (`toCInt`). It is correct under RTZ.

The proposal deliberately does **not**:

- change the canonical oracle or the TS arithmetic in this spike. The issue requires the rule to be agreed on this evidence first;
- regenerate any fixture;
- introduce a global FPU service.

## 8. Consequences, if the rule is accepted

### Oracle

- **Canonical configuration.** Keep SSE at declared type and set MXCSR rounding to toward zero. That is the `sse-rtz` variant, which equals `x87-rtz-pc24`, a historical hypothesis. Set x87 RC to chop as well, so libm and `fistp` agree.
- **Precision.** Adopting `x87-rtz-pc53` instead is the alternative if the runtime trace shows PC_53. It changes a further 15/1500 range results and the non-default-rate subdivisions.
- **Flags.** The GCC flags stay the same; the harness installs the rounding control word at start-up (the variant mechanism, promoted).
- **Fixtures.** `c-reference-random.cases.ts` (Slice 1), `c-reference-random-timelines.cases.ts` (Slice 2) and the hand-written matrices with inexact supply and timer values would need re-recording. Each change is an **oracle-fidelity correction**, to be reviewed explicitly, with its reduced class from §5.1 attached. Slice 3 lifecycle fixtures are unaffected (0/1000 under RTZ; quarter-metre positions and power-of-two sectors are exact).
- **CI guard.** `test/c-reference/fpu-environment.cref.test.ts` (added now) pins the canonical environment (`fpu cw 037f mxcsr-rc 0 flt-eval-method 0`). A change must update it deliberately. The variant builds already abort on control-word drift.

### TS/TSTL

- **Which helpers change.** `toFloat32` becomes `toFloat32` toward zero at every site in §3 marked "stored bits". That covers `group.ts`, `update.ts`, `range.ts`, `en_values.ts`, `en_attrs.ts`, `en_world.ts`, `mobile.ts`, `cargo.ts` and `time.ts`: the same sites, not new ones.
- **Why a narrowing helper alone is not enough.** A JS/Lua `double` operation has already rounded to nearest, so where the double result is inexact, direction matters.
- **What a small operation-level helper set covers:**
  - float `+ − ×`, where the double result of two floats is exact (sums within 2^29 exponent spread, all products), so narrowing toward zero is exact;
  - the `double` expression in `get_approx_2d_range`, and `/` and `sqrt`, which need the error sign: TwoSum, and a Dekker product residual (Lua 5.1 has no FMA);
  - RTZ composes, so a double rounded toward zero and then narrowed toward zero equals a direct narrowing toward zero. No double-rounding hazard arises.
- **Verification.** Each helper would be verified bit for bit against the `sse-rtz` C probes in JS, in Lua 5.1, and through the C reference, like `toFloat32` today.
- **Mutation controls.** RTZ becoming RN must be caught; the adversarial corpora provide the killing inputs.

## Appendix: running the spike

```
npm run spike:fpu      # builds the canonical oracle and all variants, runs
                       # the generator and adversarial differentials
                       # (build/fpu-spike/*.json) and the probes table
node c-reference/fpu-probes/run-probes.mjs --asm   # plus assembly excerpts
```

## 9. RTZ migration (implemented)

### 9.1 EECH numerical contract

```
Rounding direction
    RTZ                           established, canonical
Declared float operations
    IEEE binary32 + RTZ           canonical
float -> int
    truncation / EECH semantics   established (toCInt; fistp under RTZ)
Compile-time constants
    round to nearest              established (C constant folding; time.c's 0.1)
x87 intermediate precision
    24 / 53 / extended            unresolved: requires the runtime trace (§9.3)
```

### 9.2 What changed

- **Canonical oracle** (`c-reference/harness.c`):
  - installs MXCSR rounding toward zero, and x87 control word `0x0f7f` (chop) so libm's x87 `sqrt` and `fistp` agree, before any scenario line;
  - checks the environment before every line and at exit, and aborts with status 4 on drift;
  - still evaluates at declared float type (SSE, `FLT_EVAL_METHOD 0`);
  - parses and narrows scenario input to nearest (`next_double` / `next_float`), because input is the scenario's value, not EECH arithmetic.

  The `fpu` command reports `fpu cw 0f7f mxcsr-rc 3 flt-eval-method 0`, and `test/c-reference/fpu-environment.cref.test.ts` pins that.
- **Helpers** (`src/core/float32.ts`), one per operation shape the frozen code performs:
  - `toFloat32RTZ`: narrowing;
  - `f32Add` / `f32Sub`: a float or double sum stored as float, using an exact TwoSum error term;
  - `f32Mul`: the double product of floats is exact;
  - `f32Div` / `f32Sqrt`: the double result of float operands is never within half an ulp of a float unless exact, since 53 ≥ 2·24+2.

  There is no FPU abstraction: each site calls the helper for the C expression it ports. `toFloat32` (round to nearest) remains for C constant initialisers and scenario input.
- **Verification** (`test/c-reference/float32-rtz.cref.test.ts`):
  - 40,000 fresh generated operations and the special values (signed zeros, infinities, overflow to `FLT_MAX`, subnormals) are compared bit for bit against the harness `f32` command.
  - 3,000 C-recorded operations (`test/scenarios/generated/c-reference-float32-rtz.cases.ts`) are replayed in JavaScript (`test/unit/float32-rtz.test.ts`) and in Lua 5.1 (`test/lua/conformance.ts`).
  - Mutation controls cover: narrowing to nearest (JS and Lua); the missing step below (JS and Lua); a wrong error sign; the power-of-two spacing; `mul`/`div`/`sqrt` to nearest; the constant initialiser toward zero; unnarrowed timer arithmetic; and an oracle that still rounds to nearest.
- **Port call sites** (inventory §3):

  | Code | Inventory items |
  |---|---|
  | `group.ts` supply | #1–#4 |
  | `group.ts` timers | #7 |
  | `update.ts` | #8, #9 |
  | `range.ts` | #5, #6 |
  | `en_world.ts` | #11, #12 |
  | `en_values.ts`, `en_attrs.ts`, `mobile.ts`, `cargo.ts` | #14, #16 |
  | `time.ts` | set and measured deltas |

- **Test runners** narrow scenario input to nearest, as the harness does (a timeline's `set` value and frame delta, and the group leader's position).

### 9.3 Fixture changes, each explained

`test/fpu-spike/fixture-deltas.fpu.test.ts` (run by `npm run spike:fpu`) checks the recorded fixtures against the last round-to-nearest commit. For every case whose expectation changed, it requires three things:
- the former oracle (`sse-rn`) reproduces the old expectation exactly;
- the canonical oracle reproduces the new one;
- every changed field belongs to a documented class.

| Fixture | Cases | Changed | Class (fields) |
|---|---|---|---|
| Slice 1 `c-reference-random.cases.ts` | 250 | 21 | **supply**: transmitted values (31), group and keysite ammo/fuel (31) |
| | | 2 | **range**: `closestRange` (2) |
| Slice 2 `c-reference-random-timelines.cases.ts` | 150 | 75 | **timer**: sleep (103) and assist (163) timer bits |
| Slice 3 `c-reference-random-lifecycles.cases.ts` | 150 | 0 | byte-identical (asserted) |

No change fell outside the documented classes. No update-list membership, message or result changed in the recorded fixtures. The fresh-corpus differentials of `differential.cref.test.ts` (1,500 / 1,000 / 1,000 scenarios) pass against the RTZ oracle.

The two hand-written matrix cases whose C outcome changed were updated with the C-confirmed values. Each now documents its class.
- `keysite-float-arithmetic` (supply): `33.3f + (100.0 − 33.3f)` truncates to `99.99999f`, so the group stays below 100 after the refill.
- `frame-is-subdivided-into-equal-float-sub-steps` (timer): `1.0f / 3` truncates to `0.3333333f`, so three passes leave `2^-24`, and the group stays on the update list for another frame.

Three unit tests asserted round-to-nearest narrowing literals (`0.1`, `1.1`). They now assert the toward-zero bits `0x3dcccccc` and `0x3f8ccccc`.

### 9.4 Unresolved: intermediate evaluation precision

The open question is not only "is the precision control 24 or 53 bits". It is **at what precision the original executable evaluated intermediates, and at which points it rounded them back to declared type**. The control word decides the first part at run time. The compiler's generated instructions decide the second: which values stay in x87 registers, and which are stored to `float` memory. That is MSVC `/fp:precise` rounding points, or Watcom register allocation.

The runtime trace settles only the first part. It needs `get_fpu_control_word_value ()` (`fpu.c:188`) logged inside `update_client_server_entities`, on Windows with Direct3D active. The second part needs the shipped binary's instructions for the affected functions.

Until both are established, the contract evaluates at declared type. The known sites where this matters are:
- **`get_2d_range` (canary).** It is the only operation whose results depend on intermediate precision in the generator corpora: x87 does not round `dx*dx + dz*dz` to float before `sqrt`. Measured directly against the migrated canonical oracle, `x87-rtz-pc53` differs in 15/1500 Slice 1 scenarios, all in `closest` lines, and in none of the Slice 2 or 3 generator scenarios. Under round-to-nearest the figure was 2/1500 (§5). An earlier estimate of 7, obtained by subtracting two counts, is superseded. `c-reference/fpu-probes` keeps the probe (`range -7027.003 362.58 1414.948 -6811.288`, `range 12345.678 -9876.543 1.1 2.2`). If the evidence establishes extended intermediates, this operation changes deliberately and no other changes along with it.
- **`set_entity_update_frame_rate`.** `dt * rate` is not rounded before `+ 1.0`. This is only observable at non-default entity update rates; the default rate is 2.
- **`keysite.c :: update_keysite_cargo`'s crate-row step (Slice 4).** The statement `position.x += (bounding_box->xmax - bounding_box->xmin) + 1.0` is the port's `advanceCrateRow`.
  - It is checked whole against the executed C (`test/c-reference/keysite-crate-row.cref.test.ts`) and probed under the x87 variants (`c-reference/fpu-probes`, `crate-row`).
  - Its `+ 1.0` is a double operation, so precision control matters: at `x 0.1, xmin -0.025, xmax 0.025` it stores `3f933332` under x87 RTZ PC24 and `3f933333` under declared type and under x87 RTZ PC53.
  - It is the first ported expression that separates PC24 from PC53. In the Slice 1–3 corpora, `x87-rtz-pc24` was output-identical to `sse-rtz`.

**Slice 4 must not start until this migration is frozen.**
