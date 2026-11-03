

































































#include "project.h"

#include "suitable.h"





#define DEBUG_MODULE   0





static float
	**group_task_array = NULL;





static float calculate_group_to_task_suitability (entity_sub_types group_type, entity_sub_types task_type)
{

	const ai_statistics
		*group_factors,
		*task_factors;

	float
		a,
		b,
		result;

	int
		landing_type;

	ASSERT (group_task_array);

	group_factors = &(group_database [group_type].ai_stats);

	task_factors = &(task_database [task_type].ai_stats);

	//
	// Critical factors
	//

	// Movement type 

	if ((task_database [task_type].movement_type != MOVEMENT_TYPE_ALL) && (task_database [task_type].movement_type != group_database [group_type].movement_type))
	{
		return 0.0;
	}

	landing_type = group_database [group_type].default_landing_type;

	if (!(task_database [task_type].landing_types & (1 << landing_type)))
	{
		return 0.0;
	}

	// Movement speed 

	if (task_factors->movement_speed > group_factors->movement_speed)
	{
		return 0.0;
	}

	// Movement stealth

	if (task_factors->movement_stealth > group_factors->movement_stealth)
	{
		return 0.0;
	}

	// Cargo space

	if (task_factors->cargo_space > group_factors->cargo_space)
	{
		return 0.0;
	}

	// Troop space

	if (task_factors->troop_space > group_factors->troop_space)
	{
		return 0.0;
	}

	// Ground Attack Strength

	if (task_factors->ground_attack_strength > group_factors->ground_attack_strength)
	{
		return 0.0;
	}

	// Air Attack Strength

	if (task_factors->air_attack_strength > group_factors->air_attack_strength)
	{
		return 0.0;
	}

	// Engage enemy (If task requires engagement - group must be capable)

	if (task_database [task_type].engage_enemy)
	{
		if (!group_database [group_type].default_engage_enemy)
		{
			return 0.0;
		}
	}

	//
	// Non Critical factors
	//

	result = 1.0;

	if (task_factors->air_attack_strength > 0)
	{
		a = (float)(group_factors->air_attack_strength);

		b = (float)(task_factors->air_attack_strength);

		result *= min ((a / b), 1.0f);
	}

	if (task_factors->ground_attack_strength > 0)
	{
		a = (float)(group_factors->ground_attack_strength);

		b = (float)(task_factors->ground_attack_strength);

		result *= min ((a / b), 1.0f);
	}

	ASSERT ((result >= 0.0) && (result <= 1.0));

	return result;
}





float get_group_to_task_suitability (entity_sub_types group_type, entity_sub_types task_type)
{
	// JB 030313 Fly any aircraft (test code)
	// return 1;
	return group_task_array [group_type][task_type];
}





void initialise_group_task_array (void)
{
	int
		group,
		task;

	ASSERT (!group_task_array);

	group_task_array = (float **) malloc_heap_mem (sizeof (float *) * NUM_ENTITY_SUB_TYPE_GROUPS);

	for (group = 0; group < NUM_ENTITY_SUB_TYPE_GROUPS; group ++)
	{
		group_task_array [group] = (float*) malloc_heap_mem (sizeof (float) * NUM_ENTITY_SUB_TYPE_TASKS);

		for (task = 0; task < NUM_ENTITY_SUB_TYPE_TASKS; task ++)
		{
			group_task_array [group][task] = calculate_group_to_task_suitability (group, task);
		}
	}





















}





void deinitialise_group_task_array (void)
{
	int
		loop;

	ASSERT (group_task_array);

	for (loop = 0; loop < NUM_ENTITY_SUB_TYPE_GROUPS; loop ++)
	{
		free_mem (group_task_array [loop]);
	}

	free_mem (group_task_array);

	group_task_array = NULL;
}





