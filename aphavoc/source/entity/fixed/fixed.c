

































































#include "project.h"





#define DEBUG_MODULE 0





static int
	num_building_destroyed_cpg_comments [NUM_ENTITY_SIDES] = {0, 2, 2},
	last_building_destroyed_cpg_comment [NUM_ENTITY_SIDES] = {0, 0, 0};

static speech_instance_type
	building_destroyed_cpg_comments [] =
	{
		{ -1,												(speech_array_types) -1, 										100,	SPEECH_INTRODUCTION_NEVER },
		{ SPEECH_CPG_TARGET_DESTROYED,						SPEECH_ARRAY_CPG_MESSAGES,			25,	SPEECH_INTRODUCTION_SOMETIMES },
	};


























































void restore_local_fixed_entity (entity *en)
{
	int
		object_index;

	entity
		*keysite;

	ASSERT (en);

	//
	// Set alive flag
	//

	ASSERT (!get_local_entity_int_value (en, INT_TYPE_ALIVE));

	set_local_entity_int_value (en, INT_TYPE_ALIVE, TRUE);

	//
	// 3d shape
	//

	object_index = get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);

	set_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE, get_3d_object_restored_object_index (object_index));

	//
	// Restore damage level (MUST be done AFTER object 3d shape reset)
	//

	set_local_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL, get_local_entity_int_value (en, INT_TYPE_INITIAL_DAMAGE_LEVEL));

	ASSERT (get_local_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL) > 0);

	if (get_local_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL) <= 0) {
		kill_local_entity(en);
		return;
	}

	//
	// Keysite strength
	//

	keysite = get_fixed_structure_keysite_entity (en);

	ASSERT (keysite);
	
	restore_local_entity_importance_to_keysite (en, keysite);
}







