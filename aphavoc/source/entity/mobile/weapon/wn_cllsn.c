// 
// 	 Enemy Engaged RAH-66 Comanche Versus KA-52 Hokum
// 	 Copyright (C) 2000 Empire Interactive (Europe) Ltd,
// 	 677 High Road, North Finchley, London N12 0DA
// 
// 	 Please see the document LICENSE.TXT for the full licence agreement
// 
// 2. LICENCE
//  2.1 	
//  	Subject to the provisions of this Agreement we now grant to you the 
//  	following rights in respect of the Source Code:
//   2.1.1 
//   	the non-exclusive right to Exploit  the Source Code and Executable 
//   	Code on any medium; and 
//   2.1.2 
//   	the non-exclusive right to create and distribute Derivative Works.
//  2.2 	
//  	Subject to the provisions of this Agreement we now grant you the
// 	following rights in respect of the Object Code:
//   2.2.1 
// 	the non-exclusive right to Exploit the Object Code on the same
// 	terms and conditions set out in clause 3, provided that any
// 	distribution is done so on the terms of this Agreement and is
// 	accompanied by the Source Code and Executable Code (as
// 	applicable).
// 
// 3. GENERAL OBLIGATIONS
//  3.1 
//  	In consideration of the licence granted in clause 2.1 you now agree:
//   3.1.1 
// 	that when you distribute the Source Code or Executable Code or
// 	any Derivative Works to Recipients you will also include the
// 	terms of this Agreement;
//   3.1.2 
// 	that when you make the Source Code, Executable Code or any
// 	Derivative Works ("Materials") available to download, you will
// 	ensure that Recipients must accept the terms of this Agreement
// 	before being allowed to download such Materials;
//   3.1.3 
// 	that by Exploiting the Source Code or Executable Code you may
// 	not impose any further restrictions on a Recipient's subsequent
// 	Exploitation of the Source Code or Executable Code other than
// 	those contained in the terms and conditions of this Agreement;
//   3.1.4 
// 	not (and not to allow any third party) to profit or make any
// 	charge for the Source Code, or Executable Code, any
// 	Exploitation of the Source Code or Executable Code, or for any
// 	Derivative Works;
//   3.1.5 
// 	not to place any restrictions on the operability of the Source 
// 	Code;
//   3.1.6 
// 	to attach prominent notices to any Derivative Works stating
// 	that you have changed the Source Code or Executable Code and to
// 	include the details anddate of such change; and
//   3.1.7 
//   	not to Exploit the Source Code or Executable Code otherwise than
// 	as expressly permitted by  this Agreement.
// 



/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#include "project.h"
#include "entity/mobile/mb_wpnfx.h"

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define DEBUG_MODULE	0

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define COLLISION_THRESHOLD	(5.0)

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

float get_local_entity_armour_thickness (entity *target, entity *weapon)
{
	float
		armor_thickness,
		result = 0;

	entity_sub_types
		sub_type;

	ASSERT (target);

	ASSERT (weapon);

	ASSERT (get_local_entity_int_value (weapon, INT_TYPE_IDENTIFY_WEAPON));

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	// get game difficulty level (use player's own level)

	{
		game_difficulty_settings
			difficulty;

		entity
			*launcher,
			*pilot;

		launcher = get_local_entity_parent (weapon, LIST_TYPE_LAUNCHED_WEAPON);

		if (launcher)
		{
			if (get_local_entity_int_value (launcher, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
			{
				pilot = get_local_entity_first_child (launcher, LIST_TYPE_AIRCREW);

				if (pilot)
				{
					difficulty = (game_difficulty_settings) get_local_entity_int_value (pilot, INT_TYPE_DIFFICULTY_LEVEL);

					if (difficulty == GAME_DIFFICULTY_EASY)
					{
						return (0.1);
					}
				}
			}
		}
	}

	// get armour thickness

	armor_thickness = (float) get_local_entity_int_value (target, INT_TYPE_ARMOR_LEVEL);

	ASSERT(armor_thickness);
	
	if (get_local_entity_int_value (target, INT_TYPE_IDENTIFY_VEHICLE) ||
			get_local_entity_int_value (target, INT_TYPE_IDENTIFY_AIRCRAFT))
	{
		float
			front_armor,
			rear_armor,
			side_armor,
			top_armor,
			norm;
		vec3d
			impact_vector,
			*target_position,
			*weapon_position;
		matrix3x3
			*target_attitude;

		
		if (get_local_entity_int_value (target, INT_TYPE_IDENTIFY_VEHICLE))
		{
			sub_type = get_local_entity_int_value (target, INT_TYPE_ENTITY_SUB_TYPE);

			front_armor = vehicle_database[sub_type].armour_front;
			rear_armor = vehicle_database[sub_type].armour_rear;
			side_armor = vehicle_database[sub_type].armour_side;
			top_armor = (rear_armor + side_armor) / 3.0;
		}
		else
		{
			front_armor = 1;
			rear_armor = 0.25;
			side_armor = 0.75;
			top_armor = 0.33;
		}
		
		target_position = get_local_entity_vec3d_ptr (target, VEC3D_TYPE_POSITION);
		weapon_position = get_local_entity_vec3d_ptr (weapon, VEC3D_TYPE_POSITION);
		target_attitude = get_local_entity_attitude_matrix_ptr (target);
		
		impact_vector.x = weapon_position->x - target_position->x;
		impact_vector.y = weapon_position->y - target_position->y;
		impact_vector.z = weapon_position->z - target_position->z;
		
		multiply_matrix3x3_vec3d(&impact_vector, *target_attitude, &impact_vector);
		
		norm = fabs(impact_vector.x) + fabs(impact_vector.y) + fabs(impact_vector.z);
		
		impact_vector.x /= norm;
		impact_vector.y /= norm;
		impact_vector.z /= norm;

		if (impact_vector.x >= 0)
			result += side_armor * impact_vector.x;
		else
			result -= side_armor * impact_vector.x;
		
		if (impact_vector.y >= 0)
			result += top_armor * impact_vector.y;
		else
			result -= top_armor * impact_vector.y;
		
		if (impact_vector.z >= 0)
			result += front_armor * impact_vector.z;
		else
			result -= rear_armor * impact_vector.z;
		
		ASSERT(result);
		
		armor_thickness *= result;
		
		#if DEBUG_MODULE
		debug_log("WN_COLLSN: armor: side %f, top %f, front %f, result %f, armour_thickness %f", impact_vector.x, impact_vector.y, impact_vector.z, result, armor_thickness);
		#endif
	}
	
//	{
//		float
//			target_zvx,
//			target_zvz,
//			weapon_zvx,
//			weapon_zvz,
//			cos_theta,
//			theta;
//
//		matrix3x3
//			*target_attitude,
//			*weapon_attitude;
//
//		target_attitude = get_local_entity_attitude_matrix_ptr (target);
//		weapon_attitude = get_local_entity_attitude_matrix_ptr (weapon);
//
//		target_zvx = (*target_attitude)[2][0];
//		target_zvz = (*target_attitude)[2][2];
//		weapon_zvx = -(*weapon_attitude)[2][0];
//		weapon_zvz = -(*weapon_attitude)[2][2];
//
//		cos_theta = (target_zvx * weapon_zvx) + (target_zvz * weapon_zvz);
//
//		cos_theta = bound (cos_theta, -1.0, 1.0);
//
//		theta = acos (cos_theta);
//
//		if (theta <= rad (45.0))
//		{
//			armour_thickness *= vehicle_database[sub_type].armour_front;
//		}
//		else if (theta <= rad (135.0))
//		{
//			armour_thickness *= vehicle_database[sub_type].armour_side;
//		}
//		else
//		{
//			armour_thickness *= vehicle_database[sub_type].armour_rear;
//		}
//	}

	return (armor_thickness);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
