

































































#define DEBUG_MODULE 0





#include "project.h"





static void draw_local_3d_object (entity *en, float range)
{
}





static void draw_local_2d_symbol (entity *en)
{

	landing
		*raw;

	raw = get_local_entity_data (en);

	set_2d_instance_position (active_2d_environment, raw->position.x, raw->position.z);

	draw_2d_cross (sys_col_yellow);
}





void overload_landing_draw_functions (void)
{
	fn_draw_local_entity_3d_object[ENTITY_TYPE_LANDING]	= draw_local_3d_object;

	fn_draw_local_entity_2d_symbol[ENTITY_TYPE_LANDING]	= draw_local_2d_symbol;
}




