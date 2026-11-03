

































































#include "project.h"

#include "servlog.h" // Jabberwock Server log





#define DEBUG_MODULE 0

#define NUM_TABLE_ENTRIES 10

#define FLASH_RATE 1.0





entity
	*pilot_entity = NULL;










struct PILOT_SCORE_TYPE
{
	int
		kills,
		valid;

	entity_sides
		side;

	char
		name [STRING_TYPE_PLAYERS_NAME_MAX_LENGTH + 1];
};

typedef struct PILOT_SCORE_TYPE pilot_score_type;

pilot_score_type pilot_high_score_table [NUM_TABLE_ENTRIES];

static void qs_table (pilot_score_type *list, int left, int right);





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

		// turn on NEXT button now Pilot entity has arrived
		if (get_comms_model () == COMMS_MODEL_CLIENT)
		{

			set_display_gunship_buttons (FALSE, "ENGAGE");

			// turn on only the gunship_next button
			set_ui_object_drawable (gunship_screen_next_button, TRUE);
		}
		//-- Werewolf
		else
		{
			// If we're the server, remember our player name. This will be sent out in the heartbeat packets.
			net_set_hostname( get_local_entity_string (en, STRING_TYPE_PILOTS_NAME) );
		}
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





void create_client_pilot (void)
{
	client_pilot_request_data
		pilot_data;

	ASSERT (get_comms_model () == COMMS_MODEL_CLIENT);

	//
	// pack required gunship data
	//

	strcpy (pilot_data.name, get_player_log_name (get_current_player_log ()));
	pilot_data.side = get_global_gunship_side ();
	pilot_data.rank = get_player_log_rank (get_global_gunship_side (), get_current_player_log ());
	pilot_data.sub_type = ENTITY_SUB_TYPE_PILOT_PILOT;
	pilot_data.unique_id = direct_play_get_player_id ();
	pilot_data.difficulty = get_global_difficulty_level ();

	//
	// Send request
	//

	send_packet (get_server_id (), PACKET_TYPE_CLIENT_PILOT_REQUEST, (unsigned char *) &pilot_data, sizeof (client_pilot_request_data), SEND_TYPE_PERSONAL);

	// turn off NEXT button while Pilot entity is being created client_server
	if (get_comms_model () == COMMS_MODEL_CLIENT)
	{

		set_display_gunship_buttons (FALSE, "ENGAGE");
	}
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





















































































































































































































































































































































































