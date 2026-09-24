# Recordings

Dynamic campaigns recorded by the harness. `eech-world` runs
`lua/campaign.lua`, which drives the `eech_dc` DLL (all of EECH, headless).
Tacview opens the `.zip.acmi` files directly. Each is the ACMI 2.2 text file,
zipped, sampled once per simulated second.

They were recorded with patches X1 (float-to-int conversion) and S1 (airbase
resupply) (`docs/engine.md`). The generated map also has supply producers.

```sh
cargo build --release -p eech-map -p eech-dc -p eech-world
target/release/eech-map georgia-latest.osm.pbf srtm-georgia/ /tmp/georgia      # SRTM N41-N43 x E040-E046
target/release/eech-world crates/eech-world/lua/campaign.lua root=/tmp/georgia scenario=georgia \
    hours=12 seed=1 record_every=10 diagnostics_every=7200 acmi=georgia-12h.acmi
```

The retail campaign needs retail data outside the repository (see
`tools/retail-map3.sh`), and `tools/acmi-summary.py` gives the numbers below:

```sh
tools/retail-map3.sh <data> /tmp/geo_retail
target/release/eech-world crates/eech-world/lua/campaign.lua root=/tmp/geo_retail scenario=georgia_retail \
    hours=2 record_every=10 diagnostics_every=3600 acmi=georgia-retail-2h.acmi
tools/acmi-summary.py georgia-retail-2h.acmi run.log
```

## `georgia-retail-2h.zip.acmi`

Two simulated hours of the retail Georgia campaign ("Caspian Black Gold",
map3), from 08:50. The data is the GOG Comanche vs Hokum `cohokum/3ddata`,
`camp01`, `POPNAME.DAT` and `BRIDGE.POP`, with the Steam Apache vs Havoc
map3 roads (`ROADDATA.*`) and terrain.

| | Blue (US) | Red (Russia) |
|---|---|---|
| Keysites, start → end | 16 → 15 | 54 → 55 |
| Destroyed | 133: 68 vehicles, 29 helicopters, 25 infantry, 9 air defence, 2 fixed wing | 135: 58 vehicles, 33 helicopters, 26 infantry, 15 air defence, 3 fixed wing |
| Weapons launched (recorded) | 553 | 357 |
| Ground vehicles, 10 min → end | 227 → 159 | 226 → 168 |
| Helicopters, 10 min → end | 70 → 41 | 242 → 211 |

Keysites change hands:

- FARP 18 goes to blue at 1:11, back to red at 1:19, and to blue again 31 s later.
- Red takes FARP 7 at 1:52 and FARP 3 at 1:58.

The losses are even, but they hurt blue far more. Blue loses 29 of its 70
helicopters, while red has more than three times as many to start with.

Tasks (peak concurrent, from the hourly diagnostics):

- **Blue:** recon 7, helicopter transfers 6, CAS 2, CAP 2, ground strike 2, escort, BAI, BDA and troop insertion.
- **Red:** helicopter transfers 9, supply 7, BAI 4, CAP 3, recon 3, fixed-wing transfers 3, repair 3, ground strike 3, troop insertion 3, escort 2, BDA 2, CAS and SEAD.

Red, with its ten airbases and two carriers, runs the supply, repair and
troop-insertion missions that the generated maps never see. The run took
about six minutes of wall time.

## Retail Georgia to a conclusion (not committed: 98 MB zipped)

The same campaign run until it was decided, with `stop=conclusion`: the run
ends when a side holds no airbase or FARP, or after 48 hours with nothing
captured or destroyed. Positions are sampled every 10 s (`record_every=100`),
so short-lived weapons are under-counted.

```sh
target/release/eech-world crates/eech-world/lua/campaign.lua root=/tmp/geo_retail scenario=georgia_retail \
    hours=336 stop=conclusion stalemate_hours=48 record_every=100 log_every=1800 diagnostics_every=21600 \
    acmi=georgia-retail-long.acmi
```

**Red wins after 61.8 simulated hours** (day 3, 22:36), when it takes blue's
last FARP. The run took 47 minutes of wall time.

| | Blue (US) | Red (Russia) |
|---|---|---|
| Keysites, start → end | 16 → 0 | 54 → 70 |
| Destroyed | 961: 610 infantry, 227 vehicles, 70 helicopters, 38 air defence, 16 fixed wing | 267: 93 vehicles, 91 infantry, 59 helicopters, 18 air defence, 5 fixed wing, 1 ship |
| Aircraft rebuilt from reserves | 0 | 33 (jets 27 → 36, helicopters 185 → 209) |

How the campaign unfolds:

- **The first 12 hours:** these repeat the 12-hour run below. Red's OCA strike puts Batumi out of action, and blue's air arm is destroyed.
- **Hours 12–18:** blue's last ground vehicles die. Only blue infantry is left.
- **Day 2:** almost nothing happens, with no captures between 12 h and 46.7 h. Red's airbases have drained to the 10% ammo and fuel floor, so there is little to task.
- **About 17 h and 47 h:** red's reserves arrive at the retail campaign's 16.7-hour regen interval (below).
- **46.7 h:** red takes Batumi.
- **48.6–61.8 h:** red captures blue's nine remaining FARPs, one every hour or two, with troop insertions.

Nothing in the run is a malfunction. Three settings in the retail data decide
it:

1. **Reinforcement is almost off.** `GEORGIA.CHC` sets `:REGEN_FREQUENCY 60000` for both sides. Each regen site tries once every 16.7 simulated hours; Thailand, Cuba, Taiwan, Lebanon and Yemen use 180–600 s. No aircraft is rebuilt in the first 12 hours. Regen also needs a usable keysite, so once Batumi is out of action blue can never rebuild at all.
2. **Blue has one airbase.** A REPAIR task never starts from the keysite it serves (`docs/engine.md`, finding 8), so Batumi is never repaired.
3. **There are no producers.** The map3 population file is version 1 and has no factories or refineries. Red's airbases drain to the 10% floor, which is why it fights so slowly after the first day.

The retail scripts define no victory condition. Comanche vs Hokum's map1–3
scripts (`THAILAND.SCR`, `CUBA.SCR`, `GEORGIA.SCR`) only end the campaign as a
FAIL after 30 minutes, and `GEORGIA.CHC` has no `CAMPAIGN_CRITERIA`. The
conclusion is the runner's own rule.

## Retail Georgia, 12 hours (not committed: 157 MB zipped)

The same setup run for 12 simulated hours, from 08:50 to 20:44. The first two
hours repeat `georgia-retail-2h` exactly. Then red takes over:

| | Blue (US) | Red (Russia) |
|---|---|---|
| Keysites, start → end | 16 → 10 | 54 → 60 |
| Destroyed | 485: 187 infantry, 175 vehicles, 69 helicopters, 38 air defence, 16 fixed wing | 257: 93 vehicles, 81 infantry, 59 helicopters, 18 air defence, 5 fixed wing, 1 ship |
| Weapons launched (recorded) | 1,548 | 1,406 |
| Helicopters, 10 min → end | 70 → 1 | 242 → 185 |
| Ground vehicles, 10 min → end | 227 → 52 | 226 → 133 |

- **FARP captures:** there are 14. Blue briefly holds FARPs 18, 17 and 19. Red then takes FARPs 17, 18, 11, 13, 8, 15 and 16, one every hour or two after the fifth hour.
- **Blue's air force:** its jets are gone by the fourth hour. 11 of them are lost around 2:55, parked at Batumi, to a red OCA strike with Kh-25MT missiles and S-8 rockets. Its air defence is gone by 6 h, and it is down to one helicopter by 10 h.
- **Red's losses:** red stops losing units after about 6 h. Its jets, helicopters and vehicles hold steady from then on, and troop insertions push its infantry from 208 to 355.
- **Tasks:** both sides fly OCA strikes and sweeps, SEAD and repair, besides the 2-hour mix.

## `georgia-12h.zip.acmi`

Twelve simulated hours of Georgia, from 08:50 into the night. The map spans
40.4–46.7° E by 41.0–43.6° N, with the front at 44.0° E through Shida Kartli.

| | Blue (US) | Red (Russia) |
|---|---|---|
| Airbases | Kutaisi, Senaki | Vaziani, Marneuli |
| Producers | factory near Ozurgeti, refinery near Supsa | factory at Rustavi, refinery in east Tbilisi |
| FARPs | 4, west of 44.0° E | 4, east of 44.0° E |
| Destroyed | 519: 224 vehicles, 167 infantry, 128 helicopters | 330: 143 vehicles, 62 infantry, 123 helicopters, 2 air defence |
| Weapons launched (recorded) | 1,281 | 2,675 |
| Ground vehicles, 10 min → end | 127 → 22 | 223 → 123 |

The war is fought over the Kartli plain by artillery (red's BM-21s fire 2,019
rockets) and by FARP-based attack helicopters: Hellfire, Vikhr, Ataka and
Shturm duels. Each side loses more than 120 helicopters, and the regen sites
rebuild them from reserves. The airbases are 70–160 km behind the front, so
the jets mostly fly recon, CAP, CAS, ground strike and SEAD, plus the
transfers and SUPPLY missions that carry factory and refinery output to the
airbases. By nightfall blue's frontline armour is nearly destroyed. No keysite
changes hands in these twelve hours.

A 48-hour continuation (not committed: 167 MB) also runs without a fault:
1,396 destroyed and 4,560 weapons launched. By day 3 both sides have used
all 160 of their reserve helicopters, and the front is static with 16 blue
and 68 red vehicles left. Still no keysite changes hands. Red never mounts a
troop insertion or an OCA strike across Georgia's distances.

The generated-map runs are deterministic: the same extract, SRTM tiles and
seed reproduce them line for line.
