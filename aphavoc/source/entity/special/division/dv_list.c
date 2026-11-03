

































































#include "project.h"





#define DEBUG_MODULE 0






//
// LIST ROOT
//


#define LIST_TYPE_ROOT_FUNCTIONS_REQUIRED

#define LIST_TYPE_DIVISION_ROOT


//
// LIST LINK
//


#define LIST_TYPE_LINK_FUNCTIONS_REQUIRED

#define LIST_TYPE_DIVISION_LINK

#define LIST_TYPE_DIVISION_HEADQUARTERS_LINK





#ifdef LIST_TYPE_ROOT_FUNCTIONS_REQUIRED

static void set_local_first_child (entity *en, list_types type, entity *first_child)
{
	division
		*raw;

	raw = (division *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/set_frst.h"
}

#endif





#ifdef LIST_TYPE_ROOT_FUNCTIONS_REQUIRED

static entity *get_local_first_child (entity *en, list_types type)
{
	division
		*raw;

	entity
		*first_child;

	raw = (division *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/get_frst.h"

	return (first_child);
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static void set_local_parent (entity *en, list_types type, entity *parent)
{
	division
		*raw;

	raw = (division *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/set_prnt.h"
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static entity *get_local_parent (entity *en, list_types type)
{
	division
		*raw;

	entity
		*parent;

	raw = (division *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/get_prnt.h"

	return (parent);
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static void set_local_child_succ (entity *en, list_types type, entity *child_succ)
{
	division
		*raw;

	raw = (division *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/set_succ.h"
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static entity *get_local_child_succ (entity *en, list_types type)
{
	division
		*raw;

	entity
		*child_succ;

	raw = (division *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/get_succ.h"

	return (child_succ);
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static void set_local_child_pred (entity *en, list_types type, entity *child_pred)
{
	division
		*raw;

	raw = (division *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/set_pred.h"
}

#endif





#ifdef LIST_TYPE_LINK_FUNCTIONS_REQUIRED

static entity *get_local_child_pred (entity *en, list_types type)
{
	division
		*raw;

	entity
		*child_pred;

	raw = (division *) get_local_entity_data (en);

	#include "entity/system/en_funcs/en_list/get_pred.h"

	return (child_pred);
}

#endif





void overload_division_list_functions (void)
{
	#define TYPE (ENTITY_TYPE_DIVISION)

	#include "entity/system/en_funcs/en_list/fn_list.h"
}




