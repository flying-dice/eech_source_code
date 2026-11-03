






































































#define COMMS_STATS  2





enum COMMS_MODEL_TYPES
{

   COMMS_MODEL_SERVER,
   COMMS_MODEL_CLIENT,
   NUM_COMMS_MODEL_TYPES
};

typedef enum COMMS_MODEL_TYPES comms_model_types;

extern const char
	*comms_model_string,
	*comms_model_names [],
	*send_type_names [];





enum SEND_TYPES
{

   SEND_TYPE_PERSONAL,
   SEND_TYPE_GROUP,
   NUM_SEND_TYPES
};

typedef enum SEND_TYPES send_types;





enum COMMS_DATA_FLOW_TYPES
{

   COMMS_DATA_FLOW_TX,
   COMMS_DATA_FLOW_RX,
   NUM_COMMS_DATA_FLOW_TYPES
};

typedef enum COMMS_DATA_FLOW_TYPES comms_data_flow_types;





enum PACKET_TYPES
{

   //
   // System packets, used internally
   //

   PACKET_TYPE_INVALID = 0,

	//
	// Personal Packets
	//

	PACKET_TYPE_FIRST_PERSONAL_PACKET,

   PACKET_TYPE_RESEND_PACKET,
	PACKET_TYPE_SESSION_INFO,
   PACKET_TYPE_CLIENT_PILOT_REQUEST,
   PACKET_TYPE_CLIENT_GUNSHIP_REQUEST,
   PACKET_TYPE_CLIENT_CAMPAIGN_DATA_REQUEST,
   PACKET_TYPE_PILOT_REQUEST_ACCEPTED,
   PACKET_TYPE_GUNSHIP_REQUEST_REFUSED,
   PACKET_TYPE_GUNSHIP_REQUEST_ACCEPTED,
   PACKET_TYPE_MISSION_DATA,
   PACKET_TYPE_FRAME_ID,
	PACKET_TYPE_CLIENT_FRAME_ID,
   PACKET_TYPE_SERVER_REJECTED,				
   PACKET_TYPE_END_GAME,
   PACKET_TYPE_CONNECTION_VALIDATION,
   PACKET_TYPE_CONNECTION_RESPONSE,
   PACKET_TYPE_SETTINGS_REQUEST, // Jabberwock 031118 Server side settings
   PACKET_TYPE_SETTINGS_DATA,

	//
	// Group Packets
	//

	PACKET_TYPE_FIRST_GROUP_PACKET,

   PACKET_TYPE_SESSION_QUERY,
   PACKET_TYPE_AI_DATA,

	//
	//
	//

	NUM_PACKET_TYPES
};

typedef enum PACKET_TYPES packet_types;





extern comms_model_types
   system_comms_model;

extern comms_data_flow_types
   system_comms_data_flow;

extern DPID
   system_server_id;

extern int
   send_group_frame_id,
	resend_packet_list_counter,
	group_packet_list_counter,
	personal_packet_list_counter;

struct STUB_PACKET_TYPE
{

	int
		send_type,		// group or personal
		packet_id, 		// used for rerequest list
		frame_id, 		// used for rerequest list
		actual_time,	// used for rerequest list
		received_time;

	struct PACKET_HEADER_TYPE
		*packet;

	struct STUB_PACKET_TYPE
		*prev,
		*next;
};

typedef struct STUB_PACKET_TYPE stub_packet_type;





extern stub_packet_type
	*rerequest_packet_list,
	*resend_packet_list,
	*received_group_packets,
	*received_personal_packets;

extern float
	comms_weapon_lag_timing[3];
extern int
	comms_weapon_damage_multiplier[64][3];

#if COMMS_STATS

extern float
	av_packet_size_sent,
	av_packet_size_received,
	instant_packet_size_sent;

extern int
	packet_record_data_count;

#define zero_average_pack_size() (av_packet_size_sent = 0.0, av_packet_size_received = 0.0)

#else

#define zero_average_pack_size() (TRUE)

#endif





extern void set_comms_data_flow (comms_data_flow_types data_flow);

extern void set_comms_model (comms_model_types model);

#define get_comms_model() (system_comms_model)

#define get_comms_data_flow() (system_comms_data_flow)

#define set_server_id(SERVER_ID) (system_server_id = (SERVER_ID))

#define get_server_id() (system_server_id)

#define set_group_frame_id(FRAME_ID) (send_group_frame_id = (FRAME_ID))

#define get_group_frame_id() (send_group_frame_id)

#define get_packet_send_type(PACKET) (((PACKET)->type > PACKET_TYPE_FIRST_GROUP_PACKET) ? SEND_TYPE_GROUP : SEND_TYPE_PERSONAL)





#define add_stub_packet_to_receive_list(LIST,PACKET) (((LIST) == SEND_TYPE_GROUP) ? (add_stub_packet_to_list (&received_group_packets, (PACKET)), group_packet_list_counter ++) : (add_stub_packet_to_list (&received_personal_packets, (PACKET)), personal_packet_list_counter ++))

#define add_stub_packet_to_resend_list(PACKET) (add_stub_packet_to_list (&resend_packet_list, (PACKET)), resend_packet_list_counter ++)

#define remove_stub_packet_from_resend_list(PACKET) (remove_stub_packet_from_list (&resend_packet_list, (PACKET)), resend_packet_list_counter --)

#define remove_stub_packet_from_receive_list(LIST,PACKET) (((LIST) == SEND_TYPE_GROUP) ? (remove_stub_packet_from_list (&received_group_packets, (PACKET)), group_packet_list_counter --) : (remove_stub_packet_from_list (&received_personal_packets, (PACKET)), personal_packet_list_counter --))





struct PACKET_HEADER_TYPE
{

   packet_types
      type;

   DPID
      player_id;

   int
      frame_id;

   short int
      number_of_packets,
      packet_id,
      data_size;

	int
		check_sum;
};

typedef struct PACKET_HEADER_TYPE packet_header_type;





/* forward declarations for types used below */
struct CONNECTION_LIST_TYPE;
/*********************************************/

extern void initialise_comms (void);

extern void reset_comms_data (void);

extern void initialise_comms_debug (void);

extern void deinitialise_comms (void);

extern void send_packet (DPID player_id, packet_types type, unsigned char *data, int size, send_types send_type);

extern void receive_packets (void);

extern int process_packet_list (send_types send_type, struct CONNECTION_LIST_TYPE *connection, DPID *player_id, unsigned char **data, int *size);

extern DPID get_packet_destination (void);

extern stub_packet_type *resend_packet (DPID player_id, int frame_id, int packet_id, send_types send_type);

extern void free_connection_packets (DPID connection_id);

extern void remove_stub_packet_from_list (stub_packet_type **list, stub_packet_type *packet);

extern void update_resend_packet_list (int packet_limit);

#if COMMS_STATS

extern void debug_comms_stats (void);

#endif

extern void reset_comms_stats (void);

extern void reset_receive_packet_list_time_stamps (void);

extern void display_comms_stats (void);

extern int comms_weapon_damage_multiplier_value (entity *launcher, int burst_size, int weapon_sub_type);





#include "comm_man.h"

// JB 030311 VC rename source files
#include "commsserver.h"




