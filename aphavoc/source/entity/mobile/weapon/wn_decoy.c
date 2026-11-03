

































































#include "project.h"





#define DEBUG_MODULE 0





float get_decoy_timer_start_value (weapon_decoy_types type)
{
	float
		timer;

	switch (type)
	{

		case WEAPON_DECOY_TYPE_NONE:

		{
			timer = 0.0;

			break;
		}

		case WEAPON_DECOY_TYPE_CHAFF:

		{
			timer = 0.1;

			break;
		}

		case WEAPON_DECOY_TYPE_FLARE:

		{
			timer = 0.1;

			break;
		}

		case WEAPON_DECOY_TYPE_SMOKE_GRENADE:

		{
			timer = 3.0;

			break;
		}

		default:

		{
			debug_fatal ("Invalid weapon decoy type = %d", type);

			break;
		}
	}

	return (timer);
}





float get_decoy_timer_restart_value (weapon_decoy_types type)
{
	float
		timer;

	switch (type)
	{

		case WEAPON_DECOY_TYPE_NONE:

		{
			timer = 0.0;

			break;
		}

		case WEAPON_DECOY_TYPE_CHAFF:

		{
			timer = 0.4 + (frand1 () * 0.2);

			break;
		}

		case WEAPON_DECOY_TYPE_FLARE:

		{
			timer = 0.4 + (frand1 () * 0.2);

			break;
		}

		case WEAPON_DECOY_TYPE_SMOKE_GRENADE:

		{
			timer = 1.0 + (frand1 () * 0.5);

			break;
		}

		default:

		{
			debug_fatal ("Invalid weapon decoy type = %d", type);

			break;
		}
	}

	return (timer);
}





int get_weapon_locked_onto_decoy (weapon_decoy_types type, entity *sender)
{
	int
		locked;
	float coefficient = 1.0;
	game_difficulty_settings difficulty;

	entity
		*launcher,
		*pilot;

	// get game difficulty level (use player's own level)

	launcher = get_local_entity_parent (sender, LIST_TYPE_LAUNCHED_WEAPON);

	if (launcher)
	{
		if (get_local_entity_int_value (launcher, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
		{
			pilot = get_local_entity_first_child (launcher, LIST_TYPE_AIRCREW);

			if (pilot)
			{
				difficulty = (game_difficulty_settings) get_local_entity_int_value (pilot, INT_TYPE_DIFFICULTY_LEVEL);

				if (difficulty == GAME_DIFFICULTY_EASY) {
					coefficient = 0.1;
				}
				else if (difficulty == GAME_DIFFICULTY_MEDIUM) {
					coefficient = 0.5;
				}
			}
		}
	}

	switch (type)
	{

		case WEAPON_DECOY_TYPE_NONE:

		{
			locked = FALSE;

			break;
		}

		case WEAPON_DECOY_TYPE_CHAFF:

		{
			locked = frand1 () < (pow(0.2, coefficient) * command_line_chaff_effectiveness);

			break;
		}

		case WEAPON_DECOY_TYPE_FLARE:

		{
			locked = frand1 () < (pow(0.2, coefficient) * command_line_flare_effectiveness);

			break;
		}

		case WEAPON_DECOY_TYPE_SMOKE_GRENADE:

		{
			locked = frand1 () < (pow(0.2, coefficient) * command_line_smoke_effectiveness);

			break;
		}

		default:

		{
			debug_fatal ("Invalid weapon decoy type = %d", type);

			break;
		}
	}

	return (locked);
}





void update_decoy (entity *en)
{
	weapon
		*raw;

	entity
		*launcher,
		*locked_on_weapon,
		*succ;

	ASSERT (en);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	raw = (weapon *) get_local_entity_data (en);

	ASSERT (weapon_database[raw->mob.sub_type].decoy_type != WEAPON_DECOY_TYPE_NONE);

	raw->decoy_timer -= get_delta_time ();

	if (raw->decoy_timer < 0.0)
	{
		raw->decoy_timer = get_decoy_timer_restart_value (weapon_database[raw->mob.sub_type].decoy_type);

		launcher = get_local_entity_parent (en, LIST_TYPE_LAUNCHED_WEAPON);

		if (launcher)
		{
			locked_on_weapon = get_local_entity_first_child (launcher, LIST_TYPE_TARGET);

			while (locked_on_weapon)
			{
				succ = get_local_entity_child_succ (locked_on_weapon, LIST_TYPE_TARGET);

				notify_local_entity (ENTITY_MESSAGE_DECOY_ACTIVE, locked_on_weapon, en);

				locked_on_weapon = succ;
			}
		}
	}
}





void get_smoke_grenade_distract_position (entity *weapon, entity *grenade, vec3d *distract_position)
{
	int
		seed;

	vec3d
		grenade_position;

	ASSERT (weapon);

	ASSERT (grenade);

	ASSERT (distract_position);

	seed = get_client_server_entity_random_number_seed (weapon);

	get_local_entity_vec3d (grenade, VEC3D_TYPE_POSITION, &grenade_position);

	*distract_position = grenade_position;

	distract_position->x += sfrand1x (&seed) * 50.0;
	distract_position->z += sfrand1x (&seed) * 50.0;

	if (point_inside_map_area (distract_position))
	{
		distract_position->y = get_3d_terrain_elevation (distract_position->x, distract_position->z);
	}
	else
	{
		*distract_position = grenade_position;
	}
}





void get_jammer_distract_position (entity *weapon, vec3d *target_position)
{
	int
		seed;

	ASSERT (weapon);

	ASSERT (target_position);

	seed = get_client_server_entity_random_number_seed (weapon);

	if (frand1x (&seed) > 0.5)
	{
		target_position->x += sfrand1x (&seed) * 35.0;
		target_position->y += sfrand1x (&seed) * 35.0;
		target_position->z += sfrand1x (&seed) * 35.0;
	}
}





entity_sub_types get_decoy_type_for_weapon (entity *en)
{
	weapon
		*raw;

	entity_sub_types
		weapon_sub_type;

	ASSERT (en);

	raw = (weapon *) get_local_entity_data (en);

	switch (weapon_database[raw->mob.sub_type].guidance_type)
	{

		case WEAPON_GUIDANCE_TYPE_NONE:

		{
			weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON;

			break;
		}

		case WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_RADAR:
		case WEAPON_GUIDANCE_TYPE_ACTIVE_RADAR:
		case WEAPON_GUIDANCE_TYPE_RADIO_COMMAND:

		{
			weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_CHAFF;

			break;
		}

		case WEAPON_GUIDANCE_TYPE_PASSIVE_INFRA_RED:

		{
			weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_FLARE;

			break;
		}

		case WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_LASER:
		case WEAPON_GUIDANCE_TYPE_SEMI_ACTIVE_LASER_BEAM_RIDING:
		case WEAPON_GUIDANCE_TYPE_ACTIVE_LASER:
		case WEAPON_GUIDANCE_TYPE_WIRE_GUIDED:

		{
			weapon_sub_type = ENTITY_SUB_TYPE_WEAPON_SMOKE_GRENADE;

			break;
		}

		default:

		{
			debug_fatal ("Invalid weapon guidance type = %d", weapon_database[raw->mob.sub_type].guidance_type);

			break;
		}
	}

	return (weapon_sub_type);
}




