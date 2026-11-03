

































































#include "project.h"





struct REGENERATION_OBJECT_DATABASE_STRUCTURE
{

	object_3d_index_numbers
		object_index;

	enum ENTITY_SUB_TYPE_REGEN
		regeneration_sub_type;
};

typedef struct REGENERATION_OBJECT_DATABASE_STRUCTURE regeneration_object_database_structure;





regeneration_object_database_structure
	regeneration_object_database[OBJECT_3D_LAST];








regen_list_element
	*regen_queue[NUM_ENTITY_SIDES][NUM_ENTITY_SUB_TYPE_REGENS];

regen_management_element
	regen_manager [NUM_ENTITY_SIDES][NUM_ENTITY_SUB_TYPE_REGENS];





int
	regen_to_landing_conversion [NUM_ENTITY_SUB_TYPE_REGENS];

float
	regen_frequency [NUM_ENTITY_SIDES];





void initialise_object_3d_regeneration_database ( void )
{

	int
		count;

	//
	// Setup the regeneration flags for each 3d object
	//

	regen_to_landing_conversion [ENTITY_SUB_TYPE_REGEN_NONE] = 0;
	regen_to_landing_conversion [ENTITY_SUB_TYPE_REGEN_HELICOPTER] = ENTITY_SUB_TYPE_LANDING_HELICOPTER;
	regen_to_landing_conversion [ENTITY_SUB_TYPE_REGEN_FIXED_WING] = ENTITY_SUB_TYPE_LANDING_FIXED_WING;
	regen_to_landing_conversion [ENTITY_SUB_TYPE_REGEN_GROUND] = ENTITY_SUB_TYPE_LANDING_GROUND;
	regen_to_landing_conversion [ENTITY_SUB_TYPE_REGEN_PEOPLE] = ENTITY_SUB_TYPE_LANDING_PEOPLE;
	regen_to_landing_conversion [ENTITY_SUB_TYPE_REGEN_SEA] = ENTITY_SUB_TYPE_LANDING_SEA;

	for ( count = OBJECT_3D_INVALID_OBJECT_INDEX + 1; count < OBJECT_3D_LAST; count++ )
	{

		regeneration_object_database[count].object_index = OBJECT_3D_INVALID_OBJECT_INDEX;

		regeneration_object_database[count].regeneration_sub_type = NUM_ENTITY_SUB_TYPE_REGENS;
	}










}






























































enum ENTITY_SUB_TYPE_REGEN get_object_3d_regeneration_type ( object_3d_index_numbers object, int *landing_sub_type )
{

	ASSERT ( object > OBJECT_3D_INVALID_OBJECT_INDEX );

	ASSERT ( object < OBJECT_3D_LAST );

	*landing_sub_type = NUM_ENTITY_SUB_TYPE_LANDINGS;

	if ( regeneration_object_database[object].regeneration_sub_type == NUM_ENTITY_SUB_TYPE_REGENS )
	{

		object_3d_instance
			*object_instance;

		object_3d_sub_object_search_data
			search;

		object_instance = construct_3d_object ( object );

		search.search_object = object_instance;
		search.search_depth = 0;
		search.sub_object_index = OBJECT_3D_SUB_OBJECT_REGEN_FIXED_WING;
	
		if ( find_object_3d_sub_object ( &search ) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND )
		{

			regeneration_object_database[object].regeneration_sub_type = ENTITY_SUB_TYPE_REGEN_FIXED_WING;

			*landing_sub_type = ENTITY_SUB_TYPE_LANDING_FIXED_WING;
		}

		search.search_object = object_instance;
		search.search_depth = 0;
		search.sub_object_index = OBJECT_3D_SUB_OBJECT_REGEN_HELICOPTER;
	
		if ( find_object_3d_sub_object ( &search ) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND )
		{

			regeneration_object_database[object].regeneration_sub_type = ENTITY_SUB_TYPE_REGEN_HELICOPTER;

			*landing_sub_type = ENTITY_SUB_TYPE_LANDING_HELICOPTER;
		}

		search.search_object = object_instance;
		search.search_depth = 0;
		search.sub_object_index = OBJECT_3D_SUB_OBJECT_REGEN_ROUTED_VEHICLE;
	
		if ( find_object_3d_sub_object ( &search ) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND )
		{

			regeneration_object_database[object].regeneration_sub_type = ENTITY_SUB_TYPE_REGEN_GROUND;

			*landing_sub_type = ENTITY_SUB_TYPE_LANDING_GROUND;
		}

		search.search_object = object_instance;
		search.search_depth = 0;
		search.sub_object_index = OBJECT_3D_SUB_OBJECT_REGEN_PERSON;
	
		if ( find_object_3d_sub_object ( &search ) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND )
		{

			regeneration_object_database[object].regeneration_sub_type = ENTITY_SUB_TYPE_REGEN_PEOPLE;

			*landing_sub_type = ENTITY_SUB_TYPE_LANDING_PEOPLE;
		}

/*
		search.object = object_instance;
		search.search_depth = 0;
		search.sub_object_index = OBJECT_3D_SUB_OBJECT_REGEN_SHIP_VEHICLE;
	
		if ( find_object_3d_sub_object ( &search ) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND )
		{

			regeneration_object_database[object].regeneration_sub_type = ENTITY_SUB_TYPE_REGEN_SHIP_VEHICLE;

			*landing_sub_type = ENTITY_SUB_TYPE_LANDING_SEA;
		}
*/
		destruct_3d_object ( object_instance );

		if ( regeneration_object_database[object].regeneration_sub_type == NUM_ENTITY_SUB_TYPE_REGENS )
		{

			regeneration_object_database[object].regeneration_sub_type = ENTITY_SUB_TYPE_REGEN_NONE;
		}
	}

	switch ( regeneration_object_database[object].regeneration_sub_type )
	{

		case ENTITY_SUB_TYPE_REGEN_FIXED_WING:	{ *landing_sub_type = ENTITY_SUB_TYPE_LANDING_FIXED_WING; break; }
		case ENTITY_SUB_TYPE_REGEN_HELICOPTER:	{ *landing_sub_type = ENTITY_SUB_TYPE_LANDING_HELICOPTER; break; }
		case ENTITY_SUB_TYPE_REGEN_GROUND:		{ *landing_sub_type = ENTITY_SUB_TYPE_LANDING_GROUND; break; }
		case ENTITY_SUB_TYPE_REGEN_PEOPLE:		{ *landing_sub_type = ENTITY_SUB_TYPE_LANDING_PEOPLE; break; }
		case ENTITY_SUB_TYPE_REGEN_NONE:			{ *landing_sub_type = NUM_ENTITY_SUB_TYPE_LANDINGS; break; }
		default:
		{

			debug_fatal ( "REGEN: unknown regen type" );
		}
	}

	return ( regeneration_object_database[object].regeneration_sub_type );
}





float get_regen_frequency_difficulty_modifier (void)
{
	//
	// Modify Regen Frequency wrt difficulty level
	//

	// (just use medium for now..)
	return REGEN_UPDATE_MEDIUM	;
}





