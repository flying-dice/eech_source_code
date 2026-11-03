

































































#include "project.h"





#define DEBUG_MODULE 0





static terrain_database_type
	terrain_database [TERRAIN_TYPE_LAST];





void set_terrain_database_element (terrain_types type, terrain_classes terr_class, int draw_crater, int raised, int suitable_for_ground_vehicle)
{

	ASSERT (type >= TERRAIN_TYPE_INVALID);
	ASSERT (type < TERRAIN_TYPE_LAST);

	terrain_database [type].terr_class = terr_class;

	terrain_database [type].draw_crater = draw_crater;

	terrain_database [type].raised = raised;

	terrain_database [type].suitable_for_ground_vehicle = suitable_for_ground_vehicle;
}





terrain_classes get_terrain_type_class ( terrain_types type )
{
	ASSERT (type >= TERRAIN_TYPE_INVALID);
	ASSERT (type < TERRAIN_TYPE_LAST);

	return terrain_database [type].terr_class;
}





int get_terrain_type_draw_crater (terrain_types type)
{
	ASSERT (type >= TERRAIN_TYPE_INVALID);
	ASSERT (type < TERRAIN_TYPE_LAST);

	return terrain_database [type].draw_crater;
}





int get_terrain_type_raised (terrain_types type)
{
	ASSERT (type >= TERRAIN_TYPE_INVALID);
	ASSERT (type < TERRAIN_TYPE_LAST);

	return terrain_database [type].raised;
}





int get_terrain_type_suitable_for_ground_vehicle (terrain_types type)
{
	ASSERT (type >= TERRAIN_TYPE_INVALID);
	ASSERT (type < TERRAIN_TYPE_LAST);

	return terrain_database [type].suitable_for_ground_vehicle;
}





void initialise_terrain_database ()
{

	//
	// parameters : (type, terr_class, draw_crater, raised, suitable_for_ground_vehicle)
	//

	set_terrain_database_element (TERRAIN_TYPE_INVALID, TERRAIN_CLASS_INVALID, FALSE, FALSE, FALSE);

	// Sunken features tops
	set_terrain_database_element (TERRAIN_TYPE_RIVER, TERRAIN_CLASS_WATER, FALSE, FALSE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_RESERVOIR, TERRAIN_CLASS_WATER, FALSE, FALSE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_TRENCH, TERRAIN_CLASS_LAND, FALSE, TRUE, FALSE);

	// Sunken features banks
	set_terrain_database_element (TERRAIN_TYPE_RIVER_BANK, TERRAIN_CLASS_LAND, TRUE, FALSE, FALSE);

	// Regular terrain types
	set_terrain_database_element (TERRAIN_TYPE_SEA, TERRAIN_CLASS_WATER, FALSE, FALSE, FALSE);

	set_terrain_database_element (TERRAIN_TYPE_BEACH, TERRAIN_CLASS_LAND, TRUE, FALSE, TRUE);

	set_terrain_database_element (TERRAIN_TYPE_LAND, TERRAIN_CLASS_LAND, TRUE, FALSE, TRUE);

	set_terrain_database_element (TERRAIN_TYPE_FIELD1, TERRAIN_CLASS_LAND, TRUE, FALSE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_FIELD2, TERRAIN_CLASS_LAND, TRUE, FALSE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_FIELD3, TERRAIN_CLASS_LAND, TRUE, FALSE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_FIELD4, TERRAIN_CLASS_LAND, TRUE, FALSE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_FIELD5, TERRAIN_CLASS_LAND, TRUE, FALSE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_FIELD6, TERRAIN_CLASS_LAND, TRUE, FALSE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_FIELD7, TERRAIN_CLASS_LAND, TRUE, FALSE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_FIELD8, TERRAIN_CLASS_LAND, TRUE, FALSE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_FIELD9, TERRAIN_CLASS_LAND, TRUE, FALSE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_FIELD10, TERRAIN_CLASS_LAND, TRUE, FALSE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_FIELD11, TERRAIN_CLASS_LAND, TRUE, FALSE, FALSE);

	set_terrain_database_element (TERRAIN_TYPE_ALTERED_LAND1, TERRAIN_CLASS_LAND, TRUE, FALSE, TRUE);
	set_terrain_database_element (TERRAIN_TYPE_ALTERED_LAND2, TERRAIN_CLASS_LAND, TRUE, FALSE, TRUE);
	set_terrain_database_element (TERRAIN_TYPE_ALTERED_LAND3, TERRAIN_CLASS_LAND, TRUE, FALSE, TRUE);

	set_terrain_database_element (TERRAIN_TYPE_FOREST_FLOOR, TERRAIN_CLASS_LAND, TRUE, FALSE, FALSE);

	// Raised features banks
	set_terrain_database_element (TERRAIN_TYPE_ROAD_BANK, TERRAIN_CLASS_LAND, TRUE, FALSE, TRUE);
	set_terrain_database_element (TERRAIN_TYPE_RAIL_BANK, TERRAIN_CLASS_LAND, TRUE, FALSE, TRUE);

	// Raised features sides
	set_terrain_database_element (TERRAIN_TYPE_HEDGE_SIDE_X, TERRAIN_CLASS_LAND, FALSE, TRUE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_HEDGE_SIDE_Z, TERRAIN_CLASS_LAND, FALSE, TRUE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_WALL_SIDE_X, TERRAIN_CLASS_LAND, FALSE, TRUE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_WALL_SIDE_Z, TERRAIN_CLASS_LAND, FALSE, TRUE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_TRENCH_SIDE_X, TERRAIN_CLASS_LAND, FALSE, TRUE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_TRENCH_SIDE_Z, TERRAIN_CLASS_LAND, FALSE, TRUE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_FOREST_SIDE_BOTTOM_X, TERRAIN_CLASS_FOREST, FALSE, TRUE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_FOREST_SIDE_BOTTOM_Z, TERRAIN_CLASS_FOREST, FALSE, TRUE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_FOREST_SIDE_MID_X, TERRAIN_CLASS_FOREST, FALSE, TRUE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_FOREST_SIDE_MID_Z, TERRAIN_CLASS_FOREST, FALSE, TRUE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_FOREST_SIDE_TOP_X, TERRAIN_CLASS_FOREST, FALSE, TRUE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_FOREST_SIDE_TOP_Z, TERRAIN_CLASS_FOREST, FALSE, TRUE, FALSE);

	// Raised features tops
	set_terrain_database_element (TERRAIN_TYPE_BUILT_UP_AREA1, TERRAIN_CLASS_LAND, TRUE, FALSE, TRUE);
	set_terrain_database_element (TERRAIN_TYPE_BUILT_UP_AREA2, TERRAIN_CLASS_LAND, TRUE, FALSE, TRUE);
	set_terrain_database_element (TERRAIN_TYPE_BUILT_UP_AREA3, TERRAIN_CLASS_LAND, TRUE, FALSE, TRUE);
	set_terrain_database_element (TERRAIN_TYPE_BUILT_UP_AREA4, TERRAIN_CLASS_LAND, TRUE, FALSE, TRUE);
	set_terrain_database_element (TERRAIN_TYPE_ROAD, TERRAIN_CLASS_LAND, TRUE, FALSE, TRUE);
	set_terrain_database_element (TERRAIN_TYPE_TRACK, TERRAIN_CLASS_LAND, TRUE, FALSE, TRUE);
	set_terrain_database_element (TERRAIN_TYPE_RAIL, TERRAIN_CLASS_LAND, TRUE, FALSE, TRUE);

	set_terrain_database_element (TERRAIN_TYPE_WALL_TOP, TERRAIN_CLASS_LAND, FALSE, TRUE, FALSE);
	set_terrain_database_element (TERRAIN_TYPE_HEDGE_TOP, TERRAIN_CLASS_LAND, FALSE, TRUE, FALSE);

	set_terrain_database_element (TERRAIN_TYPE_FOREST_TOP, TERRAIN_CLASS_FOREST, FALSE, TRUE, FALSE);
}







