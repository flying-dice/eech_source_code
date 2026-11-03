

































































#include "project.h"





const char *task_status_names [] =
{
	"In Progress",
	"Failed",
	"Completed",
	"Completed",

	"NUM_TASK_COMPLETED_TYPES"
};





const char *task_debrief_result_names [] =
{
	"In Progress",
	"Failed",
	"Partially Completed",
	"Successfully Completed",

	"NUM_TASK_COMPLETED_TYPES"
};





const char *task_terminated_names [] =
{

	"TASK_TERMINATED_IN_PROGRESS",
	"TASK_TERMINATED_EXPIRE_TIME_REACHED",
	"TASK_TERMINATED_ABORTED",
	"TASK_TERMINATED_GROUP_DESTROYED",
	"TASK_TERMINATED_OBJECTIVE_MESSAGE",
	"TASK_TERMINATED_STOP_TIME_REACHED",
	"TASK_TERMINATED_WAYPOINT_ROUTE_COMPLETE",

	"NUM_TASK_TERMINATED_TYPES"
};





const char *task_target_source_names [] =
{

	"TASK_TARGET_SOURCE_SCAN_AIR",
	"TASK_TARGET_SOURCE_SCAN_ALL",
	"TASK_TARGET_SOURCE_SCAN_GROUND",
	"TASK_TARGET_SOURCE_OBJECTIVE_NOMINATED",
	"TASK_TARGET_SOURCE_TASK_OBJECTIVE",
	"TASK_TARGET_SOURCE_NONE",

	"NUM_TASK_TASK_TARGET_SOURCE_TYPES"
};




