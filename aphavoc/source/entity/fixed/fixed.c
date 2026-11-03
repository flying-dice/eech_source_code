

































































#include "project.h"





#define DEBUG_MODULE 0









































































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







