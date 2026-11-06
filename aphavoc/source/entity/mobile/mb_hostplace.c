// See mb_hostplace.h: the campaign's report of the sector lists to the host's roster (EEDCS-49, D3, sixth slice).  The table of keys, the insertion, removal, release and reset are the shared header
// mb_hostplacemap.h; this binds it to the real entity heap.

#include "project.h"

#include "entity/mobile/mb_hostmodel.h"

#include "entity/mobile/mb_hostobj.h"

#include "entity/mobile/mb_hostplace.h"

// the object-report identity a NAME carries has its own domain beside the placement key's, and the model request's identity field is sized from it (a mismatch is a negative array size)
typedef char campaign_model_object_id_domain_is_the_object_registrys [(CAMPAIGN_MODEL_OBJECT_ID_MAX == CAMPAIGN_OBJECT_ID_MAX) ? 1 : -1];

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

// the fixed family: the entity types that carry the fixed record (fixed.h) whose collision state the host's record of a placement holds. A city carries it too but is not one here: its own position and
// bounding box (ct_vec3d.c) are a declared follow-on, not moved by this
static int place_fixed_family (int type)
{
	switch ((entity_types) type)
	{
		case ENTITY_TYPE_CITY_BUILDING:
		case ENTITY_TYPE_CRATER:
		case ENTITY_TYPE_OBJECT:
		case ENTITY_TYPE_PYLON:
		case ENTITY_TYPE_SCENIC:
		case ENTITY_TYPE_SEGMENT:
		case ENTITY_TYPE_SITE:
		case ENTITY_TYPE_SITE_UPDATABLE:
		{
			return TRUE;
		}
		default:
		{
			return FALSE;
		}
	}
}

// the entity's fixed collision state as it is now, with the entity's own getters (the same ones the scan reads): the position's three words, the heading, the shape (a scene index: a whole number a float holds
// exactly) and the two flags its sub type derives. The position is copied as the words the entity holds.
static void place_fixed_read (struct ENTITY *en, float *state)
{
	const float
		*position;

	position = place_position (en);

	memcpy (state, position, 3 * sizeof (float));

	state [3] = get_local_entity_float_value (en, FLOAT_TYPE_HEADING);

	state [4] = (float) get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);

	state [5] = (get_local_entity_int_value (en, INT_TYPE_COLLISION_TEST_FIXED) != 0) ? 1.0f : 0.0f;

	state [6] = (get_local_entity_int_value (en, INT_TYPE_ALIGN_WITH_TERRAIN) != 0) ? 1.0f : 0.0f;
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
#define CAMPAIGN_PLACE_FIXED_FAMILY(TYPE) place_fixed_family ((TYPE))
#define CAMPAIGN_PLACE_FIXED_READ(E,OUT) place_fixed_read ((E), (OUT))

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

void campaign_place_fixed_state (struct ENTITY *en)
{
	place_fixed_state (en);
}

const char *campaign_place_key (struct ENTITY *en, int child)
{
	return place_key (en, child);
}

#endif
