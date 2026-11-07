/*
 * TEST-ONLY (EEDCS-49, D3, sector-height public baseline; feature `sector-height-observer`): the observer hooks that the staged copies of en_list.c, sc_msgs.c, sc_pack.c,
 * fw_move.c, mb_hostplacemap.h, mb_hostscan.h and mb_hostoffer.h call (build/patches.rs, SECTOR_HEIGHT_OBSERVER_PATCHES). This header is force-included by the build ONLY when
 * the feature is on; with it off none of the hooks, the patches or the definitions (eech_sho.c) exist in the build. docs/d3-sector-height-public-baseline-plan.md is the plan.
 *
 * Every hook only RECORDS a value the original statement next to it has just read or is about to use, in bounded private tables (eech_sho.c). None allocates, frees, writes a
 * campaign or entity byte, draws a random number, reads a clock or does floating-point arithmetic: floats cross as pointers and are copied as 32-bit patterns; entities cross as
 * opaque pointers and are read only through the getters the original reads itself (plus the entity's own heap index, the observer's private record identity: no production semantic
 * identifier is allocated or changed). A table that fills latches an overflow counter; nothing is silently truncated, and the report says `complete=0`.
 */
#ifndef EECH_SHO_HOOKS_H
#define EECH_SHO_HOOKS_H

/* en_list.c: an entity has just been written into a SECTOR list and the roster has been told (campaign_place_insert), before LINK_CHILD; and the removal, reported while it is still listed */
extern void eech_sho_sector_insert (const void *en, const void *sector);
extern void eech_sho_sector_remove (const void *en);

/* en_heap.c set_free_entity: an entity's slot is freed; an open observer record of it is closed so that a reused index never joins an old placement */
extern void eech_sho_entity_release (const void *en);

/* sc_msgs.c response_to_link_child, list type SECTOR: before and after the tallest-structure block; `word` points at the sector's held word */
extern void eech_sho_link_begin (const void *sector, const void *sender, const float *word);
extern void eech_sho_link_end (const void *sector, const void *sender, const float *word);

/* mb_hostplacemap.h place_fixed_state: the statement the host is about to receive (a fixed family's six words, a city's ten) under the host's key */
extern void eech_sho_statement_fixed (const void *en, int type, const char *key, const float *state);
extern void eech_sho_statement_city (const void *en, const char *key, const float *city);

/* mb_hostscan.h: the FIRST read of the sector the host named, and mb_hostoffer.h: fact 12 of one candidate question (the word the question carries, related to the last FIRST) */
extern void eech_sho_scan_first (int x_sector, int z_sector, const float *word);
extern void eech_sho_candidate_fact (const float *word);

/* fw_move.c: the two runtime guards of the clearance read (task leader; terrain following), then the read itself and the final minimum height */
extern void eech_sho_clearance_leader (void);
extern void eech_sho_clearance_follow (void);
extern void eech_sho_clearance_tallest (const void *sector, const float *terrain_elevation, const float *min_height);
extern void eech_sho_clearance_final (const float *min_height);

/* sc_pack.c: the sector's word as packed (server session) and as unpacked */
extern void eech_sho_pack (const float *word, int x_sector, int z_sector);
extern void eech_sho_unpack (const float *word, int x_sector, int z_sector);

#endif
