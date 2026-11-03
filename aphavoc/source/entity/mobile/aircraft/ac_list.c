

































































#include "project.h"





#define DEBUG_MODULE 0






//
// LIST ROOT
//


#define LIST_TYPE_ROOT_FUNCTIONS_REQUIRED

#define LIST_TYPE_AIRCREW_ROOT

#define LIST_TYPE_TASK_DEPENDENT_ROOT

#define LIST_TYPE_LAUNCHED_WEAPON_ROOT

#define LIST_TYPE_MOVEMENT_DEPENDENT_ROOT


//
// LIST LINK
//


#define LIST_TYPE_LINK_FUNCTIONS_REQUIRED

#define LIST_TYPE_DESIGNATED_TARGET_LINK // Jabberwock 031107 Designated targets

#define LIST_TYPE_FOLLOWER_LINK

#define LIST_TYPE_GUNSHIP_TARGET_LINK

#define LIST_TYPE_MEMBER_LINK

#define LIST_TYPE_VIEW_LINK

#define LIST_TYPE_TAKEOFF_QUEUE_LINK

#define LIST_TYPE_MOVEMENT_DEPENDENT_LINK





#ifdef LIST_TYPE_ROOT_FUNCTIONS_REQUIRED

static void set_local_first_child (entity *en, list_types type, entity *first_child)
{
	aircraft
		*raw;

	raw = (aircraft *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/set_frst.h"
}

#endif





#ifdef LIST_TYPE_ROOT_FUNCTIONS_REQUIRED

static entity *get_local_first_child (entity *en, list_types type)
{
	aircraft
		*raw;

	entity
		*first_child;

	raw = (aircraft *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/get_frst.h"

	return (first_child);
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static void set_local_parent (entity *en, list_types type, entity *parent)
{
	aircraft
		*raw;

	raw = (aircraft *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/set_prnt.h"
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static entity *get_local_parent (entity *en, list_types type)
{
	aircraft
		*raw;

	entity
		*parent;

	raw = (aircraft *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/get_prnt.h"

	return (parent);
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static void set_local_child_succ (entity *en, list_types type, entity *child_succ)
{
	aircraft
		*raw;

	raw = (aircraft *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/set_succ.h"
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static entity *get_local_child_succ (entity *en, list_types type)
{
	aircraft
		*raw;

	entity
		*child_succ;

	raw = (aircraft *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/get_succ.h"

	return (child_succ);
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static void set_local_child_pred (entity *en, list_types type, entity *child_pred)
{
	aircraft
		*raw;

	raw = (aircraft *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/set_pred.h"
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static entity *get_local_child_pred (entity *en, list_types type)
{
	aircraft
		*raw;

	entity
		*child_pred;

	raw = (aircraft *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/get_pred.h"

	return (child_pred);
}

#endif





void overload_aircraft_list_functions (entity_types type)
{
	#define TYPE (type)

	#include "entity/system/en_funcs/en_list/fn_list.h"
}




