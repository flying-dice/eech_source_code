

































































#include "project.h"

#include "ai/highlevl/highlevl.h"





#define DEBUG_MODULE 				0





static object_3d_index_numbers
	*destroyed_object_table,
	*restored_object_table;





extern int weapon_damage_capability (entity *wpn, entity *target, float damage_modifier, entity_sub_types weapon_type, float velocity);

static float get_player_damage_modifier (entity *en);





int damage_client_server_entity (entity *en, entity *weapon, float damage_modifier)
{
	int
		damage_capability,
		damage_level;

	weapon_warhead_types
		warhead_type;

	ASSERT (en);

	if (!weapon)
	{
		return 0;
	}

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT ((damage_modifier >= 0.0) && (damage_modifier <= 1.0));

	warhead_type = (weapon_warhead_types) get_local_entity_int_value (weapon, INT_TYPE_WEAPON_WARHEAD_TYPE);

	ASSERT ((warhead_type >= WEAPON_WARHEAD_TYPE_NONE) && (warhead_type < NUM_WEAPON_WARHEAD_TYPES));
























































































		}

		//
		// invulnerability option for players
		//

		if (get_local_entity_int_value (en, INT_TYPE_INVULNERABLE_FROM_WEAPONS))
		{
			return 0;
		}
	}

	damage_level = get_local_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL);

	damage_capability = weapon_damage_capability (weapon, en, damage_modifier, get_local_entity_int_value (weapon, INT_TYPE_ENTITY_SUB_TYPE), FALSE);

	ASSERT (get_session_entity ());

	if (damage_capability > 0 && (!command_line_debug_damage_invulnerable || !get_local_entity_int_value (get_session_entity (), INT_TYPE_CHEATS_ENABLED)))
	{
		damage_level -= damage_capability;

		damage_level = max (damage_level, 0);

		set_client_server_entity_int_value (en, INT_TYPE_DAMAGE_LEVEL, damage_level);






	}

	return damage_capability;
}































































































int weapon_damage_capability (entity *wpn, entity *target, float damage_modifier, entity_sub_types weapon_type, float velocity)
{
	int
		result = 0,
		i,
		temp,
		high_explosive = FALSE,
		warhead_type;

	float
		weapon_armor_penetration_capability,
		random_modifier = 1,
		target_armor_thickness,
		armor_modifier,
		damage_capability,
		armor_penetration_capability;
	weapon
		*raw;

	ASSERT (target);

	switch (weapon_database[weapon_type].warhead_type)
	{
		case WEAPON_WARHEAD_TYPE_CONVENTIONAL_MUNITIONS:
			warhead_type = weapon_database[weapon_type + 1].warhead_type;
			damage_capability = weapon_database[weapon_type + 1].damage_capability * weapon_database[weapon_type + 1].burst_duration / 3;
			armor_penetration_capability = weapon_database[weapon_type + 1].armor_penetration_capability;
			break;
		default:
			warhead_type = weapon_database[weapon_type].warhead_type;
			damage_capability = weapon_database[weapon_type].damage_capability;
			armor_penetration_capability = weapon_database[weapon_type].armor_penetration_capability;
	}
	
	if (wpn) // only for real weapon!
	{
		raw = (weapon *) get_local_entity_data (wpn);

		target_armor_thickness = get_local_entity_armour_thickness (target, wpn);
		random_modifier = frand1();

		if (warhead_type >= WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK && 
				warhead_type <= WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_AIRCRAFT)
		{
			velocity = 1;	
			high_explosive = TRUE;
			random_modifier = 0.75 + 2 * random_modifier * random_modifier * random_modifier; // random chance from 75% to 275%, average 100%
		}
		else
		{
			velocity = raw->mob.velocity / weapon_database[weapon_type].cruise_velocity;
			random_modifier = 0.75 + 0.5 * random_modifier; // random chance from 75% to 125%, average 100%
		}

	}
	else
	{
		target_armor_thickness = (float) get_local_entity_int_value (target, INT_TYPE_ARMOR_LEVEL);
		if (warhead_type >= WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK && 
				warhead_type <= WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_AIRCRAFT)
		{
			velocity = 1;	
			high_explosive = TRUE;
		}
	}
	
	weapon_armor_penetration_capability = armor_penetration_capability * random_modifier * velocity;
	
	if (weapon_armor_penetration_capability >= target_armor_thickness && damage_modifier < 0.01) // weapon pierces through the armor
	{
		result = (int) (damage_capability * random_modifier * velocity);





		
	}
	else if (high_explosive)
	{
		float he_warheads = WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_AIRCRAFT - WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK; // subtract 1
		
		damage_modifier = 1 - pow(damage_modifier, pow((float)(warhead_type - WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK + 1) / he_warheads, 1.5f));
		armor_modifier =  min(1.0f, damage_capability * damage_modifier * random_modifier / pow(target_armor_thickness, 1.5f));
		
		result = (int) (damage_capability * damage_modifier * armor_modifier * random_modifier);






	}
	
	if (get_local_entity_int_value (target, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
		result *= get_player_damage_modifier (target);
	











































	return result;
}





#define DESTROYED_OBJECT_KEYWORD "_DESTROYED"


































































































































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

float get_player_damage_modifier (entity *en)
{
	int
		loop;

	entity
		*pilot_en;

	game_difficulty_settings
		difficulty;

	ASSERT (en);

	pilot_en = get_local_entity_first_child (en, LIST_TYPE_AIRCREW);

	if (pilot_en)
	{
		difficulty = (game_difficulty_settings) get_local_entity_int_value (pilot_en, INT_TYPE_DIFFICULTY_LEVEL);
	
		loop = 0;
	
		while (player_damage_modifier_table [loop].difficulty >= 0)
		{
			if (player_damage_modifier_table [loop].difficulty == difficulty)
			{
				return player_damage_modifier_table [loop].damage_modifier;
			}
	
			loop ++;
		}
	
		debug_fatal ("DAMAGE: No player damage modifier for difficulty level %d", difficulty);
	}
	
	return 1.0;
}
























