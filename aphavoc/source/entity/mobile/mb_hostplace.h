// The campaign's report of the sector lists to the host's roster (EEDCS-49, D3, sixth slice): see mb_hostmodel.h, ENTITY_SCAN, and mb_hostplace.c.
//
// The host owns which entities stand in which sector and in what order a scan visits them.  The campaign's sector lists remain its entity state (and the conformance mirror of the roster), and every
// write to one of them goes through two functions, insert_local_entity_into_parents_child_list and delete_local_entity_from_parents_child_list (en_list.c): the creators of every fixed, mobile and effect
// entity, the relink of a position that crosses a sector edge, the unpack of a session and the destroyers.  These are the campaign's two reports; a build with no hosted entity scan compiles them away.

#ifndef MB_HOSTPLACE_H
#define MB_HOSTPLACE_H

struct ENTITY;

#ifdef EECH_CAMPAIGN_HOST_ENTITY

// `en` was inserted at the head of a sector's list: the host places it and answers the key it issued.
void campaign_place_insert (struct ENTITY *en);

// `en` is about to leave its sector's list: the host removes the key it issued for it.
void campaign_place_remove (struct ENTITY *en);

// the heap slot of `en` is released: an entity that is still placed is removed first (a path that frees an entity without unlinking it from its list)
void campaign_place_release_entity (struct ENTITY *en);

// a new boot: the host's roster and the campaign's keys both end
void campaign_place_reset (void);

// the entity the host's roster key names, or NULL when the campaign holds none for it (the key is the host's, never an index of the entity heap)
struct ENTITY *campaign_place_entity (const char *key);

#else

#define campaign_place_insert(ENTITY) ((void) 0)
#define campaign_place_remove(ENTITY) ((void) 0)
#define campaign_place_release_entity(ENTITY) ((void) 0)
#define campaign_place_reset() ((void) 0)

#endif

#endif
