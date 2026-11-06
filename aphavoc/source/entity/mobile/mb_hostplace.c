// See mb_hostplace.h: the campaign's report of the sector lists to the host's roster (EEDCS-49, D3, sixth slice).  The table of keys, the insertion, removal, release and reset are the shared header
// mb_hostplacemap.h; this binds it to the real entity heap.

#include "project.h"

#include "entity/mobile/mb_hostmodel.h"

#include "entity/mobile/mb_hostobj.h"

#include "entity/mobile/mb_hostplace.h"

#ifdef EECH_CAMPAIGN_HOST_ENTITY

static int place_heap (void)
{
	if ((number_of_entities <= 0) || (number_of_entities > MAX_NUM_ENTITIES))
	{
		debug_fatal ("host roster: the entity heap holds %d slots, which is not a heap this run allocated", number_of_entities);
	}

	return number_of_entities;
}

// the entity's three position words: the position getter answers the entity's own vec3d, three floats in a row, or NULL for an entity that has none
static const float *place_position (struct ENTITY *en)
{
	return (const float *) get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);
}

// the legacy object-report identity already issued for a STRUCTURE (campaign_object_host_id answers a hosted mobile's roster identity for any other entity, which is not an object identity and is never named here); NULL
// when none was issued yet: it is never allocated by a placement
static const char *place_object_id (struct ENTITY *en)
{
	if (!campaign_object_is_structure ((int) get_local_entity_type (en)))
	{
		return NULL;
	}

	return campaign_object_host_id (en);
}

#define CAMPAIGN_PLACE_ENTITY struct ENTITY
#define CAMPAIGN_PLACE_HEAP() place_heap ()
#define CAMPAIGN_PLACE_SLOT(E) get_local_entity_index ((E))
#define CAMPAIGN_PLACE_AT(SLOT) get_local_entity_ptr ((SLOT))
#define CAMPAIGN_PLACE_TYPE(E) ((int) get_local_entity_type ((E)))
#define CAMPAIGN_PLACE_NAME(E) get_local_entity_type_name ((E))
#define CAMPAIGN_PLACE_POSITION(E) place_position ((E))
#define CAMPAIGN_PLACE_IDENTITY(E) campaign_roster_host_id ((E))
#define CAMPAIGN_PLACE_OBJECT_ID(E) place_object_id ((E))
#define CAMPAIGN_PLACE_SIDE() ((float) SECTOR_SIDE_LENGTH)

#include "entity/mobile/mb_hostplacemap.h"

void campaign_place_insert (struct ENTITY *en)
{
	place_insert (en);
}

void campaign_place_remove (struct ENTITY *en)
{
	place_remove (en);
}

void campaign_place_release_entity (struct ENTITY *en)
{
	place_release_entity (en);
}

void campaign_place_reset (void)
{
	place_reset ();
}

struct ENTITY *campaign_place_entity (const char *key)
{
	return place_entity (key);
}

void campaign_place_child_insert (struct ENTITY *en, struct ENTITY *parent, struct ENTITY *pred)
{
	place_child_insert (en, parent, pred);
}

void campaign_place_child_remove (struct ENTITY *en)
{
	place_child_remove (en);
}

struct ENTITY *campaign_place_child_first (struct ENTITY *city, struct ENTITY *mirror)
{
	return place_child_first (city, mirror);
}

struct ENTITY *campaign_place_child_next (struct ENTITY *city, struct ENTITY *building, struct ENTITY *mirror)
{
	return place_child_next (city, building, mirror);
}

void campaign_place_name (struct ENTITY *en, const char *id)
{
	place_name (en, id);
}

#endif
