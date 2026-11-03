

































































struct PILOT
{

	entity_sub_types
		sub_type;

	char
		pilots_name [STRING_TYPE_PLAYERS_NAME_MAX_LENGTH + 1];

	int
		rank,
		kills;

	int
		unique_id;

	crew_roles
		crew_role;

	list_root
		pilot_lock_root;

	list_link
		aircrew_link,
		pilot_link,
		player_task_link;

	unsigned int
		side					: NUM_SIDE_BITS,
		difficulty_level	: NUM_DIFFICULTY_LEVEL_BITS;
};

typedef struct PILOT pilot;





struct CLIENT_PILOT_REQUEST_DATA
{

	char
		name [STRING_TYPE_PLAYERS_NAME_MAX_LENGTH + 1];

	int
		string_length,
		side,
		rank,
		sub_type,
		unique_id,
		difficulty;
};

typedef struct CLIENT_PILOT_REQUEST_DATA client_pilot_request_data;

























extern void set_pilot_entity (entity *en);

extern entity *create_new_pilot_entity (const char *name, entity_sides side, int rank, entity_sub_types sub_type, int unique_id, int difficulty);

extern void create_server_pilot (void);

extern void create_client_pilot (void);

extern entity *get_player_task (entity *en);

extern void send_pilot_quit_message (entity *en);

extern void initialise_pilot_high_score_table (void);

extern int update_pilot_high_score_table (void);















extern entity
	*pilot_entity;





#define get_pilot_entity() (pilot_entity)




