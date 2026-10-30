// 
// 	 Enemy Engaged RAH-66 Comanche Versus KA-52 Hokum
// 	 Copyright (C) 2000 Empire Interactive (Europe) Ltd,
// 	 677 High Road, North Finchley, London N12 0DA
// 
// 	 Please see the document LICENSE.TXT for the full licence agreement
// 
// 2. LICENCE
//  2.1 	
//  	Subject to the provisions of this Agreement we now grant to you the 
//  	following rights in respect of the Source Code:
//   2.1.1 
//   	the non-exclusive right to Exploit  the Source Code and Executable 
//   	Code on any medium; and 
//   2.1.2 
//   	the non-exclusive right to create and distribute Derivative Works.
//  2.2 	
//  	Subject to the provisions of this Agreement we now grant you the
// 	following rights in respect of the Object Code:
//   2.2.1 
// 	the non-exclusive right to Exploit the Object Code on the same
// 	terms and conditions set out in clause 3, provided that any
// 	distribution is done so on the terms of this Agreement and is
// 	accompanied by the Source Code and Executable Code (as
// 	applicable).
// 
// 3. GENERAL OBLIGATIONS
//  3.1 
//  	In consideration of the licence granted in clause 2.1 you now agree:
//   3.1.1 
// 	that when you distribute the Source Code or Executable Code or
// 	any Derivative Works to Recipients you will also include the
// 	terms of this Agreement;
//   3.1.2 
// 	that when you make the Source Code, Executable Code or any
// 	Derivative Works ("Materials") available to download, you will
// 	ensure that Recipients must accept the terms of this Agreement
// 	before being allowed to download such Materials;
//   3.1.3 
// 	that by Exploiting the Source Code or Executable Code you may
// 	not impose any further restrictions on a Recipient's subsequent
// 	Exploitation of the Source Code or Executable Code other than
// 	those contained in the terms and conditions of this Agreement;
//   3.1.4 
// 	not (and not to allow any third party) to profit or make any
// 	charge for the Source Code, or Executable Code, any
// 	Exploitation of the Source Code or Executable Code, or for any
// 	Derivative Works;
//   3.1.5 
// 	not to place any restrictions on the operability of the Source 
// 	Code;
//   3.1.6 
// 	to attach prominent notices to any Derivative Works stating
// 	that you have changed the Source Code or Executable Code and to
// 	include the details anddate of such change; and
//   3.1.7 
//   	not to Exploit the Source Code or Executable Code otherwise than
// 	as expressly permitted by  this Agreement.
// 



/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#include "project.h"

#include "ai/taskgen/taskgen.h"
#include "ai/ai_misc/ai_route.h"

#include "entity/mobile/mb_cmbfx.h"

#include "entity/mobile/mb_hostaim.h"

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define DEBUG_MODULE 0

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static int response_to_child_killed (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	int
		start_node,
		end_node;

	entity_sub_types
		bridge_type;

	entity
		*sec;

	vec3d
		*position;

	bridge_type = get_local_entity_int_value (receiver, INT_TYPE_ENTITY_SUB_TYPE);

	#if DEBUG_MODULE >= 2

	debug_log_entity_message (message, receiver, sender, pargs);

	#endif

	CMB_FX_BRIDGE (EECH_CMB_BRIDGE_CHILD_KILLED, receiver, sender, get_local_entity_int_value (receiver, INT_TYPE_ALIVE), 0, 0.0f)

	start_node = get_local_entity_int_value (receiver, INT_TYPE_START_NODE);

	end_node = get_local_entity_int_value (receiver, INT_TYPE_END_NODE);

	#if DEBUG_MODULE

	debug_log ("BR_MSGS : Bridge Segment Killed ! Breaking link between nodes %d and %d", start_node, end_node);

	#endif

	//
	// Break route link
	//

	if (get_comms_model () == COMMS_MODEL_SERVER)
	{
		ASSERT (get_local_entity_type (sender) == ENTITY_TYPE_SEGMENT);
	
		if (get_local_entity_int_value (receiver, INT_TYPE_ALIVE))
		{

			int
				count;

			set_local_entity_int_value (receiver, INT_TYPE_ALIVE, FALSE);

			count = get_road_link_breaks (start_node, end_node);

			count ++;

			set_road_link_breaks (start_node, end_node, count);

			CMB_FX_BRIDGE (EECH_CMB_BRIDGE_LINK_BROKEN, receiver, sender, count, 0, 0.0f)

			debug_log ("BR_MSGS: Breaking bridge and route between %d and %d... broken bridge count = %d", count);
	
			//
			// Stop any vehicle on route
			//
	
			stop_vehicles_on_route (start_node, end_node);

			//
			// Create Repair task
			//

			position = get_local_entity_vec3d_ptr (sender, VEC3D_TYPE_POSITION);
				
			sec = get_local_sector_entity (position);

			ASSERT (sec);
			
			if (!create_repair_task ((entity_sides) get_local_entity_int_value (sec, INT_TYPE_SIDE), position, receiver, task_database [ENTITY_SUB_TYPE_TASK_REPAIR].task_priority, NULL, NULL))
			{
				debug_log ("BR_MSGS: can't create repair mission");

				CMB_FX_BRIDGE (EECH_CMB_BRIDGE_REPAIR_TASK, receiver, sender, 0, 0, 0.0f)
			}
			else
			{
				debug_log ("BR_MSGS: created repair mission to [%f, %f, %f]", position->x, position->y, position->z);

				CMB_FX_BRIDGE (EECH_CMB_BRIDGE_REPAIR_TASK, receiver, sender, 1, 0, 0.0f)
			}
		}
	}

	//
	// insert segment into the update list ( so that it can fall to the floor etc. )
	//

	insert_local_entity_into_parents_child_list (sender, LIST_TYPE_UPDATE, get_update_entity (), NULL);

	CMB_FX_BRIDGE (EECH_CMB_BRIDGE_UPDATE_INSERTED, receiver, sender, 0, 0, 0.0f)

	return (TRUE);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// N.B. This function called on both SERVER and CLIENT

static int response_to_collision (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	entity
		*pred,
		*succ;

	entity_sub_types
		sub_type;

	//
	// Special case for bridge collision ( actual collision is handled by segments )
	//
	// This fn called by a segment when either of its neighbours have been killed, or it has been hit but not killed
	//

	sub_type = get_local_entity_int_value (sender, INT_TYPE_ENTITY_SUB_TYPE);

	CMB_FX_BRIDGE (EECH_CMB_BRIDGE_COLLISION, receiver, sender, sub_type, get_local_entity_int_value (sender, INT_TYPE_ALIVE), 0.0f)

	//
	// Whether the segment collapses NOW is the host's physical decision (EEDCS-46, R5, mb_hostaim.h BRIDGE, KILL): the segment's sub type, whether it is alive and whether this is a server are sent as they
	// are.  The reference host's answer is the original's: a living unsupported mid section of a server has nothing to support it, so it is killed.  The kill and its effects are the campaign's.
	//

	if (campaign_aim_bridge_kill (sub_type, get_local_entity_int_value (sender, INT_TYPE_ALIVE), get_comms_model() == COMMS_MODEL_SERVER))
	{
		CMB_FX_BRIDGE (EECH_CMB_BRIDGE_UNSUPPORTED_KILL, receiver, sender, 0, 0, 0.0f)

		kill_client_server_entity (sender);

		create_client_server_object_killed_explosion_effect (sender);
	}

	//
	// change object shape ( depending on state of its neighbours )
	//

	pred = get_local_entity_child_pred (sender, LIST_TYPE_SEGMENT);

	succ = get_local_entity_child_succ (sender, LIST_TYPE_SEGMENT);

	set_local_entity_int_value (sender, INT_TYPE_OBJECT_3D_SHAPE, get_bridge_segment_damaged_object (sender, pred, succ));

	CMB_FX_BRIDGE (EECH_CMB_BRIDGE_SHAPE, receiver, sender, get_local_entity_int_value (sender, INT_TYPE_OBJECT_3D_SHAPE), 2 * ((pred == NULL) || get_local_entity_int_value (pred, INT_TYPE_ALIVE)) + ((succ == NULL) || get_local_entity_int_value (succ, INT_TYPE_ALIVE)), 0.0f)

	return (TRUE);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#if DEBUG_MODULE

static int response_to_link_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#if DEBUG_MODULE

static int response_to_unlink_child (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#if DEBUG_MODULE

static int response_to_link_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#if DEBUG_MODULE

static int response_to_unlink_parent (entity_messages message, entity *receiver, entity *sender, va_list pargs)
{
	debug_log_entity_message (message, receiver, sender, pargs);

	return (TRUE);
}

#endif

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void overload_bridge_message_responses (void)
{
	message_responses[ENTITY_TYPE_BRIDGE][ENTITY_MESSAGE_CHILD_KILLED] 			= response_to_child_killed;

	message_responses[ENTITY_TYPE_BRIDGE][ENTITY_MESSAGE_COLLISION] 				= response_to_collision;

	#if DEBUG_MODULE

	message_responses[ENTITY_TYPE_BRIDGE][ENTITY_MESSAGE_LINK_CHILD]				= response_to_link_child;

	message_responses[ENTITY_TYPE_BRIDGE][ENTITY_MESSAGE_UNLINK_CHILD]			= response_to_unlink_child;

	message_responses[ENTITY_TYPE_BRIDGE][ENTITY_MESSAGE_LINK_PARENT]				= response_to_link_parent;

	message_responses[ENTITY_TYPE_BRIDGE][ENTITY_MESSAGE_UNLINK_PARENT]			= response_to_unlink_parent;

	#endif
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
