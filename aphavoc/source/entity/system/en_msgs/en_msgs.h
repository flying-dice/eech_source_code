

































































enum ENTITY_MESSAGES
{
   ENTITY_MESSAGE_ADJUST_LAND_URGENCY,                   		// (entity *receiver, entity *sender, float value)
   ENTITY_MESSAGE_ARTICULATE_LOADING_DOORS,                		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_ARTICULATE_UNDERCARRIAGE,                		// (entity *receiver, entity *sender)
	ENTITY_MESSAGE_CHECK_CAMPAIGN_OBJECTIVES,							// (entity *receiver, entity *sender)
	ENTITY_MESSAGE_CHECK_MOBILE_REACHED_GUIDE,						// (entity *receiver, entity *sender)
	ENTITY_MESSAGE_CHILD_KILLED,											// (entity *receiver, entity *sender)
	ENTITY_MESSAGE_CHILD_MOVE,												// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_COLLISION,                             		// (entity *receiver, entity *sender, float distance)
	ENTITY_MESSAGE_DECOY_ACTIVE,	  										// (entity *receiver, entity *sender)
	ENTITY_MESSAGE_DECOY_EXPIRED,		 									// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_DEPENDENT_CHILD_TASK_TERMINATED,					// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_DEPENDENT_PARENT_TASK_TERMINATED,				// (entity *receiver, entity *sender)
	ENTITY_MESSAGE_ENEMY_ACTIVITY,										// (entity *receiver, entity *sender)
	ENTITY_MESSAGE_ENTITY_FIRED_AT,										// (entity *receiver, entity *sender, entity *victim)
	ENTITY_MESSAGE_ENTITY_TARGETED,										// (entity *receiver, entity *sender, entity *victim)
	ENTITY_MESSAGE_ESCORT_TASK_START,									// (entity *receiver, entity *sender)
	ENTITY_MESSAGE_ESCORT_TASK_STOP,										// (entity *receiver, entity *sender)
	ENTITY_MESSAGE_FORCE_ARMOUR_ADVANCING,								// (entity *receiver, entity *sender, entity_sub_type type)
	ENTITY_MESSAGE_FORCE_ARMOUR_RESISTING,								// (entity *receiver, entity *sender, entity_sub_type type)
	ENTITY_MESSAGE_FORCE_ARMOUR_RETREATING,							// (entity *receiver, entity *sender, entity_sub_type type)
	ENTITY_MESSAGE_FORCE_ARMOUR_STUCK,									// (entity *receiver, entity *sender, entity_sub_type type)
	ENTITY_MESSAGE_FORCE_DESTROYED,										// (entity *receiver, entity *sender, entity *enemy)
	ENTITY_MESSAGE_FORCE_ENTERED_SECTOR,								// (entity *receiver, entity *sender, entity *enemy)
	ENTITY_MESSAGE_FORCE_INEFFICIENT_KEYSITE,							// (entity *receiver, entity *sender, entity *enemy)
	ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES,								// (entity *receiver, entity *sender, entity *enemy)
	ENTITY_MESSAGE_FORCE_SIDE_SURRENDERED,								// (entity *receiver, entity *sender, entity_sub_type type)
	ENTITY_MESSAGE_FORCE_SPECIAL_KILL,									// (entity *receiver, entity *sender, entity *enemy)
	ENTITY_MESSAGE_FORCE_VACANT_FARP,									// (entity *receiver, entity *sender, entity *enemy)
	ENTITY_MESSAGE_FORCE_WAYPOINT_REACHED,								// (entity *receiver, entity *sender, entity_sub_type type)
	ENTITY_MESSAGE_GROUND_FORCE_ADVANCE,								// (entity *receiver, entity *sender, int current_node, int destination_node)
	ENTITY_MESSAGE_GROUND_FORCE_RETREAT,								// (entity *receiver, entity *sender, int current_node, int destination_node)
   ENTITY_MESSAGE_GROUP_KILLED,   		                    		// (entity *receiver, entity *sender, int count)
   ENTITY_MESSAGE_GROUP_TAKEN_OFF,                       		// (entity *receiver, entity *sender, int count)
   ENTITY_MESSAGE_GUIDE_COVER_POSITION_REACHED,            		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_LANDING_SITE_REQUEST,                  		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_LINK_CHILD,                            		// (entity *receiver, entity *sender, list_types type)
   ENTITY_MESSAGE_LINK_PARENT,                           		// (entity *receiver, entity *sender, list_types type)
   ENTITY_MESSAGE_LOCK_LANDING_ROUTE,                    		// (entity *receiver, entity *sender, int count)
   ENTITY_MESSAGE_LOCK_LANDING_SITE,                     		// (entity *receiver, entity *sender, int count)
   ENTITY_MESSAGE_LOCK_TAKEOFF_ROUTE,                    		// (entity *receiver, entity *sender, int count)
   ENTITY_MESSAGE_MOBILE_KILLED,  			                 		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_PROTOTYPE_1,  		                      		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_PROTOTYPE_2,  		                      		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_PROTOTYPE_3,  		                      		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_PROTOTYPE_4,  		                      		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_REQUEST_ASSISTANCE,     	             		// (entity *receiver, entity *sender, entity *aggressor)
   ENTITY_MESSAGE_RESERVE_LANDING_SITE,                  		// (entity *receiver, entity *sender, int count)
   ENTITY_MESSAGE_SET_CAMERA_ACTION,                     		// (entity *receiver, entity *sender, camera_actions action)
   ENTITY_MESSAGE_SET_ENTITY_POSITION,                   		// (entity *receiver, entity *sender, vec3d *position, float heading, float velocity)
   ENTITY_MESSAGE_TASK_ASSIGNED,                           		// (entity *receiver, entity *sender)
	ENTITY_MESSAGE_TASK_COMPLETED,										// (entity *receiver, entity *sender, int task_completed)
	ENTITY_MESSAGE_TASK_CREATED,											// (entity *receiver, entity *sender, entity *enemy)
   ENTITY_MESSAGE_TASK_OBJECTIVE_MET,									// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_TASK_OBJECTIVE_FAILED,								// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_TASK_TERMINATED,										// (entity *receiver, entity *sender, int task_terminated)
   ENTITY_MESSAGE_UNLINK_CHILD,                          		// (entity *receiver, entity *sender, list_types type)
   ENTITY_MESSAGE_UNLINK_PARENT,                         		// (entity *receiver, entity *sender, list_types type)
   ENTITY_MESSAGE_UNLOCK_LANDING_ROUTE,                  		// (entity *receiver, entity *sender, int count)
   ENTITY_MESSAGE_UNLOCK_LANDING_SITE,                   		// (entity *receiver, entity *sender, int count)
   ENTITY_MESSAGE_UNLOCK_TAKEOFF_ROUTE,                  		// (entity *receiver, entity *sender, int count)
   ENTITY_MESSAGE_UNRESERVE_LANDING_SITE,                		// (entity *receiver, entity *sender, int count)
   ENTITY_MESSAGE_WAYPOINT_ACTION,  	                    		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_APPROACH_REACHED,               		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_ATTACK_ACTION,                 		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_CONVOY_REACHED,                 		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_DEFEND_REACHED,                 		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_DROP_OFF_REACHED,             		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_END_REACHED,                  		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_FINISH_DROP_OFF_REACHED,            // (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_FARP_RECON_REACHED,             		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_LAND_REACHED,                 		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_LANDED_REACHED,               		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_LIFT_OFF_REACHED,               		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_LOOP_REACHED,                 		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_LOWER_UNDERCARRIAGE_ACTION,    		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_NAVIGATION_REACHED,           		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_PREPARE_FOR_DROP_OFF_REACHED,			// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_PICK_UP_REACHED,              		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_RAISE_UNDERCARRIAGE_REACHED,    		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_REACHED,                      		// (entity *receiver, entity *sender, float sqr_range)
   ENTITY_MESSAGE_WAYPOINT_RECON_ACTION,                  		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_RECON_REACHED,                  		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_REPAIR_REACHED,                 		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_REVERSE_CONVOY_REACHED,         		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_START_UP_REACHED,					 		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_SUB_ROUTE_NAVIGATION_REACHED, 		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_TAKEN_OFF_REACHED,            		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_TAXI_REACHED,		            		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_TOUCH_DOWN_REACHED,            		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_TROOP_CAPTURE_REACHED,          		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_TROOP_DEFEND_REACHED,          		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_TROOP_EXTRACT_REACHED,          		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_TROOP_INSERT_REACHED,          		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_TROOP_PICKUP_POINT_END_REACHED,  	// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_TROOP_PICKUP_POINT_START_REACHED,   // (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_TROOP_PUTDOWN_POINT_REACHED,    		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_WAIT_REACHED,			          		// (entity *receiver, entity *sender)
   ENTITY_MESSAGE_WAYPOINT_PREPARE_FOR_INSERTION_REACHED,
   NUM_ENTITY_MESSAGES
};

typedef enum ENTITY_MESSAGES entity_messages;

extern const char
	*entity_message_names [];





#define ENTITY_MESSAGE_DONT_CARE (-1)





extern int notify_local_entity (entity_messages message, entity *receiver, entity *sender, ...);












