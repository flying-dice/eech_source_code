

































































#include "project.h"





#define DEBUG_MODULE 0






//
// LIST ROOT
//


#define LIST_TYPE_ROOT_FUNCTIONS_REQUIRED

#define LIST_TYPE_GUIDE_ROOT

#define LIST_TYPE_PLAYER_TASK_ROOT

#define LIST_TYPE_TASK_DEPENDENT_ROOT

#define LIST_TYPE_WAYPOINT_ROOT


//
// LIST LINK
//


#define LIST_TYPE_LINK_FUNCTIONS_REQUIRED

#define LIST_TYPE_PILOT_LOCK_LINK

#define LIST_TYPE_ASSIGNED_TASK_LINK

#define LIST_TYPE_COMPLETED_TASK_LINK

#define LIST_TYPE_SECTOR_TASK_LINK

#define LIST_TYPE_TASK_DEPENDENT_LINK

#define LIST_TYPE_UNASSIGNED_TASK_LINK

#define LIST_TYPE_UPDATE_LINK





#ifdef LIST_TYPE_ROOT_FUNCTIONS_REQUIRED

static void set_local_first_child (entity *en, list_types type, entity *first_child)
{
	task
		*raw;

	raw = (task *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/set_frst.h"
}

#endif





#ifdef LIST_TYPE_ROOT_FUNCTIONS_REQUIRED

static entity *get_local_first_child (entity *en, list_types type)
{
	task
		*raw;

	entity
		*first_child;

	raw = (task *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/get_frst.h"

	return (first_child);
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static void set_local_parent (entity *en, list_types type, entity *parent)
{
	task
		*raw;

	raw = (task *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/set_prnt.h"
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static entity *get_local_parent (entity *en, list_types type)
{
	task
		*raw;

	entity
		*parent;

	raw = (task *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/get_prnt.h"

	return (parent);
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static void set_local_child_succ (entity *en, list_types type, entity *child_succ)
{
	task
		*raw;

	raw = (task *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/set_succ.h"
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static entity *get_local_child_succ (entity *en, list_types type)
{
	task
		*raw;

	entity
		*child_succ;

	raw = (task *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/get_succ.h"

	return (child_succ);
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static void set_local_child_pred (entity *en, list_types type, entity *child_pred)
{
	task
		*raw;

	raw = (task *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/set_pred.h"
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static entity *get_local_child_pred (entity *en, list_types type)
{
	task
		*raw;

	entity
		*child_pred;

	raw = (task *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/get_pred.h"

	return (child_pred);
}

#endif





void overload_task_list_functions (void)
{
	#define TYPE (ENTITY_TYPE_TASK)

	#include "entity/system/en_funcs/en_list/fn_list.h"
}




