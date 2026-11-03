

































































#define MAX_LOG_DISPLAY_MESSAGES	25





struct MESSAGE_LOG_TYPE
{
	message_text_types
		type;

	unsigned int
		day						: NUM_DAY_BITS;

	float
		time_of_day;

	vec3d
		position;

	rgb_colour
		colour;

	char
		*string;
};

typedef struct MESSAGE_LOG_TYPE message_log_type;





extern message_log_type *get_message_log (int index);

extern int get_message_log_number_of_messages (void);

extern message_log_type *get_current_display_message (void);

extern void initialise_message_log (void);

extern void deinitialise_message_log (void);

extern int get_incoming_in_flight_message_flag (void);

extern void display_in_flight_incoming_messages (void);

extern void display_last_received_message (event *arg);

extern void set_incoming_message (entity *sender, entity *target, message_text_types type, char *string);

extern void update_message_log (void);

extern void process_radio_message (entity *en, message_categories type, int value);

extern void play_client_server_radio_message_response (entity *en, int speech_index, float priority, float expire_time);

extern void pack_local_message_log (pack_modes mode);

extern void unpack_local_message_log (pack_modes mode);







