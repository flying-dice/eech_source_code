

































































#define DEBUG_MODULE 0





#include "project.h"





/*
static void draw_local_3d_object (entity *en, float range)
{
	pilot
		*raw;

	float
		y = 0;

	char
		s [200];

	//
	// Messages
	//

	raw = get_local_entity_data (en);

	if (get_local_entity_parent (en, LIST_TYPE_AIRCREW) == get_gunship_entity ())
	{
	
		set_viewport (main_3d_x_min, main_3d_y_min, main_3d_x_max, main_3d_y_max);
	
		if (!lock_screen (active_screen))
		{

			return;
		}
	
		set_mono_font_type (MONO_FONT_TYPE_7X12);
	
		set_mono_font_colour (hud_colour);
	

	
		set_mono_font_position (0.0, y);
	
		unlock_screen (active_screen);
	}
}





static void draw_local_2d_symbol (entity *en)
{
}
*/




void overload_pilot_draw_functions (void)
{
	//fn_draw_local_entity_3d_object[ENTITY_TYPE_PILOT]	= draw_local_3d_object;

	//fn_draw_local_entity_2d_symbol[ENTITY_TYPE_PILOT]	= draw_local_2d_symbol;
}




