

































































enum ENTITY_FORCE_ACTION_TYPES
{

	ENTITY_FORCE_ACTION_INVALID,
	ENTITY_FORCE_ACTION_RETREAT,
	ENTITY_FORCE_ACTION_NOTHING,
	ENTITY_FORCE_ACTION_RECON,
	ENTITY_FORCE_ACTION_INTERCEPT,
	ENTITY_FORCE_ACTION_DEEP_STRIKE,
	NUM_ENTITY_FORCE_ACTIONS
};

typedef enum ENTITY_FORCE_ACTION_TYPES entity_force_action_types;

extern const char
	*entity_force_action_names [];





enum ENTITY_FORCE_ATTITUDE_TYPES
{
	ENTITY_FORCE_ATTITUDE_COWARD,
	ENTITY_FORCE_ATTITUDE_PASSIVE,
	ENTITY_FORCE_ATTITUDE_CAUTIOUS,
	ENTITY_FORCE_ATTITUDE_NORMAL,
	ENTITY_FORCE_ATTITUDE_FEISTY,
	ENTITY_FORCE_ATTITUDE_AGGRESSIVE,
	ENTITY_FORCE_ATTITUDE_DESTRUCTIVE,
	ENTITY_FORCE_ATTITUDE_ERRATIC,
	NUM_ENTITY_FORCE_ATTITUDES
};

typedef enum ENTITY_FORCE_ATTITUDE_TYPES entity_force_attitude_types;

extern const char
	*entity_force_attitude_names [];





enum ENTITY_FORCES
{
	ENTITY_FORCE_UNKNOWN,
	ENTITY_FORCE_AIR,
	ENTITY_FORCE_GROUND,
	ENTITY_FORCE_SEA,
	NUM_ENTITY_FORCES
};

typedef enum ENTITY_FORCES entity_forces;

extern const char
	*entity_force_names [];





enum CAMPAIGN_RESULT_TYPES
{

	CAMPAIGN_RESULT_NONE,
	CAMPAIGN_RESULT_FAIL,
	CAMPAIGN_RESULT_SUCCESS,
	CAMPAIGN_RESULT_STALEMATE,
	CAMPAIGN_RESULT_OUTOFHARDWARE,
	CAMPAIGN_RESULT_SERVER_REJECTED,
	NUM_CAMPAIGN_RESULT_TYPES
};

typedef enum CAMPAIGN_RESULT_TYPES campaign_result_types;

extern const char
	*campaign_result_names [];





struct CAMPAIGN_CRITERIA_TYPE
{

	short int
		criteria_type,
		valid,
		result;

	int
		rank_points,
		experience_points,
		*rank_variable,
		*experience_variable;

	union
	{

		struct
		{

			short int
				goal,
				count,
				type;
		};

		struct
		{

			short int
				days,
				hours,
				minutes,
				seconds;
		};

		struct
		{

			short int
				value1,
				value2,
				value3,
				value4;
		};
	};

	struct CAMPAIGN_CRITERIA_TYPE
		*next;
};

typedef struct CAMPAIGN_CRITERIA_TYPE campaign_criteria_type;





enum CAMPAIGN_TRIGGER
{

	CAMPAIGN_TRIGGER_NONE,
	CAMPAIGN_TRIGGER_BALANCE_OF_POWER,
	CAMPAIGN_TRIGGER_TASK_COMPLETED,
	CAMPAIGN_TRIGGER_TASK_FAILED,
	CAMPAIGN_TRIGGER_OBJECT_DESTROYED,
	CAMPAIGN_TRIGGER_OBJECT_FIRED,
	CAMPAIGN_TRIGGER_OBJECT_TARGETED,
	CAMPAIGN_TRIGGER_OBJECT_LANDED,
	CAMPAIGN_TRIGGER_INEFFICIENT_KEYSITE,
	CAMPAIGN_TRIGGER_WAYPOINT_REACHED,
	CAMPAIGN_TRIGGER_SECTOR_WON,
	CAMPAIGN_TRIGGER_SECTOR_LOST,
	CAMPAIGN_TRIGGER_SECTOR_REACHED,
	CAMPAIGN_TRIGGER_TIME_DURATION,
	CAMPAIGN_TRIGGER_VARIABLE_CONDITION,
	CAMPAIGN_TRIGGER_RANDOM,
	CAMPAIGN_TRIGGER_USER_LANDED,
	CAMPAIGN_TRIGGER_KEY_PRESS,
	
	NUM_CAMPAIGN_TRIGGER_TYPES
};

typedef enum CAMPAIGN_TRIGGER campaign_trigger;

extern const char
	*campaign_trigger_names [];





struct TASK_GENERATION_TYPE
{

	unsigned short int
		valid						: 1;

	int
		created,
		completed,
		failed;
};

typedef struct TASK_GENERATION_TYPE task_generation_type;





enum FORCE_INFO_CATAGORIES
{

	FORCE_INFO_CATAGORY_ARMED_FIXED_WING,
	FORCE_INFO_CATAGORY_UNARMED_FIXED_WING,
	FORCE_INFO_CATAGORY_ARMED_HELICOPTER,
	FORCE_INFO_CATAGORY_UNARMED_HELICOPTER,
	FORCE_INFO_CATAGORY_ARMED_ROUTED_VEHICLE,
	FORCE_INFO_CATAGORY_UNARMED_ROUTED_VEHICLE,
	FORCE_INFO_CATAGORY_ARMED_SHIP_VEHICLE,
	FORCE_INFO_CATAGORY_UNARMED_SHIP_VEHICLE,
	NUM_FORCE_INFO_CATAGORIES
};

typedef enum FORCE_INFO_CATAGORIES force_info_catagories;

extern const char
	*force_info_catagory_names [];





#define ENTITY_FORCE_MIN	(ENTITY_FORCE_UNKNOWN + 1)
#define ENTITY_FORCE_MAX	(NUM_ENTITY_FORCES - 1)





struct CAMPAIGN_TRIGGER_TYPE
{

	short int
		type,
		trigger_type;

	union
	{

		struct
		{

			short int
				days,
				hours,
				minutes,
				seconds;
		};

		struct
		{

			short int
				value1,
				value2,
				value3,
				value4;
		};
	};

	char
		*event_name;

	struct CAMPAIGN_TRIGGER_TYPE
		*next;
};

typedef struct CAMPAIGN_TRIGGER_TYPE campaign_trigger_type;





struct CAMPAIGN_EVENT_TYPE
{

	char
		*name,
		*filename;

	int
		triggered,
		file_offset;

	struct CAMPAIGN_EVENT_TYPE
		*next;
};

typedef struct CAMPAIGN_EVENT_TYPE campaign_event_type;





struct CAMPAIGN_WHILE_LOOP_TYPE
{

	char
		*filename;

	int
		file_offset;

	struct CAMPAIGN_WHILE_LOOP_TYPE
		*next;
};

typedef struct CAMPAIGN_WHILE_LOOP_TYPE campaign_while_loop_type;




