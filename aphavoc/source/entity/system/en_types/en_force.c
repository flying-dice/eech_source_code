

































































#include "project.h"





const char
	*entity_force_action_names [] =
	{
	
		"FORCE_ACTION_INVALID",
		"FORCE_ACTION_RETREAT",
		"FORCE_ACTION_NOTHING",
		"FORCE_ACTION_RECON",
		"FORCE_ACTION_INTERCEPT",
		"FORCE_ACTION_DEEP_STRIKE",
		"NUM_ENTITY_FORCE_ACTIONS"
	};





const char
	*entity_force_attitude_names [] =
	{
		"FORCE_ATTITUDE_COWARD",
		"FORCE_ATTITUDE_PASSIVE",
		"FORCE_ATTITUDE_CAUTIOUS",
		"FORCE_ATTITUDE_NORMAL",
		"FORCE_ATTITUDE_FEISTY",
		"FORCE_ATTITUDE_AGGRESSIVE",
		"FORCE_ATTITUDE_DESTRUCTIVE",
		"FORCE_ATTITUDE_ERRATIC",
		"NUM_ENTITY_FORCE_ATTITUDES"
	};





const char
	*entity_force_names [] =
	{
		"FORCE_UNKNOWN",
		"FORCE_AIR",
		"FORCE_GROUND",
		"FORCE_SEA",
		"NUM_ENTITY_FORCES"
	};





const char
	*campaign_result_names [] =
	{
	
		"NONE",
		"FAIL",
		"SUCCESS",
		"STALEMATE",
	
		"NUM_CAMPAIGN_RESULT_TYPES"
	};





const char
	*campaign_trigger_names [] =
	{

  		"NONE",
		"BALANCE_OF_POWER",
		"TASK_COMPLETED",
		"TASK_FAILED",
		"OBJECT_DESTROYED",
		"OBJECT_FIRED",
		"OBJECT_TARGETED",
		"OBJECT_LANDED",
		"INEFFICIENT_KEYSITE",
		"WAYPOINT_REACHED",
		"SECTOR_WON",
		"SECTOR_LOST",
		"SECTOR_REACHED",
		"TIME_DURATION",
		"VARIABLE_CONDITION",
		"RANDOM",
		"USER_LANDED",
		"KEY_PRESS",
	
		"NUM_CAMPAIGN_TRIGGERS"
	};





const char
	*force_info_catagory_names [] =
	{
	
		"ARMED_FIXED_WING",
		"UNARMED_FIXED_WING",
		"ARMED_HELICOPTER",
		"UNARMED_HELICOPTER",
		"ARMED_ROUTED_VEHICLE",
		"UNARMED_ROUTED_VEHICLE",
		"ARMED_SHIP_VEHICLE",
		"UNARMED_SHIP_VEHICLE",
		"NUM_FORCE_INFO_CATAGORIES"
	};




