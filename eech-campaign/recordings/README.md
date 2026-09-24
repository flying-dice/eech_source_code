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
