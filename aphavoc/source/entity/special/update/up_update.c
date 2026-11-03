

































































#include "project.h"

#include "entity/tacview/tacview.h"





#define DEBUG_MODULE 0





entity
	*update_entity = NULL,
	*update_succ = NULL;

unsigned int
	moved_entities[MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8)],
	rotated_entities[MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8)],
	updated_entities [MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8)];





void set_update_entity (entity *en)
{
	update_entity = en;

	if (en)
	{
		debug_log ("Setting update_entity (type = %s, index = %d)", get_local_entity_type_name (en), get_local_entity_index (en));
	}
	else
	{
		debug_log ("Setting update_entity to NULL");
	}
}





void update_client_server_entities (void)
{
	entity
		*en;

	float
		delta_time;

	int
		loop,
		iterations;

	ASSERT (get_update_entity ());



	iterations = 1;

	if (!locked_frame_rate)
	{

		delta_time = get_delta_time ();

		iterations = set_entity_update_frame_rate (command_line_entity_update_frame_rate);
	}

	if (tacview_is_logging() && tacview_reset_frame())
	{
		write_tacview_frame_header();
		memset (moved_entities, 0, MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8));
		memset (rotated_entities, 0, MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8));
	}

	for (loop = 0; loop < iterations; loop ++)
	{



		//
		// update_entities is coded around the entity system to avoid spinning through
		// the update list twice (which Vtune shows up as slow). MB_INT sets and gets
		// the values. You may only set the value to true. It is set to false by the
		// memset below.
		//

		memset (updated_entities, 0, MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8));

		//
		// replaces....
		//
		/*
		en = get_local_entity_first_child (get_update_entity (), LIST_TYPE_UPDATE);

		while (en)
		{

			set_local_entity_int_value (en, INT_TYPE_UPDATED, FALSE);

			en = get_local_entity_child_succ (en, LIST_TYPE_UPDATE);
		}
		*/


		en = get_local_entity_first_child (get_update_entity (), LIST_TYPE_UPDATE);

		while (en)
		{
			set_update_succ (get_local_entity_child_succ (en, LIST_TYPE_UPDATE));

			update_client_server_entity (en);

			set_local_entity_int_value (en, INT_TYPE_UPDATED, TRUE);

			en = get_update_succ ();
		}

		set_update_succ (NULL);


	}

	if (!locked_frame_rate)
	{

		set_manual_delta_time (delta_time);
	}


}





int set_entity_update_frame_rate (int frame_rate)
{

	int
		entity_update_iterations;

	float
		entity_update_delta_time;

	entity_update_iterations = 1;

	ASSERT ((frame_rate >= 1) && (frame_rate <= 100));

	entity_update_iterations = (int) (get_delta_time () * frame_rate + 1.0);

	entity_update_delta_time = get_delta_time () / entity_update_iterations;

	#if DEBUG_MODULE

	debug_log ("UPDATE: requested frame rate %d, delta time %f, iterations %d, set_delta_time %f", frame_rate, get_delta_time (), entity_update_iterations, entity_update_delta_time);

	#endif

	set_manual_delta_time (entity_update_delta_time);

	return entity_update_iterations;
}




