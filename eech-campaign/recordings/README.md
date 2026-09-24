# Recordings

## `luxembourg-6h.zip.acmi`

Six simulated hours of the Luxembourg dynamic campaign, recorded by the
harness. `eech-world` runs `lua/campaign.lua`, which drives the `eech_dc` DLL
(all of EECH, headless). Tacview opens the file directly. It is the ACMI 2.2
text file, zipped.

```sh
cargo build --release -p eech-map -p eech-dc -p eech-world
target/release/eech-map luxembourg-latest.osm.pbf srtm/ /tmp/lux
target/release/eech-world crates/eech-world/lua/campaign.lua root=/tmp/lux hours=6 seed=1 \
    record_every=10 diagnostics_every=7200 acmi=luxembourg-6h.acmi
```

The run is deterministic. The same OSM extract, SRTM tiles and seed reproduce
it: the current build matches this file line for line, except for the order of
object-removal lines within a frame. That order was unsorted when this file
was written.

Sampled once per simulated second; 2,820 objects.

| | Blue (US) | Red (Russia) |
|---|---|---|
| Airbases at the start | Useldange, Wiltz-Noertrange | Luxembourg Findel, Echternach (synthesised) |
| Destroyed | 1,240: 1,031 vehicles, 107 helicopters, 43 fixed wing, 51 infantry, 8 air defence | 629: 472 vehicles, 91 helicopters, 16 fixed wing, 50 infantry |
| Weapons launched (recorded) | 4,261 | 7,261 |
| Keysites at the end | 2 (its FARPs) | 6 |

What happens:

- **0–1 h.** The frontline forces meet along 6.07° E, and the artillery (BM-21, M270 MLRS) and armour duels cost each side hundreds of vehicles. Both air forces fly the full range of tasks: BAI, CAS, CAP, recon, SEAD, OCA strike and sweep, ground strike, escort, BDA, supply, repair and transfers. The regen sites replace losses from the force's reserves.
- **1–2.5 h.** Blue's ground forces wear down faster, and red's OCA strikes knock Useldange out of action. REPAIR and SUPPLY missions fly to keep keysites working.
- **2.5 h.** Red helicopters insert troops, and **Airfield Useldange is captured by red**.
- **4 h.** A second insertion **captures Aérodrome de Wiltz-Noertrange**. Red repairs the captured bases back to full operation and flies from them. Blue has no airbase left, so it can neither regenerate nor rearm aircraft, and its air force dies out. By the end red has 46 fixed wing and 68 helicopters flying against blue's 4 helicopters.
