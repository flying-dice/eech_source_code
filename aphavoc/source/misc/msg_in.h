

































































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





















extern void set_incoming_message (entity *sender, entity *target, message_text_types type, char *string);



extern void process_radio_message (entity *en, message_categories type, int value);

extern void play_client_server_radio_message_response (entity *en, int speech_index, float priority, float expire_time);











