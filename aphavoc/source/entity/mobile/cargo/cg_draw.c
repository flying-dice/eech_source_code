

































































#include "project.h"





#define DEBUG_MODULE 0




/*
static void draw_local_3d_object (entity *en, float range)
{
	cargo
		*raw;

	raw = get_local_entity_data (en);

	//
	// update viewpoint
	//

	if (!raw->inst3d)
	{
		return;
	}

	raw->inst3d->vp.position = raw->mob.position;

	memcpy (&raw->inst3d->vp.attitude, &raw->mob.attitude, sizeof (matrix3x3));

	//
	// animate
	//

	//
	// draw
	//

	insert_object_into_3d_scene (OBJECT_3D_DRAW_TYPE_OBJECT, raw->inst3d);
}
*/




void overload_cargo_draw_functions (void)
{
	//fn_draw_local_entity_3d_object[ENTITY_TYPE_CARGO]	= draw_local_3d_object;
}



