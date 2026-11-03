

































































#define AI_ESCORT_TASK_GENERATION_POWER_THRESH_HOLD 2





enum STRATEGY_TYPES
{

   STRATEGY_TYPE_INVALID,
   STRATEGY_TYPE_RETREAT,
   STRATEGY_TYPE_DEFEND,
   STRATEGY_TYPE_ADVANCE,
   NUM_STRATEGY_TYPES
};

typedef enum STRATEGY_TYPES strategy_types;

extern const char *strategy_names [];





enum GROUP_MODE_TYPES
{
   GROUP_MODE_IDLE,
   GROUP_MODE_BUSY,
   NUM_GROUP_MODE_TYPES
};

typedef enum GROUP_MODE_TYPES group_mode_types;

extern const char *group_mode_names [];





enum MOVEMENT_TYPES
{
   MOVEMENT_TYPE_NONE,
   MOVEMENT_TYPE_AIR,
   MOVEMENT_TYPE_GROUND,
   MOVEMENT_TYPE_SEA,
   MOVEMENT_TYPE_ALL,
   NUM_MOVEMENT_TYPES
};

typedef enum MOVEMENT_TYPES movement_types;

extern const char *movement_names [];





enum AI_SUITABILITY_TYPES
{
   AI_SUITABILITY_AIR_ATTACK_STRENGTH,
   AI_SUITABILITY_GROUND_ATTACK_STRENGTH,
   AI_SUITABILITY_MOVEMENT_SPEED,
   AI_SUITABILITY_MOVEMENT_STEALTH,
   AI_SUITABILITY_CARGO_SPACE,
   AI_SUITABILITY_TROOP_SPACE,
   NUM_AI_SUITABILITY_TYPES
};

typedef enum AI_SUITABILITY_TYPES ai_suitability_types;

extern const char *ai_suitability_type_names [];





enum TASK_URGENCY_TYPES
{

	TASK_URGENCY_0,
	TASK_URGENCY_1,
	TASK_URGENCY_2,
	TASK_URGENCY_3,
	TASK_URGENCY_4,
	TASK_URGENCY_5,
	TASK_URGENCY_6,
	TASK_URGENCY_7,
	TASK_URGENCY_8,
	TASK_URGENCY_9,
	TASK_URGENCY_10,
};

typedef enum TASK_URGENCY_TYPES task_urgency_types;





struct AI_STATISTICS
{

   union
	{
	
  		struct
		{

			int
				air_attack_strength,
				ground_attack_strength,
				movement_speed,
				movement_stealth,
				cargo_space,
				troop_space;
		};

		int
			stats [NUM_AI_SUITABILITY_TYPES];
   };
};

typedef struct AI_STATISTICS ai_statistics;





enum FRONTLINE_SECTOR_TYPES
{

	FRONTLINE_SECTOR_NONE,
	FRONTLINE_SECTOR_PRIMARY,
	NUM_FRONTLINE_SECTOR_TYPES
};

typedef enum FRONTLINE_SECTOR_TYPES frontline_sector_types;




