

































































#include "project.h"

#include "setup.h"

#include "order.h"





#define DEBUG_MODULE   0

#define NUMBER_OF_CAMPAIGN_OBJECTIVES_PER_SIDE	5





static void create_force_campaign_objectives (entity *force);





void setup_campaign (void)
{
	entity
		*force;
		
	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (get_session_entity ());

	//
	// Create force overall campaign objectives
	//

	force = get_local_entity_first_child (get_session_entity (), LIST_TYPE_FORCE);

	ASSERT (force);

	while (force)
	{
		create_force_campaign_objectives (force);

		force = get_local_entity_child_succ (force, LIST_TYPE_FORCE);
	}

	//
	// Setup Order Of Battle
	//

	initialise_order_of_battle ();
}





void create_force_campaign_objectives (entity *force)
{
	entity
		*keysite,
		*target_force,
		**list;

	int
		loop,
		count,
		side,
		target_side;

	float
		highest,
		*rating;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (get_session_entity ());

	ASSERT (force);

	side = get_local_entity_int_value (force, INT_TYPE_SIDE);

	//
	// count up potential objective keysites
	//

	count = 0;

	target_force = get_local_entity_first_child (get_session_entity (), LIST_TYPE_FORCE);

	while (target_force)
	{
		if (target_force != force)
		{
			keysite = get_local_entity_first_child (target_force, LIST_TYPE_KEYSITE_FORCE);

			while (keysite)
			{
				if (get_local_entity_int_value (keysite, INT_TYPE_POTENTIAL_CAMPAIGN_OBJECTIVE))
				{
					if (get_local_entity_int_value (keysite, INT_TYPE_IN_USE))
					{
						count ++;
					}
				}

				keysite = get_local_entity_child_succ (keysite, LIST_TYPE_KEYSITE_FORCE);
			}
		}

		target_force = get_local_entity_child_succ (target_force, LIST_TYPE_FORCE);
	}

	if (count == 0)
	{
		debug_fatal ("SETUP: No potential campaign objectives for side %s", entity_side_short_names [side]);
	}

	//
	// construct the list and rate each keysite according to sector importance
	//

	list = (entity **) malloc_heap_mem (sizeof (entity *) * count);

	rating = (float*)malloc_heap_mem (sizeof (float) * count);

	highest = 0.0;

	count = 0;

	target_force = get_local_entity_first_child (get_session_entity (), LIST_TYPE_FORCE);

	while (target_force)
	{
		if (target_force != force)
		{
			target_side = get_local_entity_int_value (target_force, INT_TYPE_SIDE);

			keysite = get_local_entity_first_child (target_force, LIST_TYPE_KEYSITE_FORCE);

			while (keysite)
			{
				if (get_local_entity_int_value (keysite, INT_TYPE_POTENTIAL_CAMPAIGN_OBJECTIVE))
				{
					if (get_local_entity_int_value (keysite, INT_TYPE_IN_USE))
					{
						float
							actual_range;
	
						vec3d
							*pos;
	
						list [count] = keysite;
	
						pos = get_local_entity_vec3d_ptr (keysite, VEC3D_TYPE_POSITION);
	
						get_closest_keysite (NUM_ENTITY_SUB_TYPE_KEYSITES, (entity_sides) target_side, pos, 1.0 * KILOMETRE, &actual_range, TRUE, keysite);
	
						rating [count] = actual_range;
	
						highest = max (highest, rating [count]);
		
						count ++;
					}
				}

				keysite = get_local_entity_child_succ (keysite, LIST_TYPE_KEYSITE_FORCE);
			}
		}

		target_force = get_local_entity_child_succ (target_force, LIST_TYPE_FORCE);
	}

	//
	// Normalise ratings
	//

	if (highest == 0.0)
	{
		debug_fatal ("SETUP: No sector importance for side %s", entity_side_short_names [side]);
	}
	
	for (loop = 0; loop < count; loop ++)
	{
		rating [loop] = rating [loop] / highest;
	}

	//
	// obligatory random factor
	//

	for (loop = 0; loop < count; loop ++)
	{
		rating [loop] += frand1 ();
	}

	//
	// sort the list
	//

	quicksort_entity_list (list, count, rating);

	//
	// now find the best N targets and link them into the force
	//

	count = min (count, NUMBER_OF_CAMPAIGN_OBJECTIVES_PER_SIDE);

	for (loop = 0; loop < count; loop ++)
	{
		insert_local_entity_into_parents_child_list (list [loop], LIST_TYPE_CAMPAIGN_OBJECTIVE, force, NULL);

		debug_log ("Side %s Objective :- %s (%s)", entity_side_short_names [side],
						get_local_entity_string (list [loop], STRING_TYPE_KEYSITE_NAME),
						get_local_entity_string (list [loop], STRING_TYPE_FULL_NAME));
	}

	free_mem (list);

	free_mem (rating);
}






