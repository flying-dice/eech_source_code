

































































const char *operational_state_names [] =
{

	"OPERATIONAL_STATE_UNKNOWN",
	"Asleep",
	"Dead",
	"Dying",
	"Landed",
	"Landing",
	"Landing-holding",
	"Navigating",
	"Stopped",
	"Takeoff",
	"Takeoff-holding",
	"Taxiing",
	"Waiting",
	"NUM_OPERATIONAL_STATE_TYPES"
};





const char *verbose_operational_state_names [] =
{

	"Waiting", // actually unknown.
	"Air patrol",
	"Approaching target",
	"Approaching waypoint",
	"Awaiting landing clearance",
	"Awaiting takeoff clearance",
	"Engaging target",
	"En route",
	"En route to target",
	"Holding",
	"Landing",
	"Manoeuvring",
	"Performing Recon",
	"Performing Task",
	"Preparing for takeoff",
	"Rearming",
	"Refueling",
	"Repairing",
	"Returing to base",
	"Searching for target",
	"Take off",
	"Taxiing",
	"Waiting",
	"NUM_VERBOSE_OPERATIONAL_STATES"
};





const char *group_member_state_names [] =
{
	"Idle",
	"Waiting",
	"Taking Off",
	"Navigating",
	"Attacking",
	"Returning",
	"Landing",

	"NUM_GROUP_MEMBER_STATE_TYPES",
};





const char *keysite_usable_state_names [] =
{
	"Fully Operational",
	"Out Of Action",
	"Repairing",

	"NUM_KEYSITE_USABLE_STATE_TYPES"
};




