

































































#include "project.h"

#define DEBUG_MODULE 0




static int
	num_vehicle_hit_comments [NUM_ENTITY_SIDES] = {0, 4, 4},
	last_vehicle_hit_comment [NUM_ENTITY_SIDES] = {0, 0, 0};

static speech_instance_type
	vehicle_hit_comments [] =
	{
		{ SPEECH_GC_REQUEST_SUPPORT_AT,		SPEECH_ARRAY_GC_MESSAGES,		100,	SPEECH_INTRODUCTION_ALWAYS },
		{ SPEECH_GC_PINNED_DOWN_AT,			SPEECH_ARRAY_GC_MESSAGES,		100,	SPEECH_INTRODUCTION_ALWAYS },
		{ SPEECH_GC_NEED_AIR_SUPPORT_AT,		SPEECH_ARRAY_GC_MESSAGES,		100,	SPEECH_INTRODUCTION_ALWAYS },
		{ SPEECH_GC_UNDER_HEAVY_FIRE,			SPEECH_ARRAY_GC_MESSAGES,		100,	SPEECH_INTRODUCTION_ALWAYS },
	};



void play_vehicle_shot_at_speech (entity *victim, entity *aggressor)
{
	entity_sides
		victim_side,
		aggressor_side;

	int
		val,
		speech_index;

	vec3d
		*pos;

	ASSERT (victim);

	ASSERT (aggressor);

	pos = get_local_entity_vec3d_ptr (victim, VEC3D_TYPE_POSITION);

	ASSERT (pos);

	victim_side = (entity_sides) get_local_entity_int_value (victim, INT_TYPE_SIDE);

	aggressor_side = (entity_sides) get_local_entity_int_value (aggressor, INT_TYPE_SIDE);

	if (aggressor_side == victim_side)
	{
		//
		// FRIENDLY FIRE
		//


		//
		// victims speech
		//

		

		//
		// Aggressors speech
		//


		if (get_local_entity_int_value (aggressor, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
		{
			//
			// PLAY_SPEECH (CPG)
			//

			if (rand16 () & 1)
			{
				speech_index = SPEECH_CPG_FRIENDLY_FIRE1;
			}
			else
			{
				speech_index = SPEECH_CPG_FRIENDLY_FIRE2;
			}

			play_client_server_cpg_message
			(
				aggressor,
				1.0,
				3.0,
				SPEECH_CATEGORY_ATTACKING_TARGETS,
				12.0,
				speech_index
			);
		}
	}
	else
	{

		//
		// victims speech
		//


		val = get_speech_random_value
				(
					vehicle_hit_comments,
					num_vehicle_hit_comments [victim_side],
					last_vehicle_hit_comment [victim_side]
				);

		speech_index = vehicle_hit_comments [val].speech_index;
	
		get_speech_sector_coordinates (pos);

		// PLAY_SPEECH (GC)
		//
		// [GC Introduction],
		// (We are pinned down by enemy fire) | (we are taking heavy fire, requesting ariel assistance)
		// location

		play_client_server_speech
		(
			get_session_entity (),
			victim,
			victim_side,
			ENTITY_SUB_TYPE_EFFECT_SOUND_RADIO_MESSAGE,
			SOUND_LOCALITY_RADIO,
			0.0,
			1.0,
			45.0,
			SPEECH_ORIGINATOR_GROUND_CONTROLLER,
			SPEECH_CATEGORY_UNDER_ATTACK,
			300.0,
			SPEECH_ARRAY_GC_MESSAGES, SPEECH_GC_INTRODUCTION,
			SPEECH_ARRAY_GC_MESSAGES, speech_index,
			SPEECH_ARRAY_NUMBERS, speech_sector_coordinates [0],
			SPEECH_ARRAY_NUMBERS, speech_sector_coordinates [1],
			SPEECH_ARRAY_NUMBERS, speech_sector_coordinates [2],
			SPEECH_ARRAY_NUMBERS, speech_sector_coordinates [3],
			SPEECH_ARRAY_NUMBERS, speech_sector_coordinates [4],
			SPEECH_ARRAY_NUMBERS, speech_sector_coordinates [5],
			-1
		);


		//
		// Aggressors speech
		//


		if (get_local_entity_int_value (aggressor, INT_TYPE_IDENTIFY_AIRCRAFT))
		{
			if (get_local_entity_int_value (aggressor, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
			{
				//
				// PLAY_SPEECH (CPG)
				//

				play_client_server_cpg_message
				(
					aggressor,
					0.4,
					3.0,
					SPEECH_CATEGORY_ATTACKING_TARGETS,
					12.0,
					SPEECH_CPG_TARGET_HIT
				);
			}
			else
			{
				//
				// PLAY_SPEECH (WINGMAN)
				//

				play_client_server_wingman_message
				(
					aggressor,
					0.4,
					3.0,
					SPEECH_CATEGORY_ATTACKING_TARGETS, 12.0,
					SPEECH_INTRODUCTION_SOMETIMES,
					SPEECH_ARRAY_WINGMAN_MESSAGES, SPEECH_WINGMAN_TARGET_HIT
				);
			}
		}
	}
}





static int
	num_vh_destroyed_wingman_comments [NUM_ENTITY_SIDES] = {0, 1, 1},
	last_vh_destroyed_wingman_comment [NUM_ENTITY_SIDES] = {0, 0, 0};

static speech_instance_type
	vehicle_destroyed_wingman_comments [] =
	{
		{ SPEECH_WINGMAN_TARGET_DESTROYED,	SPEECH_ARRAY_WINGMAN_MESSAGES,	100, 	SPEECH_INTRODUCTION_SOMETIMES },
	};



static int
	num_vh_destroyed_cpg_comments [NUM_ENTITY_SIDES] = {0, 3, 3},
	last_vh_destroyed_cpg_comment [NUM_ENTITY_SIDES] = {0, 0, 0};

static speech_instance_type
	vehicle_destroyed_cpg_comments [] =
	{
		{ SPEECH_CPG_TARGET_DESTROYED,					SPEECH_ARRAY_CPG_MESSAGES,			100, 	SPEECH_INTRODUCTION_SOMETIMES },
		{ SPEECH_CPG_KILL_CONFIRMED,						SPEECH_ARRAY_CPG_MESSAGES,			100, 	SPEECH_INTRODUCTION_SOMETIMES },
		{ SPEECH_CPG_GOOD_KILL,								SPEECH_ARRAY_CPG_MESSAGES,			100, 	SPEECH_INTRODUCTION_SOMETIMES },
	};



void play_vehicle_destroyed_speech (entity *victim, entity *aggressor)
{
	entity_sides
		victim_side,
		aggressor_side;

	int
		val,
		array_type,
		speech_index;

	ASSERT (victim);

	ASSERT (aggressor);

	victim_side = (entity_sides) get_local_entity_int_value (victim, INT_TYPE_SIDE);

	aggressor_side = (entity_sides) get_local_entity_int_value (aggressor, INT_TYPE_SIDE);

	if (victim_side == aggressor_side)
	{
		//
		// FRIENDLY FIRE
		//

		if (get_local_entity_int_value (aggressor, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
		{
			// PLAY_SPEECH (CPG)
			//
			// (Hold your fire, we're hitting friendlies) | (Cease fire, we're hitting our own guys)

			if (rand16 () & 1)
			{
				speech_index = SPEECH_CPG_FRIENDLY_FIRE1;
			}
			else
			{
				speech_index = SPEECH_CPG_FRIENDLY_FIRE2;
			}

			play_client_server_cpg_message
			(
				aggressor,
				1.0,
				3.0,
				SPEECH_CATEGORY_ATTACKING_TARGETS,
				8.0,
				speech_index
			);
		}
	}
	else
	{
		if (get_local_entity_int_value (aggressor, INT_TYPE_IDENTIFY_AIRCRAFT))
		{
			if (get_local_entity_int_value (aggressor, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI)
			{
				// PLAY_SPEECH (CPG)
				//
				// (Target destroyed) | (Enemy armour destroyed) | (Good Kill) | etc

				val = get_speech_random_value
						(
							vehicle_destroyed_cpg_comments,
							num_vh_destroyed_cpg_comments [aggressor_side],
							last_vh_destroyed_cpg_comment [aggressor_side]
						);

				speech_index = vehicle_destroyed_cpg_comments [val].speech_index;
	
				play_client_server_cpg_message
				(
					aggressor,
					0.4,
					3.0,
					SPEECH_CATEGORY_ATTACKING_TARGETS, 10.0,
					speech_index
				);
			}
			else
			{
				// PLAY_SPEECH (WINGMAN)
				//
				// (Target destroyed) | (Enemy armour destroyed) | (Good Kill) | etc

				val = get_speech_random_value
						(
							vehicle_destroyed_wingman_comments,
							num_vh_destroyed_wingman_comments [aggressor_side],
							last_vh_destroyed_wingman_comment [aggressor_side]
						);

				array_type = vehicle_destroyed_wingman_comments [val].array_type;
				speech_index = vehicle_destroyed_wingman_comments [val].speech_index;
	
				play_client_server_wingman_message
				(
					aggressor,
					0.4,
					3.0,
					SPEECH_CATEGORY_ATTACKING_TARGETS, 10.0,
					vehicle_destroyed_wingman_comments [val].introduction,
					(speech_array_types) array_type, speech_index
				);
			}
		}
	}
}





int vehicle_critically_damaged (entity *en)
{
	vehicle
		*raw;

	int
		percent_damaged;

	if (get_local_entity_type (en) == ENTITY_TYPE_PERSON)
		return FALSE;
	
	raw = (vehicle *) get_local_entity_data (en);

	percent_damaged = 100 * raw->damage_level / vehicle_database[raw->mob.sub_type].initial_damage_level;

	if (percent_damaged < VEHICLE_CRITICAL_DAMAGE_LEVEL)
		return TRUE;
	else
		return FALSE;
}

void assess_vehicle_damage_level (entity *en, int old_damage_level)
{
	vehicle
		*raw;
	float
		initial_damage,
		old_percent_damaged,
		new_percent_damaged;
	int
		explosive_quality = get_local_entity_int_value (en, INT_TYPE_EXPLOSIVE_QUALITY);

	raw = (vehicle *) get_local_entity_data (en);

	if (old_damage_level == raw->damage_level)
		return;

	if ((explosive_quality & EXPLOSIVE_QUALITY_EXPLOSIVE || explosive_quality & EXPLOSIVE_QUALITY_FLAMMABLE) && get_comms_model () == COMMS_MODEL_SERVER)
	{
		initial_damage = (float)(vehicle_database[raw->mob.sub_type].initial_damage_level);

		old_percent_damaged = (100.0 * (float)old_damage_level) / initial_damage;
		new_percent_damaged = (100.0 * (float)raw->damage_level) / initial_damage;





		// create smoke effect

		if(new_percent_damaged < VEHICLE_CRITICAL_DAMAGE_LEVEL && old_percent_damaged >= VEHICLE_CRITICAL_DAMAGE_LEVEL)
		{




			if (get_local_entity_int_value (en, INT_TYPE_EXPLOSIVE_POWER) > 2)
			{

				set_client_server_entity_float_value(en, FLOAT_TYPE_SLEEP, 6000);
			}
			else
			{

				set_client_server_entity_float_value(en, FLOAT_TYPE_SLEEP, 300);
			}

			set_client_server_entity_int_value (en, INT_TYPE_OPERATIONAL_STATE, OPERATIONAL_STATE_STOPPED);
		}
		
		if (!raw->damage_level)
			clear_smoke_list_generator_lifetime (en, ENTITY_SUB_TYPE_EFFECT_SMOKE_LIST_FIRE);
	}
}
