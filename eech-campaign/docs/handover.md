# Handover

State of the `claude/eech-campaign-native-spike-tjkj6g` branch at hand-over. For
the design, the patch list and the findings, see [`engine.md`](engine.md). For
the recordings, see [`../recordings/README.md`](../recordings/README.md).

## What exists

- **The full EECH engine, headless, as a Lua module.** The 1,217 C files of the
  maintained Windows build compile for x86-64 Linux
  (`crates/eech-engine-sys`) behind a safe Rust API (`crates/eech-engine`) and
  an mlua `cdylib` (`crates/eech-dc`, `require ("eech_dc")`).
- **The host.** `crates/eech-world` embeds Lua 5.1, runs
  `lua/campaign.lua`, and records Tacview ACMI 2.2. Positions can go through a
  fitted map projection (`affine`).
- **Maps:**
  - `eech-map` generates maps and campaigns from OSM and SRTM: `luxembourg`
    (map15) and `georgia` (map16);
  - `tools/retail-map3.sh` assembles the **retail Georgia campaign** (map3,
    "Caspian Black Gold") from retail data. This is the current focus.
- **Storage:** `tools/offsite.sh` uses rclone against one Google Drive folder
  for install data and run outputs. It is written but untested: no credential
  has been configured yet.

## The retail Georgia campaign

Needs data from a retail install, kept **outside the repository**:
`cohokum/3ddata`, and `common/maps/map3` with `camp01`, `route` (including
`ROADDATA.*`) and `terrain` (`terrain.ffp`, `default.sec`, `default.rgb`).
The user uploaded these to the previous session's container only, which is
ephemeral. They have also put full installs in Google Drive:

- `MyDrive/EECH` (folder id `10wI06di8fwtefZbp_xgiv5cPhB4lwtPy`), which holds:
  - `Enemy Engaged Apache vs Havoc`: the base game, with `common` and `aphavoc`;
  - `Enemy Engaged 2 Desert Operations`: a different engine, not usable.

```sh
cargo build --release -p eech-dc -p eech-world
tools/retail-map3.sh <data> /tmp/geo_retail
target/release/eech-world crates/eech-world/lua/campaign.lua root=/tmp/geo_retail \
    scenario=georgia_retail hours=2 record_every=10 diagnostics_every=3600 acmi=georgia-retail-2h.acmi
```

Status:

- **Boots and runs.**
  - Blue starts with Batumi and 16 FARPs: 16 jets, 66 helicopters and 212 vehicles.
  - Red holds ten airbases (Beslan, Gudauta, Sukhumi, Kutaisi West,
    Mikha-Tskhakaya, Samtredia, Makharadze, Vaziani, Soganlug, Marneuli), 40
    FARPs and two carriers: 32 jets, 234 helicopters, 465 vehicles and 6 ships.
- **Keysites change hands:** FARPs 18, 7, 3 and 17 changed hands within 2.2
  simulated hours.
- The last 12-hour run, started 09:46 UTC on 2026-09-24, wrote
  `/tmp/claude-0/georgia-retail-12h.acmi` in that container. It was never
  collected, because the session's shell was blocked, and it is gone with the
  container.
- **Speed:** a 12-hour run takes roughly 45–60 minutes of wall time.

Fixes this path needed (see `engine.md`):

- **C1:** a collision object index of 0 means none.
- **T1, T2:** texture mismatches are logged instead of fatal (headless).
- **The community objects overlay:** the Steam 3D database has 2,761 of the
  3,026 scenes this source defines, and `setup/cohokum/3ddata/objects`
  supplies the rest.
- **Script trigger:** the retail `GEORGIA.SCR` 30-minute fail trigger is
  dropped in the installed copy.

The map3 projection (x stretched 1.217 east–west) comes from correlating the
retail terrain with SRTM (r = 0.978). The heightmap used was
`map3/graphics/height.tga`: grey = 255 − palette index, about 18.8 m per level.

## Next steps (what the user asked for)

1. **Record a 2-hour retail Georgia session and share it through Google
   Drive.** This needs an rclone credential in the environment:
   - `RCLONE_CONFIG_EECHDRIVE_TOKEN`: the JSON from `rclone authorize "drive"`,
     run on the user's machine;
   - `RCLONE_CONFIG_EECHDRIVE_ROOT_FOLDER_ID` = `10wI06di8fwtefZbp_xgiv5cPhB4lwtPy`.

   Then run `tools/offsite.sh check`, pull the retail data, run, and
   `tools/offsite.sh push-output georgia-retail-2h <file>`. The Drive
   connector (MCP) cannot move large files: content goes inline as base64.
   Attachments in the chat are limited to 30 MB.
2. **"Delete Luxembourg"**: the user asked for this, but never confirmed the
   scope. Either remove only `recordings/luxembourg-6h.zip.acmi` and its README
   section, or remove the whole scenario (the `eech-map` spec, the
   `campaign.lua` entry, and the README and `engine.md` sections). Ask.
3. **Pack the retail install root** to Drive (`tools/offsite.sh push-install
   georgia-retail <root>`), so future sessions can pull it.

## Known limits

- **Recordings from before X1** (the 16-bit float-to-int fix) were wrong past
  32.7 km. All recordings now in `recordings/` postdate it.
- **Generated maps** (Luxembourg, Georgia map16) run wars of attrition, and no
  keysite changes hands. The retail campaign does capture.
- **Supply:** EECH airbases drain supply and are resupplied only from
  producers (S1 fixed a self-supplier bug). The retail map3 population file is
  version 1, so it has no producer keysites.
- **Linux only.** There is no player: the engine runs as a dedicated server.
