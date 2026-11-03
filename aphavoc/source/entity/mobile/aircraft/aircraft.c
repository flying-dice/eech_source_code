

































































#include "project.h"





#define DEBUG_MODULE 0
#define AIRCRAFT_MORTAL_DAMAGE_LEVEL 30
#define AIRCRAFT_CRITICAL_DAMAGE_LEVEL 5
#define MAX_META_SMOKE_LIST_ATTACHMENT_POINTS 	2

aircraft_damage_types aircraft_critically_damaged (entity *en)
{

	aircraft
		*raw;

	int
		percent_damaged;

	raw = (aircraft *) get_local_entity_data (en);

	percent_damaged = (100 * raw->damage_level) / aircraft_database[raw->mob.sub_type].initial_damage_level;

	if (percent_damaged < AIRCRAFT_CRITICAL_DAMAGE_LEVEL)
	{
		return AIRCRAFT_DAMAGE_CRITICAL;
	}
	else if (percent_damaged < AIRCRAFT_MORTAL_DAMAGE_LEVEL)
	{
		return AIRCRAFT_DAMAGE_MORTAL;
	}
	else
	{
		return AIRCRAFT_DAMAGE_NONE;
	}
}





static struct ATTACHED_META_SMOKE_LIST_TABLE
{

	meta_smoke_list_types							type;
	object_3d_sub_object_index_numbers			attachment_point[MAX_META_SMOKE_LIST_ATTACHMENT_POINTS];

} attached_meta_smoke_list_table[] =
	{

		//
		// LIGHT DAMAGE MOVING
		//

		{
			META_SMOKE_LIST_TYPE_AIRCRAFT_LIGHT_DAMAGE_MOVING,
			{
				OBJECT_3D_SUB_OBJECT_DAMAGED_SMOKE,
				-1
			}
		},

		//
		// HEAVY DAMAGE MOVING
		//

		{
			META_SMOKE_LIST_TYPE_AIRCRAFT_HEAVY_DAMAGE_MOVING,
			{
				OBJECT_3D_SUB_OBJECT_DAMAGED_SMOKE,
				-1
			}
		},

		//
		// LIGHT DAMAGE STATIC
		//

		{
			META_SMOKE_LIST_TYPE_AIRCRAFT_LIGHT_DAMAGE_STATIC,
			{
				OBJECT_3D_SUB_OBJECT_DAMAGED_SMOKE,
				-1
			}
		},

		//
		// HEAVY DAMAGE STATIC
		//

		{
			META_SMOKE_LIST_TYPE_AIRCRAFT_HEAVY_DAMAGE_STATIC,
			{
				OBJECT_3D_SUB_OBJECT_DAMAGED_SMOKE,
				-1
			}
		},

		//
		// TERMINATOR
		//

		{
			(meta_smoke_list_types) -1,
			{
				-1,
				-1
			}
		}
	};



































































































void assess_aircraft_damage_level (entity *en, int old_damage_level, int new_damage_level)
{

	float
		initial_damage,
		old_percent_damaged,
		new_percent_damaged;

	if (old_damage_level == new_damage_level)
	{
		return;
	}
	
	initial_damage = (float)(aircraft_database[get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE)].initial_damage_level);
	
	old_percent_damaged = (100.0 * (float)old_damage_level) / initial_damage;
	new_percent_damaged = (100.0 * (float)new_damage_level) / initial_damage;




		
	// Eject Pilot(s)

	if (get_local_entity_int_value (en, INT_TYPE_AIRBORNE_AIRCRAFT) && get_local_entity_int_value (en, INT_TYPE_PLAYER) == ENTITY_PLAYER_AI && new_percent_damaged < 0.75 * AIRCRAFT_CRITICAL_DAMAGE_LEVEL && new_damage_level > 10)
		initiate_aircraft_crew_ejection (en);

	// update smoke
	
	if ((new_percent_damaged < AIRCRAFT_CRITICAL_DAMAGE_LEVEL && old_percent_damaged >= AIRCRAFT_CRITICAL_DAMAGE_LEVEL) || 
			(new_percent_damaged < AIRCRAFT_MORTAL_DAMAGE_LEVEL && old_percent_damaged >= AIRCRAFT_MORTAL_DAMAGE_LEVEL)) // damage threshold reached
	{
		switch_aircraft_damaged_smoke(en);
		
		if (get_comms_model() == COMMS_MODEL_SERVER && new_damage_level > 10) // return group home
		{
			entity
				*group,
				*guide,
				*task;
			
			group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

			if (group)
			{
				guide = get_local_group_primary_guide (group);
						
				if (guide)
				{
					task = get_local_entity_parent (guide, LIST_TYPE_GUIDE);




					if (task->task_data->task_priority <= 5.0)
						group_return_to_base (group);
				}
			}
		}
	}
		
	if (new_percent_damaged >= AIRCRAFT_MORTAL_DAMAGE_LEVEL && old_percent_damaged < AIRCRAFT_MORTAL_DAMAGE_LEVEL) // recovery happened
		switch_aircraft_damaged_smoke(en);
		
}





static int
	num_ac_hit_aggressor_wingman_comments [NUM_ENTITY_SIDES] = {0, 1, 1},
	last_ac_hit_aggressor_wingman_comment [NUM_ENTITY_SIDES] = {0, 0, 0};

static speech_instance_type
	aircraft_hit_aggressor_wingman_comments [] =
	{
		{ SPEECH_WINGMAN_TARGET_HIT,					SPEECH_ARRAY_WINGMAN_MESSAGES, 100,	SPEECH_INTRODUCTION_SOMETIMES },
	};



static int
	num_ac_hit_aggressor_cpg_comments [NUM_ENTITY_SIDES] = {0, 1, 1},
	last_ac_hit_aggressor_cpg_comment [NUM_ENTITY_SIDES] = {0, 0, 0};

static speech_instance_type
	aircraft_hit_aggressor_cpg_comments [] =
	{
		{ SPEECH_CPG_TARGET_HIT,								SPEECH_ARRAY_CPG_MESSAGES, 100,	SPEECH_INTRODUCTION_NEVER },
	};



static int
	num_ac_hit_victim_wingman_comments [NUM_ENTITY_SIDES] = {0, 1, 1},
	last_ac_hit_victim_wingman_comment [NUM_ENTITY_SIDES] = {0, 0, 0};

static speech_instance_type
	aircraft_hit_victim_wingman_comments [] =
	{
		{ SPEECH_WINGMAN_TAKING_HITS,					SPEECH_ARRAY_WINGMAN_MESSAGES,	100,	SPEECH_INTRODUCTION_ALWAYS },
	};



































































































































































































































static int
	num_ac_destroyed_cpg_comments [NUM_ENTITY_SIDES] = {0, 4, 4},
	last_ac_destroyed_cpg_comment [NUM_ENTITY_SIDES] = {0, 0, 0};

static speech_instance_type
	aircraft_destroyed_cpg_comments [] =
	{
		{ SPEECH_CPG_TARGET_DESTROYED,						SPEECH_ARRAY_CPG_MESSAGES,			100, 	SPEECH_INTRODUCTION_SOMETIMES },
		{ SPEECH_CPG_TARGET_IS_DOWN,							SPEECH_ARRAY_CPG_MESSAGES,			100, 	SPEECH_INTRODUCTION_SOMETIMES },
		{ SPEECH_CPG_KILL_CONFIRMED,							SPEECH_ARRAY_CPG_MESSAGES,			100, 	SPEECH_INTRODUCTION_SOMETIMES },
		{ SPEECH_CPG_GOOD_KILL,									SPEECH_ARRAY_CPG_MESSAGES,			100, 	SPEECH_INTRODUCTION_SOMETIMES },
	};




































































































































































