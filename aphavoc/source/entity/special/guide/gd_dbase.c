

































































#include "project.h"





#define DEBUG_MODULE 0





//VJ 040321 GWUT change
//remove "const" from definition
guide_data
   guide_database[NUM_ENTITY_SUB_TYPE_GUIDES] =
	{

      //
      // ENTITY_SUB_TYPE_GUIDE_NAVIGATION_DIRECT
      //

      {
			TRUE,															// evade fire
			TRUE,															// follow formation
			GUIDE_POSITION_WAYPOINT,								// position type
			GUIDE_TERRAIN_FOLLOW_CRUISE_ALTITUDE,				// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
      // ENTITY_SUB_TYPE_GUIDE_NAVIGATION_VIRTUAL
      //

      {
			TRUE,															// evade fire
			TRUE,															// follow formation
			GUIDE_POSITION_RELATIVE_WORLD_SPACE,				// position type
			GUIDE_TERRAIN_FOLLOW_CRUISE_ALTITUDE,				// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
      // ENTITY_SUB_TYPE_GUIDE_NAVIGATION_ROUTED
      //

      {
			FALSE,														// evade fire
			TRUE,															// follow formation
			GUIDE_POSITION_WAYPOINT,								// position type
			GUIDE_TERRAIN_FOLLOW_NONE,								// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
      // ENTITY_SUB_TYPE_GUIDE_NAVIGATION_ALTITUDE
      //

      {
			FALSE,														// evade fire
			TRUE,															// follow formation
			GUIDE_POSITION_WAYPOINT,								// position type
			GUIDE_TERRAIN_FOLLOW_NONE,								// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },							// GUIDE_CRITERIA_HEADING
				{ TRUE,			0.5 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
      // ENTITY_SUB_TYPE_GUIDE_LANDING_DIRECT
      //

      {
			FALSE,														// evade fire
			TRUE,															// follow formation
			GUIDE_POSITION_WAYPOINT,								// position type
			GUIDE_TERRAIN_FOLLOW_NONE,								// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
      // ENTITY_SUB_TYPE_GUIDE_LANDED
      //

      {
			FALSE,														// evade fire
			TRUE,															// follow formation
			GUIDE_POSITION_WAYPOINT,								// position type
			GUIDE_TERRAIN_FOLLOW_NONE,								// terrain check
//VJ FARP bug, date: 18-mar-03
			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
//VJ FARP bug: changed heading from 1 to 30, additional movement seems to stop bunny hopping 
//VJ 03/10/28: FARP bug: changed heading to 360, meaing all directions count 
				{ TRUE,		rad (360.0) },							// GUIDE_CRITERIA_HEADING				
//VJ FARP bug: changed altitude boundary from 0.005 to 0.5 m stops bunny hopping 
				{ TRUE,		0.5 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
//VJ FARP bug, date: 18-mar-03, end
      },

      //
		//	ENTITY_SUB_TYPE_GUIDE_ATTACK_AA_MOVE_SIX,
      //

      {
			TRUE,															// evade fire
			FALSE,														// follow formation
			GUIDE_POSITION_ABSOLUTE,								// position type
			GUIDE_TERRAIN_FOLLOW_CRUISE_ALTITUDE,				// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
		//	ENTITY_SUB_TYPE_GUIDE_ATTACK_AA_MOVE_CIRCLE,
      //

      {
			TRUE,															// evade fire
			FALSE,														// follow formation
			GUIDE_POSITION_ABSOLUTE,								// position type
			GUIDE_TERRAIN_FOLLOW_CRUISE_ALTITUDE,				// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
		//	ENTITY_SUB_TYPE_GUIDE_ATTACK_AA_FIRE_INTERCEPT,
      //

      {
			FALSE,														// evade fire
			FALSE,														// follow formation
			GUIDE_POSITION_RELATIVE_OBJECT_SPACE,				// position type
			GUIDE_TERRAIN_FOLLOW_CRUISE_ALTITUDE,				// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
		//	ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_APPROACH,
      //

      {
			TRUE,															// evade fire
			FALSE,														// follow formation
			GUIDE_POSITION_ABSOLUTE,								// position type
			GUIDE_TERRAIN_FOLLOW_ATTACK_ALTITUDE,				// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
		//	ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_SEEK_COVER,
      //

      {
			TRUE,															// evade fire
			FALSE,														// follow formation
			GUIDE_POSITION_ABSOLUTE,								// position type
			GUIDE_TERRAIN_FOLLOW_ATTACK_ALTITUDE,				// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
		//	ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_FLY_TO_COVER,
      //

      {
			TRUE,															// evade fire
			FALSE,														// follow formation
			GUIDE_POSITION_ABSOLUTE,								// position type
			GUIDE_TERRAIN_FOLLOW_ATTACK_ALTITUDE,				// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
		//	ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_TAKE_COVER,
      //

      {
			TRUE,															// evade fire
			FALSE,														// follow formation
			GUIDE_POSITION_ABSOLUTE,								// position type
			GUIDE_TERRAIN_FOLLOW_NONE,								// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
		//	ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_CLIMB,
      //

      {
			TRUE,															// evade fire
			FALSE,														// follow formation
			GUIDE_POSITION_ABSOLUTE,								// position type
			GUIDE_TERRAIN_FOLLOW_ATTACK_ALTITUDE,				// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
		//	ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_DIVE,
      //

      {
			TRUE,															// evade fire
			FALSE,														// follow formation
			GUIDE_POSITION_ABSOLUTE,								// position type
			GUIDE_TERRAIN_FOLLOW_CRUISE_ALTITUDE,				// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
		//	ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_FIRE,
      //

      {
			TRUE,															// evade fire
			FALSE,														// follow formation
			GUIDE_POSITION_ABSOLUTE,								// position type
			GUIDE_TERRAIN_FOLLOW_NONE,								// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
		//	ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_FIRE_VIRTUAL,
      //

      {
			TRUE,															// evade fire
			FALSE,														// follow formation
			GUIDE_POSITION_RELATIVE_OBJECT_SPACE,				// position type
			GUIDE_TERRAIN_FOLLOW_NONE,								// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
		//	ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_DISENGAGE,
      //

      {
			FALSE,														// evade fire
			FALSE,														// follow formation
			GUIDE_POSITION_ABSOLUTE,								// position type
			GUIDE_TERRAIN_FOLLOW_NONE,								// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
		//	ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_DISENGAGE_VIRTUAL,
      //

      {
			FALSE,														// evade fire
			FALSE,														// follow formation
			GUIDE_POSITION_RELATIVE_OBJECT_SPACE,				// position type
			GUIDE_TERRAIN_FOLLOW_NONE,								// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },

      //
		//	ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_EGRESS,
      //

      {
			TRUE,															// evade fire
			FALSE,														// follow formation
			GUIDE_POSITION_ABSOLUTE,								// position type
			GUIDE_TERRAIN_FOLLOW_ATTACK_ALTITUDE,				// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },


      //
		//	ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_HASTY_FIRE,
      //

      {
			TRUE,															// evade fire
			FALSE,														// follow formation
			GUIDE_POSITION_ABSOLUTE,								// position type
			GUIDE_TERRAIN_FOLLOW_NONE,								// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },


      //
		//	ENTITY_SUB_TYPE_GUIDE_ATTACK_AG_HASTY_TAKE_COVER,
      //

      {
			TRUE,															// evade fire
			FALSE,														// follow formation
			GUIDE_POSITION_ABSOLUTE,								// position type
			GUIDE_TERRAIN_FOLLOW_NONE,								// terrain check

			{
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_RADIUS
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_HEADING
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_ALTITUDE
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_TRANSMIT_DATA
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_LAST_TO_REACH
				{ FALSE,		0.0 },									// GUIDE_CRITERIA_WEAPON_VECTOR
			}
      },
	};




