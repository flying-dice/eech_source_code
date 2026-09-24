# Recordings

Dynamic campaigns recorded by the harness. `eech-world` runs
`lua/campaign.lua`, which drives the `eech_dc` DLL (all of EECH, headless).
Tacview opens the `.zip.acmi` files directly. Each is the ACMI 2.2 text file,
zipped, sampled once per simulated second.

Both were recorded with patches X1 (float-to-int conversion) and S1 (airbase
resupply) and with supply producers on the map (`docs/engine.md`). An earlier
Luxembourg recording from before X1 had wrong sector lookups for everything
past 32.7 km. It has been replaced.

```sh
cargo build --release -p eech-map -p eech-dc -p eech-world
target/release/eech-map georgia-latest.osm.pbf srtm-georgia/ /tmp/georgia      # SRTM N41-N43 x E040-E046
target/release/eech-world crates/eech-world/lua/campaign.lua root=/tmp/georgia scenario=georgia \
    hours=12 seed=1 record_every=10 diagnostics_every=7200 acmi=georgia-12h.acmi
target/release/eech-map luxembourg-latest.osm.pbf srtm-luxembourg/ /tmp/lux    # SRTM N49-N50 x E005-E006
target/release/eech-world crates/eech-world/lua/campaign.lua root=/tmp/lux scenario=luxembourg \
    hours=6 seed=1 record_every=10 diagnostics_every=3600 acmi=luxembourg-6h.acmi
```

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

## `luxembourg-6h.zip.acmi`

Six simulated hours of Luxembourg (front at 6.07° E).

| | Blue (US) | Red (Russia) |
|---|---|---|
| Airbases | Useldange, Wiltz-Noertrange | Luxembourg Findel, Echternach (synthesised) |
| Destroyed | 1,314: 1,099 vehicles, 156 helicopters, 41 fixed wing | 1,180: 988 vehicles, 142 helicopters, 23 fixed wing |
| Weapons launched (recorded) | 6,830 | 6,596 |

A closely matched war along the whole front. BAI, CAS, CAP, recon, SEAD,
OCA strike and sweep, ground strike, escort, BDA, repair, supply and troop
insertion are all flown. No keysite changes hands in six hours.

Both runs are deterministic: the same extract, SRTM tiles and seed reproduce
them line for line.
