

































































#include "project.h"







#define DEBUG_MODULE 0









entity
	*pilot_entity = NULL;

































static void send_pilot_joined_message (entity *en);

static int get_session_pilot_count (void);





void set_pilot_entity (entity *en)
{
	comms_data_flow_types
		store_data_flow;

	if (en)
	{
		debug_log ("PILOT: Setting pilot_entity to %s", get_local_entity_string (en, STRING_TYPE_PILOTS_NAME));

		ASSERT (pilot_entity == NULL);

		ASSERT (get_local_entity_type (en) == ENTITY_TYPE_PILOT);

		pilot_entity = en;
















		//-- Werewolf
	}
	else if (pilot_entity)
	{
		debug_log ("PILOT: Setting pilot_entity to NULL");

		ASSERT (pilot_entity);

		//
		// Program MUST be in TX mode otherwise clients pilot will not be destroyed on the server
		//

		store_data_flow = get_comms_data_flow ();

		set_comms_data_flow (COMMS_DATA_FLOW_TX);

		destroy_client_server_entity (pilot_entity);

		set_comms_data_flow (store_data_flow);

		pilot_entity = NULL;
	}
}




















































































// 040220 Jabberwock ends





entity *create_new_pilot_entity (const char *name, entity_sides side, int rank, entity_sub_types sub_type, int unique_id, int difficulty)
{
	entity
		*en,
		*force;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (name);

	force = get_local_force_entity (side);

	ASSERT (force);

	en = create_client_server_entity
				(
					ENTITY_TYPE_PILOT,
					ENTITY_INDEX_DONT_CARE,
					ENTITY_ATTR_PARENT (LIST_TYPE_PILOT, force),
					ENTITY_ATTR_STRING (STRING_TYPE_PILOTS_NAME, name),
					ENTITY_ATTR_INT_VALUE (INT_TYPE_ENTITY_SUB_TYPE, sub_type),
					ENTITY_ATTR_INT_VALUE (INT_TYPE_UNIQUE_ID, unique_id),
					ENTITY_ATTR_INT_VALUE (INT_TYPE_SIDE, side),
					ENTITY_ATTR_INT_VALUE (INT_TYPE_PILOT_RANK, rank),
					ENTITY_ATTR_INT_VALUE (INT_TYPE_DIFFICULTY_LEVEL, difficulty),
					ENTITY_ATTR_END
				);

	send_pilot_joined_message (en);

	return en;
}





void create_server_pilot (void)
{
	entity
		*en;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	en = create_new_pilot_entity
				(
					get_player_log_name (get_current_player_log ()),
					get_global_gunship_side (),
					get_player_log_rank (get_global_gunship_side (), get_current_player_log ()),
					ENTITY_SUB_TYPE_PILOT_PILOT,
					direct_play_get_player_id (),
					get_global_difficulty_level ()
				);

	set_pilot_entity (en);
}









































int get_session_pilot_count (void)
{
	entity
		*force,
		*pilot;

	int
		count;

	ASSERT (get_session_entity ());

	count = 0;

	force = get_local_entity_first_child (get_session_entity (), LIST_TYPE_FORCE);

	while (force)
	{
		pilot = get_local_entity_first_child (force, LIST_TYPE_PILOT);

		while (pilot)
		{
			count ++;

			pilot = get_local_entity_child_succ (pilot, LIST_TYPE_PILOT);
		}

		force = get_local_entity_child_succ (force, LIST_TYPE_FORCE);
	}

	return count;
}














entity *get_player_task (entity *en)
{
	if (!en)
	{
		en = get_pilot_entity ();
	}

	ASSERT (en);

	ASSERT (get_local_entity_type (en) == ENTITY_TYPE_PILOT);

	return get_local_entity_parent (en, LIST_TYPE_PLAYER_TASK);
}





















































































































































































































































































































































































