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

#include "entity/mobile/mb_cmbfx.h"

#include "entity/mobile/mb_hostaim.h"

// the classes of an aircraft's critical state that the host answers (mb_hostaim.h) are the values of aircraft_damage_types
typedef char campaign_aircraft_classes_are_aircraft_damage_types[((AIRCRAFT_DAMAGE_NONE == 0) && (AIRCRAFT_DAMAGE_MORTAL == 1) && (AIRCRAFT_DAMAGE_CRITICAL == 2)) ? 1 : -1];

#include "ai/highlevl/highlevl.h"

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define DEBUG_MODULE 				0

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static object_3d_index_numbers
	*destroyed_object_table,
	*restored_object_table;

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

extern int weapon_damage_capability (entity *wpn, entity *target, float damage_modifier, entity_sub_types weapon_type, float velocity);

static float get_player_damage_modifier (entity *en);
static int resolve_player_damage_modifier (entity *en, float *modifier, int *difficulty);

typedef weapon weapon_data_raw;

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/** The damage figures of a weapon, by value (campaign_damage_figures): those of its own definition, or, for a conventional-munitions weapon (which carries its damage in the
 *  munition it scatters, the next entry of the database, the relationship wn_creat.c relies on), those of the munition.  The host applies the arithmetic.
 */
static void fill_damage_figures (campaign_damage_figures *figures, entity_sub_types weapon_type)
{
	const weapon_data
		*row;

	row = &weapon_database[weapon_type];

	figures->weapon_warhead = row->warhead_type;

	if (row->warhead_type == WEAPON_WARHEAD_TYPE_CONVENTIONAL_MUNITIONS)
	{
		if ((weapon_type + 1) >= NUM_ENTITY_SUB_TYPE_WEAPONS)
		{
			debug_fatal ("DAMAGE: conventional munitions weapon %d has no munition entry after it in the database", weapon_type);
		}

		row = &weapon_database[weapon_type + 1];
	}

	figures->warhead = row->warhead_type;
	figures->damage_capability = row->damage_capability;
	figures->armor_penetration_capability = row->armor_penetration_capability;
	figures->burst_duration = row->burst_duration;
}

/** The damage of one hit that reached a damage handler (EEDCS-46).  The physical part is the host's (mb_hostaim.h, DAMAGE): whether the player's invulnerability stops the hit, the
 *  weapon's capability against the target (a real weapon: its one sample of the random stream, which this takes when the host asks, its speed, the armour it meets and the
 *  player's damage setting and the multiplayer multiplier), whether the capability is applied to the health and the health it leaves, and the critical-state classification of the
 *  target before and after.  This keeps what is the campaign's: the sounds a hit makes (they come first, as they did, and so before the host's answer), the settings the host is
 *  given as values, the debug display, the SETTING of the health with its side effects, and what the handlers do with the classes (the credit and the kill).
 *  `critical_before` and `critical_after` (optional) are the classes of the target's critical state (vehicle_critically_damaged and aircraft_critically_damaged returned them); a
 *  hit that was not asked of the host (no weapon, or stopped by invulnerability) changed nothing, and both are 0, so no handler sees a transition.
 */
int damage_client_server_entity_assess (entity *en, entity *weapon, float damage_modifier, int *critical_before, int *critical_after)
{
	campaign_damage_request
		request;

	campaign_aim_result
		answer;

	int
		damage_capability,
		not_ai,
		invulnerable,
		difficulty = 0,
		i,
		temp;

	entity_sub_types
		weapon_type;

	weapon_warhead_types
		warhead_type;

	ASSERT (en);

	CMB_FX_DAMAGE_BEGIN (en, weapon)

	if (critical_before)
	{
		*critical_before = 0;
	}

	if (critical_after)
	{
		*critical_after = 0;
	}

	if (!weapon)
	{
		CMB_FX_DAMAGE_EXIT (EECH_CMB_DAMAGE_EXIT_NO_WEAPON)

		return 0;
	}

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT ((damage_modifier >= 0.0) && (damage_modifier <= 1.0));

	warhead_type = (weapon_warhead_types) get_local_entity_int_value (weapon, INT_TYPE_WEAPON_WARHEAD_TYPE);

	ASSERT ((warhead_type >= WEAPON_WARHEAD_TYPE_NONE) && (warhead_type < NUM_WEAPON_WARHEAD_TYPES));

	not_ai = (get_local_entity_int_value (en, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI);

	if (not_ai)
	{
		sound_sample_indices
			int_sound,
			ext_sound;

		//
		// damage sound effects
		//

		int_sound = SOUND_SAMPLE_INDEX_NONE;
		ext_sound = SOUND_SAMPLE_INDEX_NONE;

		switch (warhead_type)
		{
			case WEAPON_WARHEAD_TYPE_SOLID_SHOT:
			{
				if (get_local_entity_index (weapon) & 1)
				{
					int_sound = SOUND_SAMPLE_INDEX_INTERIOR_LIGHT_DAMAGE2;
					ext_sound = SOUND_SAMPLE_INDEX_EXTERIOR_LIGHT_DAMAGE2;
				}
				else
				{
					int_sound = SOUND_SAMPLE_INDEX_INTERIOR_LIGHT_DAMAGE1;
					ext_sound = SOUND_SAMPLE_INDEX_EXTERIOR_LIGHT_DAMAGE1;
				}

				break;
			}
			case WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK:
			case WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_DUAL_PURPOSE:
			case WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE:
			case WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_AIRCRAFT:
			case WEAPON_WARHEAD_TYPE_CONVENTIONAL_MUNITIONS:
			{
				if (get_local_entity_index (weapon) & 1)
				{
					int_sound = SOUND_SAMPLE_INDEX_INTERIOR_HEAVY_DAMAGE2;
					ext_sound = SOUND_SAMPLE_INDEX_EXTERIOR_HEAVY_DAMAGE2;
				}
				else
				{
					int_sound = SOUND_SAMPLE_INDEX_INTERIOR_HEAVY_DAMAGE1;
					ext_sound = SOUND_SAMPLE_INDEX_EXTERIOR_HEAVY_DAMAGE1;
				}

				break;
			}
		}

		if (int_sound != SOUND_SAMPLE_INDEX_NONE)
		{
			create_client_server_sound_effect_entity
			(
				en,
				ENTITY_SIDE_NEUTRAL,
				ENTITY_SUB_TYPE_EFFECT_SOUND_MISC,
				SOUND_CHANNEL_SOUND_EFFECT,
				SOUND_LOCALITY_INTERIOR,
				NULL,												// position
				damage_modifier,								// amplification
				1.0,												// Werewolf pitch
				TRUE,												// valid sound effect
				FALSE,											// looping
				1,													// sample count
				&int_sound										// sample index list
			);
		}

		if (ext_sound != SOUND_SAMPLE_INDEX_NONE)
		{
			create_client_server_sound_effect_entity
			(
				en,
				ENTITY_SIDE_NEUTRAL,
				ENTITY_SUB_TYPE_EFFECT_SOUND_MISC,
				SOUND_CHANNEL_SOUND_EFFECT,
				SOUND_LOCALITY_EXTERIOR,
				NULL,												// position
				damage_modifier,								// amplification
				1.0,												// Werewolf pitch
				TRUE,												// valid sound effect
				FALSE,											// looping
				1,													// sample count
				&ext_sound										// sample index list
			);
		}
	}

	weapon_type = (entity_sub_types) get_local_entity_int_value (weapon, INT_TYPE_ENTITY_SUB_TYPE);

	memset (&request, 0, sizeof (request));

	fill_damage_figures (&request.figures, weapon_type);

	request.weapon_speed = ((weapon_data_raw *) get_local_entity_data (weapon))->mob.velocity;
	request.weapon_cruise_velocity = weapon_database[weapon_type].cruise_velocity;
	request.damage_modifier = damage_modifier;

	// the armour the weapon meets (get_local_entity_armour_thickness): the launcher's difficulty, the table level and, for a vehicle or an aircraft, the data of the aspect
	request.armour_level = get_local_entity_int_value (en, INT_TYPE_ARMOR_LEVEL);

	{
		entity
			*launcher,
			*pilot;

		launcher = get_local_entity_parent (weapon, LIST_TYPE_LAUNCHED_WEAPON);

		if (launcher && (get_local_entity_int_value (launcher, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI))
		{
			pilot = get_local_entity_first_child (launcher, LIST_TYPE_AIRCREW);

			if (pilot && ((game_difficulty_settings) get_local_entity_int_value (pilot, INT_TYPE_DIFFICULTY_LEVEL) == GAME_DIFFICULTY_EASY))
			{
				request.armour_easy = TRUE;
			}
		}
	}

	if (get_local_entity_int_value (en, INT_TYPE_IDENTIFY_VEHICLE) || get_local_entity_int_value (en, INT_TYPE_IDENTIFY_AIRCRAFT))
	{
		vec3d
			*target_position,
			*weapon_position;

		matrix3x3
			*target_attitude;

		if (get_local_entity_int_value (en, INT_TYPE_IDENTIFY_VEHICLE))
		{
			entity_sub_types
				sub_type;

			sub_type = (entity_sub_types) get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

			request.armour_model = CAMPAIGN_DAMAGE_ARMOUR_VEHICLE;
			request.armour_front = vehicle_database[sub_type].armour_front;
			request.armour_rear = vehicle_database[sub_type].armour_rear;
			request.armour_side = vehicle_database[sub_type].armour_side;
		}
		else
		{
			request.armour_model = CAMPAIGN_DAMAGE_ARMOUR_AIRCRAFT;
		}

		target_position = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);
		weapon_position = get_local_entity_vec3d_ptr (weapon, VEC3D_TYPE_POSITION);
		target_attitude = get_local_entity_attitude_matrix_ptr (en);

		request.target_position[0] = target_position->x;
		request.target_position[1] = target_position->y;
		request.target_position[2] = target_position->z;
		request.weapon_position[0] = weapon_position->x;
		request.weapon_position[1] = weapon_position->y;
		request.weapon_position[2] = weapon_position->z;

		for (i = 0; i < 9; i++)
		{
			request.target_attitude[i] = (*target_attitude)[i / 3][i % 3];
		}
	}

	// the settings the host is given as values: a player's invulnerability, damage setting and the multiplayer multiplier
	invulnerable = not_ai ? get_local_entity_int_value (en, INT_TYPE_INVULNERABLE_FROM_WEAPONS) : FALSE;

	request.target_not_ai = not_ai;
	request.invulnerable = invulnerable ? TRUE : FALSE;

	if (not_ai)
	{
		request.player_modifier_valid = resolve_player_damage_modifier (en, &request.player_modifier, &difficulty);
	}
	else
	{
		request.player_modifier = 1.0;
	}

	if (weapon->weapon_data->launched_weapon_link.parent)
	{
		request.has_launcher = TRUE;
		request.multiplier = comms_weapon_damage_multiplier_value (weapon->weapon_data->launched_weapon_link.parent, FALSE, weapon_type);
	}

	ASSERT (get_session_entity ());

	request.cheat_guard = (command_line_debug_damage_invulnerable && get_local_entity_int_value (get_session_entity (), INT_TYPE_CHEATS_ENABLED)) ? TRUE : FALSE;

	// the health, and how the critical state of the target is classified (a fixed structure has none and a person is never critical)
	request.damage_level = get_local_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL);
	request.initial_damage_level = get_local_entity_int_value (en, INT_TYPE_INITIAL_DAMAGE_LEVEL);

	switch (get_local_entity_type (en))
	{
		case ENTITY_TYPE_PERSON:
			request.critical_model = CAMPAIGN_DAMAGE_CRITICAL_PERSON;
			break;
		case ENTITY_TYPE_ANTI_AIRCRAFT:
		case ENTITY_TYPE_ROUTED_VEHICLE:
		case ENTITY_TYPE_SHIP_VEHICLE:
			request.critical_model = CAMPAIGN_DAMAGE_CRITICAL_VEHICLE;
			break;
		case ENTITY_TYPE_HELICOPTER:
		case ENTITY_TYPE_FIXED_WING:
			request.critical_model = CAMPAIGN_DAMAGE_CRITICAL_AIRCRAFT;
			break;
		default:
			request.critical_model = CAMPAIGN_DAMAGE_CRITICAL_NONE;
	}

	// a hit the player's invulnerability stops never reaches the capability's arithmetic: its hooks begin when the original's did, before the sample is taken
	if (!invulnerable)
	{
		CMB_FX_CAPABILITY_BEGIN ()
	}

	answer = campaign_aim_damage (weapon_type, &request, CMB_FX_DAMAGE_ECHO);

	if (answer.damage.bad_modifier)
	{
		debug_fatal ("DAMAGE: No player damage modifier for difficulty level %d", difficulty);
	}

	if (answer.damage.exited)
	{
		CMB_FX_DAMAGE_EXIT (EECH_CMB_DAMAGE_EXIT_INVULNERABLE)

		return 0;
	}

	CMB_FX_CAPABILITY_DAMAGE (weapon, en, weapon_type, answer)

	damage_capability = answer.damage.capability;

	if (command_line_debug_show_damage) // draw damage value
	{
		viewpoint
			temp_vp;
		vec3d
			temp_vec;
		int
			digit,
			zero = TRUE,
			p = 1000;

		get_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &temp_vp.position);
		get_local_entity_attitude_matrix (weapon, temp_vp.attitude);
		temp_vec.x = 4;
		temp_vec.y = temp_vec.z = 0;
		multiply_matrix3x3_vec3d(&temp_vec, temp_vp.attitude, &temp_vec);
		temp_vp.position.x -= 2 * temp_vec.x;
		temp_vp.position.y -= 2 * temp_vec.y;
		temp_vp.position.z -= 2 * temp_vec.z;

		temp = damage_capability;

		for (i = 3; i >= 0; i-- )
		{
			digit = temp / p;
			if (digit)
				zero = FALSE;

			if (!zero)
				create_debug_3d_object (&temp_vp, OBJECT_3D_DIG_0 + digit, 2, 5.0);

			temp %= p;
			p /= 10;
			temp_vp.position.x += temp_vec.x;
			temp_vp.position.y += temp_vec.y;
			temp_vp.position.z += temp_vec.z;
		}
	}

	if (answer.damage.written)
	{
		set_client_server_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL, answer.damage.new_level);

		CMB_FX_DAMAGE_WRITTEN (en, answer.damage.new_level)

		#if DEBUG_MODULE

		debug_log ("DAMAGE: %s hit = %d , new damage level = %d", get_local_entity_string (en, STRING_TYPE_FULL_NAME), damage_capability, answer.damage.new_level);

		#endif
	}

	CMB_FX_DAMAGE_END (en, weapon, damage_modifier, damage_capability)

	if (critical_before)
	{
		*critical_before = answer.damage.critical_before;
	}

	if (critical_after)
	{
		*critical_after = answer.damage.critical_after;
	}

	return damage_capability;
}

int damage_client_server_entity (entity *en, entity *weapon, float damage_modifier)
{
	return damage_client_server_entity_assess (en, weapon, damage_modifier, NULL, NULL);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void display_debug_kill_info (entity *victim, entity *aggressor)
{

	entity
		*victim_task,
		*aggressor_task;

	const char
		*desc1,
		*desc2,
		*victim_desc,
		*aggressor_desc;

	debug_colours
		col;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	//
	// get task descriptions
	//

	// victim

	victim_task = get_local_entity_primary_task (victim);

	if (victim_task)
	{
		desc1 = get_local_entity_string (victim_task, STRING_TYPE_FULL_NAME);
	}
	else
	{
		desc1 = task_database [ENTITY_SUB_TYPE_TASK_NOTHING].full_name;
	}

	// aggressor

	aggressor_task = get_local_entity_primary_task (aggressor);

	if (aggressor_task)
	{
		desc2 = get_local_entity_string (aggressor_task, STRING_TYPE_FULL_NAME);
	}
	else
	{
		desc2 = task_database [ENTITY_SUB_TYPE_TASK_NOTHING].full_name;
	}

	//
	// get mobile descriptions
	//

	// victim

	victim_desc = get_local_entity_string (victim, STRING_TYPE_FULL_NAME);

	ASSERT (victim_desc);

	// aggressor

	aggressor_desc = get_local_entity_string (aggressor, STRING_TYPE_FULL_NAME);

	ASSERT (aggressor_desc);

	//

	if (get_local_entity_int_value (aggressor, INT_TYPE_SIDE) == ENTITY_SIDE_BLUE_FORCE)
	{
		col = DEBUG_COLOUR_BLUE;
	}
	else
	{
		col = DEBUG_COLOUR_RED;
	}

	debug_colour_log (col, "DAMAGE: %s (%d) doing %s - killed by %s (%d) doing %s",
								victim_desc,
								get_local_entity_index (victim),
								desc1,
								aggressor_desc,
								get_local_entity_index (aggressor),
								desc2);

	ai_log ("(KILL) %s doing %s - killed by %s doing %s", victim_desc, desc1, aggressor_desc, desc2);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/** The damage a weapon is CAPABLE of against a target, with no weapon in flight (EEDCS-46): the forecast the target selection (wn_tgt.c) makes.  The physical arithmetic is the
 *  host's (mb_hostaim.h, FORECAST); this gathers what the original read and sends it by value, and hands back the host's answer.  It takes no sample of the random stream, as the
 *  original took none.  Kept here: which weapon and which target (the caller's), the armour level and the player settings read from the target, and the database relationship
 *  that a conventional-munitions weapon carries its damage in the next database entry (the munition it scatters, wn_creat.c).
 */
static int weapon_damage_forecast (entity *target, float damage_modifier, entity_sub_types weapon_type, float velocity)
{
	campaign_damage_figures
		figures;

	campaign_aim_result
		result;

	int
		not_ai;

	float
		player_modifier = 1.0;

	ASSERT (target);

	fill_damage_figures (&figures, weapon_type);

	not_ai = (get_local_entity_int_value (target, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI);

	if (not_ai)
	{
		player_modifier = get_player_damage_modifier (target);
	}

	result = campaign_aim_forecast (weapon_type, &figures, get_local_entity_int_value (target, INT_TYPE_ARMOR_LEVEL), not_ai, player_modifier, damage_modifier, velocity, CMB_FX_FORECAST_ECHO);

	CMB_FX_CAPABILITY_FORECAST (target, weapon_type, result)

	return result.forecast_capability;
}

int weapon_damage_capability (entity *wpn, entity *target, float damage_modifier, entity_sub_types weapon_type, float velocity)
{
	ASSERT (target);

	// the capability of a REAL weapon is part of the host's answer to a hit (damage_client_server_entity); only the forecast, with no weapon in flight, is asked on its own
	if (wpn)
	{
		debug_fatal ("DAMAGE: the capability of a real weapon is asked of the host with the hit it belongs to, not on its own");
	}

	return weapon_damage_forecast (target, damage_modifier, weapon_type, velocity);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define DESTROYED_OBJECT_KEYWORD "_DESTROYED"

#define DEBUG_NON_ENTRIES	0

void initialise_destroyed_object_database (void)
{
	object_3d_index_numbers
		normal_object,
		destroyed_object;

	const char
		*normal_object_string;
	char
		*destroyed_object_string;

	int
		found_flag,
		keyword_length;

	keyword_length = strlen (DESTROYED_OBJECT_KEYWORD);

	destroyed_object_table = (object_3d_index_numbers *) safe_malloc (sizeof (object_3d_index_numbers) * OBJECT_3D_LAST);

	restored_object_table = (object_3d_index_numbers *) safe_malloc (sizeof (object_3d_index_numbers) * OBJECT_3D_LAST);

	//
	// set defaults
	//

	for (normal_object = 0; normal_object < OBJECT_3D_LAST; normal_object ++)
	{
		destroyed_object_table [normal_object] = normal_object;

		restored_object_table [normal_object] = normal_object;
	}

	//
	// search every 3d object for a destroyed counterpart
	//

	for (normal_object = 0; normal_object < OBJECT_3D_LAST; normal_object ++)
	{
		//
		// construct search string
		//

		normal_object_string = object_3d_information_database [normal_object].name;

		destroyed_object_string = (char *) malloc_fast_mem (strlen (normal_object_string) + keyword_length + 4);

		strcpy (destroyed_object_string, normal_object_string);

		strcat (destroyed_object_string, DESTROYED_OBJECT_KEYWORD);

		//
		// search database for new string
		//

		found_flag = FALSE;

		for (destroyed_object = 0; destroyed_object < OBJECT_3D_LAST; destroyed_object ++)
		{
			if (strcmp (destroyed_object_string, object_3d_information_database [destroyed_object].name) == 0)
			{
				if (objects_3d_scene_database[destroyed_object].succeeded)
				{
					destroyed_object_table [normal_object] = destroyed_object;

					restored_object_table [destroyed_object] = normal_object;

					found_flag = TRUE;
				}
				else
				{
					debug_log("DAMAGE : Failed to load destroyed scene for %s", normal_object_string);
				}

				break;
			}
		}

		#if DEBUG_NON_ENTRIES

		if ((!found_flag) && (!strstr (normal_object_string, DESTROYED_OBJECT_KEYWORD)))
		{
			debug_log ("DAMAGE : No entry found for %s", normal_object_string);
		}

		#endif

		free_mem (destroyed_object_string);
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void deinitialise_destroyed_object_database (void)
{
	safe_free (destroyed_object_table);

	safe_free (restored_object_table);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

object_3d_index_numbers get_3d_object_destroyed_object_index (object_3d_index_numbers index)
{
	ASSERT ((index >= 0) && (index < OBJECT_3D_LAST));

	return destroyed_object_table [index];
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

object_3d_index_numbers get_3d_object_restored_object_index (object_3d_index_numbers index)
{
	ASSERT ((index >= 0) && (index < OBJECT_3D_LAST));

	return restored_object_table [index];
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static struct PLAYER_DAMAGE_MODIFIER_TABLE
{
	game_difficulty_settings
		difficulty;

	float
		damage_modifier;

} player_damage_modifier_table [] =
	{
		GAME_DIFFICULTY_EASY,		0.33,
		GAME_DIFFICULTY_MEDIUM,		0.66,
		GAME_DIFFICULTY_HARD,		1.0,
		(game_difficulty_settings) -1
	};

/** The player's damage setting of a target, without the fatal error: TRUE and the setting, or FALSE and the difficulty that has none.  A target with no pilot has the setting 1. */
static int resolve_player_damage_modifier (entity *en, float *modifier, int *difficulty_out)
{
	int
		loop;

	entity
		*pilot_en;

	game_difficulty_settings
		difficulty;

	ASSERT (en);

	*modifier = 1.0;

	pilot_en = get_local_entity_first_child (en, LIST_TYPE_AIRCREW);

	if (pilot_en)
	{
		difficulty = (game_difficulty_settings) get_local_entity_int_value (pilot_en, INT_TYPE_DIFFICULTY_LEVEL);

		*difficulty_out = difficulty;

		loop = 0;

		while (player_damage_modifier_table [loop].difficulty >= 0)
		{
			if (player_damage_modifier_table [loop].difficulty == difficulty)
			{
				*modifier = player_damage_modifier_table [loop].damage_modifier;

				return TRUE;
			}

			loop ++;
		}

		return FALSE;
	}

	return TRUE;
}

float get_player_damage_modifier (entity *en)
{
	float
		modifier;

	int
		difficulty = 0;

	if (!resolve_player_damage_modifier (en, &modifier, &difficulty))
	{
		debug_fatal ("DAMAGE: No player damage modifier for difficulty level %d", difficulty);
	}

	return modifier;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

int get_damage_debug_invulnerable_flag (void)
{
	return command_line_debug_damage_invulnerable;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

int toggle_damage_debug_invulnerable_flag (void)
{
	command_line_debug_damage_invulnerable = !command_line_debug_damage_invulnerable;

	return command_line_debug_damage_invulnerable;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
