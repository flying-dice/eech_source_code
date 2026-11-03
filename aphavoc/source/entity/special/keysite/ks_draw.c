

































































#define DEBUG_MODULE 0





#include "project.h"





static void draw_local_2d_symbol (entity *en)
{
	keysite
		*raw;

	graphics_file_specifiers
		icon_graphic;

	int
		screen_x,
		screen_y;

	raw = get_local_entity_data (en);

	if ((!raw->alive) || (!raw->in_use))
	{
		return;
	}

	//
	// Calculate window coords
	//

	reset_2d_instance (active_2d_environment);

	get_2d_int_screen_coordinates (raw->position.x, raw->position.z, &screen_x, &screen_y);

	icon_graphic = get_local_entity_int_value (en, INT_TYPE_2D_SYMBOL_ICON);

	blit_rgb_centralised_clipped_graphic (get_graphics_file_data (icon_graphic), screen_x, screen_y);
}





void overload_keysite_draw_functions (void)
{

	fn_draw_local_entity_2d_symbol[ENTITY_TYPE_KEYSITE]	= draw_local_2d_symbol;
}




