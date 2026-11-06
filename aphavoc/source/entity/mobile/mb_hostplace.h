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

// the city lists (LIST_TYPE_CITY_BUILDING; seventh slice): `en` was inserted into the list of the buildings of `parent` at the head (`pred` NULL) or after `pred`, and is about to leave it. The host lists them
// in ITS order and a scan reads that order: `campaign_place_child_first` and `_next` answer the building the host serves, resolved to its entity and compared with `mirror`, the entry the campaign's own list holds
// at the same place (NULL at its end); a disagreement or a key that names no building stops the engine by name.
void campaign_place_child_insert (struct ENTITY *en, struct ENTITY *parent, struct ENTITY *pred);

void campaign_place_child_remove (struct ENTITY *en);

struct ENTITY *campaign_place_child_first (struct ENTITY *city, struct ENTITY *mirror);

struct ENTITY *campaign_place_child_next (struct ENTITY *city, struct ENTITY *building, struct ENTITY *mirror);

// the host issued `id` as the legacy object-report identity of `en` (asked lazily, once): the host's placements of the entity are named by it
void campaign_place_name (struct ENTITY *en, const char *id);

// the fixed collision state of `en` (a fixed-family entity: position, heading, shape and the two flags its sub type derives) was written, or the entity was placed: the host's record of the placement holds the state
// as it is now (Task49). Called by every writer of a field of it after the entity is placed; an entity that is not placed, and a type that is neither of the fixed family nor the city, state nothing. A CITY states
// its own position and bounding box through the same call (the expansion a building's link makes, ct_msgs.c, is its writer of the box). See mb_hostplacemap.h.
void campaign_place_fixed_state (struct ENTITY *en);

// the key the host issued for the placement `en` has now (its sector placement's, or with `child` its place in a city's list), "" when it has none: a candidate that is not a mobile names it on every staged question
const char *campaign_place_key (struct ENTITY *en, int child);

#else

#define campaign_place_insert(ENTITY) ((void) 0)
#define campaign_place_remove(ENTITY) ((void) 0)
#define campaign_place_release_entity(ENTITY) ((void) 0)
#define campaign_place_reset() ((void) 0)
#define campaign_place_child_insert(ENTITY,PARENT,PRED) ((void) 0)
#define campaign_place_child_remove(ENTITY) ((void) 0)
#define campaign_place_name(ENTITY,ID) ((void) 0)
#define campaign_place_fixed_state(ENTITY) ((void) 0)

#endif

#endif
