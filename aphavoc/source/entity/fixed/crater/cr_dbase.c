

































































#include "project.h"





#define DEBUG_MODULE 0





const crater_data
	crater_database[NUM_CRATER_TYPES] =
	{

		//
		// CRATER_TYPE_NONE
		//

		{
			OBJECT_3D_INVALID_OBJECT_INDEX,			// object 3d shape

			0.0,												// min scale
			0.0,												// max scale
	
			0.0,												// lifetime
		},


		//
		// CRATER_TYPE_SMALL_AP_SHELL
		//

		{
			OBJECT_3D_EFFECT_CRATER,					// object 3d shape

			1.0,												// min scale
			2.0,												// max scale
	
			20.0,												// lifetime
		},


		//
		// CRATER_TYPE_LARGE_AP_SHELL
		//

		{
			OBJECT_3D_CRATER_ROUND,						// object 3d shape

			10.0,												// min scale
			20.0,												// max scale
	
			200.0,											// lifetime
		},


		//
		// CRATER_TYPE_SMALL_EXPLOSION
		//

		{
			OBJECT_3D_CRATER_ROUND,					// object 3d shape

			1.0,												// min scale
			6.0,												// max scale
	
			200.0,											// lifetime
		},


		//
		// CRATER_TYPE_MEDIUM_EXPLOSION
		//

		{
			OBJECT_3D_CRATER_ROUND,						// object 3d shape

			4.0,												// min scale
			10.0,												// max scale
	
			400.0,											// lifetime
		},


		//
		// CRATER_TYPE_LARGE_EXPLOSION
		//

		{
			OBJECT_3D_CRATER_ROUND,						// object 3d shape

			8.0,												// min scale
			16.0,												// max scale
	
			510.0,											// lifetime
		},
	};





