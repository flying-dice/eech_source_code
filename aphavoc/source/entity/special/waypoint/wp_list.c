

































































#include "project.h"





#define DEBUG_MODULE 0






//
// LIST ROOT
//


#define LIST_TYPE_ROOT_FUNCTIONS_REQUIRED

#define LIST_TYPE_CURRENT_WAYPOINT_ROOT


//
// LIST LINK
//


#define LIST_TYPE_LINK_FUNCTIONS_REQUIRED

#define LIST_TYPE_TASK_DEPENDENT_LINK

//#define LIST_TYPE_SECTOR_LINK

#define LIST_TYPE_WAYPOINT_LINK





#ifdef LIST_TYPE_ROOT_FUNCTIONS_REQUIRED

static void set_local_first_child (entity *en, list_types type, entity *first_child)
{
	waypoint
		*raw;

	raw = (waypoint *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/set_frst.h"
}

#endif





#ifdef LIST_TYPE_ROOT_FUNCTIONS_REQUIRED

static entity *get_local_first_child (entity *en, list_types type)
{
	waypoint
		*raw;

	entity
		*first_child;

	raw = (waypoint *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/get_frst.h"

	return (first_child);
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static void set_local_parent (entity *en, list_types type, entity *parent)
{
	waypoint
		*raw;

	raw = (waypoint *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/set_prnt.h"
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static entity *get_local_parent (entity *en, list_types type)
{
	waypoint
		*raw;

	entity
		*parent;

	raw = (waypoint *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/get_prnt.h"

	return (parent);
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static void set_local_child_succ (entity *en, list_types type, entity *child_succ)
{
	waypoint
		*raw;

	raw = (waypoint *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/set_succ.h"
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static entity *get_local_child_succ (entity *en, list_types type)
{
	waypoint
		*raw;

	entity
		*child_succ;

	raw = (waypoint *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/get_succ.h"

	return (child_succ);
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static void set_local_child_pred (entity *en, list_types type, entity *child_pred)
{
	waypoint
		*raw;

	raw = (waypoint *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/set_pred.h"
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static entity *get_local_child_pred (entity *en, list_types type)
{
	waypoint
		*raw;

	entity
		*child_pred;

	raw = (waypoint *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/get_pred.h"

	return (child_pred);
}

#endif





void overload_waypoint_list_functions (void)
{
	#define TYPE (ENTITY_TYPE_WAYPOINT)

	#include "entity/system/en_funcs/en_list/fn_list.h"
}




