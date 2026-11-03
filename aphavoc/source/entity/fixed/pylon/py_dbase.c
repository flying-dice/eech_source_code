

































































#include "project.h"





#define DEBUG_MODULE 0





static struct PYLON_TYPE_CONVERSION_TABLE
{
	int			entity_sub_type;
	int			pylon_type;
} pylon_type_conversion_table[] =
	{
		ENTITY_SUB_TYPE_FIXED_ELECTRICITY_PYLON_STANDARD,				PYLON_TYPE_STANDARD,
		ENTITY_SUB_TYPE_FIXED_ELECTRICITY_PYLON_START_TERMINATOR,	PYLON_TYPE_START_TERMINATOR,
		ENTITY_SUB_TYPE_FIXED_ELECTRICITY_PYLON_END_TERMINATOR,		PYLON_TYPE_END_TERMINATOR,
		-1
	};





pylon_attachment_data
	pylon_attachment_database[NUM_PYLON_TYPES] =
	{

		//
		//	STANDARD PYLON
		//

		{
			OBJECT_3D_PYLON,			// default object 3d shape

			{ 2, 2, 0, 0, 1, 1 },

			{
				{-11.3, 28.8, 0.0},
				{11.3, 28.8, 0.0},
				{-16.3, 28.8, 0.0},
				{16.3, 28.8, 0.0},
				{-11.3, 39.5, 0.0},
				{11.3, 39.5, 0.0},
			},
		},

		//
		//	START TERMINATOR PYLON
		//

		{
			OBJECT_3D_PYLON_TERMINATOR,			// default object 3d shape

			{ 2, 2, 0, 0, 1, 1 },

			{
				{-11.3, 28.8, 0.0},
				{11.3, 28.8, 0.0},
				{-16.3, 28.8, 0.0},
				{16.3, 28.8, 0.0},
				{-11.3, 39.5, 0.0},
				{11.3, 39.5, 0.0},
			},
		},

		//
		//	END TERMINATOR PYLON
		//

		{
			OBJECT_3D_PYLON_TERMINATOR,			// default object 3d shape

			{ 2, 2, 0, 0, 1, 1 },

			{
				{11.3, 28.8, 0.0},
				{-11.3, 28.8, 0.0},
				{16.3, 28.8, 0.0},
				{-16.3, 28.8, 0.0},
				{11.3, 39.5, 0.0},
				{-11.3, 39.5, 0.0},
			},
		},
	};



static float
	pylon_sag_lookup_table[ NUMBER_OF_PYLON_CABLE_POINTS ];





void initialise_pylon_database()
{
	int
		loop;

	float
		x;
	
	//
	// construct sag look-up table
	//

	for ( loop = 0 ; loop < NUMBER_OF_PYLON_CABLE_POINTS ; loop ++ )
	{
		x =(float)loop / ( NUMBER_OF_PYLON_CABLE_POINTS - 1 );

		x = ( 2.0 * x ) - 1.0;
		
		pylon_sag_lookup_table[ loop ] = 1.0 - ( x * x );
	}
}





float get_pylon_sag_value( int index )
{
	return pylon_sag_lookup_table[ index ];
}





pylon_types get_pylon_sub_type( entity_sub_types sub_type )
{
	int
		loop;

	loop = 0;

	while ( pylon_type_conversion_table[ loop ].entity_sub_type >= 0 )
	{
		if ( pylon_type_conversion_table[ loop ].entity_sub_type == sub_type )
		{
			return (pylon_types) pylon_type_conversion_table[ loop ].pylon_type;
		}

		loop ++;
	}

	debug_fatal( "Fixed sub type %d is not a valid pylon type", sub_type );

	return NUM_PYLON_TYPES;
}





