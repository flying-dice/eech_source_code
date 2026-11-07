def patch(p, pairs):
    d = open(p, "rb").read().decode("utf-8")
    assert b"\r\n" not in open(p, "rb").read(), p
    for old, new in pairs:
        assert d.count(old) == 1, (p, old[:70], d.count(old))
        d = d.replace(old, new)
    open(p, "wb").write(d.encode("utf-8"))

patch("aphavoc/source/entity/mobile/mb_hostmodel.h", [(
" *              entity_scan.roster_key, entity_scan.key) and never an end, a miss or a fallback to the list. STILL THE CAMPAIGN'S, declared and required to move: the tallest structure of a sector, a city's\n *              buildings, every entity's shape, flags and static pose, and the semantic identity of a structure (issued lazily, at the first time the campaign names it, as before). No raw pointer or",
" *              entity_scan.roster_key, entity_scan.key) and never an end, a miss or a fallback to the list. THE TALLEST STRUCTURE OF A SECTOR is the HOST's word since the host-owned sector height (Task49): the host\n *              raises it at the ARRIVAL of a fixed structure (the first state statement under a sector placement's key), holds it in its roster, and the campaign asks it by the sector's coordinates (phase 17) and states a\n *              session's saved word to it (phase 18); the campaign holds no word of its own in a hosted build. STILL THE CAMPAIGN'S, declared and required to move: a city's\n *              buildings, every entity's shape, flags and static pose, and the semantic identity of a structure (issued lazily, at the first time the campaign names it, as before). No raw pointer or")])

patch("aphavoc/source/entity/mobile/mb_hostscan.h", [
(" * list and the predecessor of an entry), because the campaign's own lists remain its entity state and the tallest-structure height is still its read; but the entry served is the one the HOST names, and",
 " * list and the predecessor of an entry), because the campaign's own lists remain its entity state; the tallest-structure height is read through the sector's getter, which asks the HOST (the host-owned sector height, Task49);\n * but the entry served is the one the HOST names, and"),
(" * performs, UPDATED and the terrain normal, change a flag word and the shared terrain record, never a list).  Still the campaign's, declared and required to move: the tallest structure of a sector, a\n * city's buildings and every entity's shape, flags and static pose.",
 " * performs, UPDATED and the terrain normal, change a flag word and the shared terrain record, never a list).  Still the campaign's, declared and required to move: a\n * city's buildings and every entity's shape, flags and static pose."),
])

patch("aphavoc/source/entity/en_misc/en_maths/en_cllsn.c", [(
"the exclusion of the source and the target (a comparison only the entity heap can make), the tallest structure of a sector, a city's buildings,",
"the exclusion of the source and the target (a comparison only the entity heap can make), a city's buildings (the tallest structure of a sector is the host's word since Task49: the sector's getter asks it),")])

patch("eech-campaign/crates/eech-world/src/entity_host.rs", [(
"The campaign still owns the exclusion of the source and the target, a sector's tallest structure, a city's buildings, the reads of the entity state the",
"The campaign still owns the exclusion of the source and the target, a city's buildings (a sector's tallest structure is the host's: `crate::roster`), the reads of the entity state the")])

patch("eech-campaign/crates/eech-world/csrc/entity_scan_step.c", [(
"What the campaign keeps, named and REQUIRED to move: the tallest structure of a sector, a city's buildings,",
"What the campaign keeps, named and REQUIRED to move: a city's buildings (a sector's tallest structure is the host's word since Task49),")])
print("ok")
