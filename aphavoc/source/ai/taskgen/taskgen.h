

































































#include "assign.h"

#include "croute.h"

#include "engage.h"





#define MAX_ROUTE_NODES 			(32)

#define TASK_ASSIGN_NO_MEMBERS	(0)
#define TASK_ASSIGN_ALL_MEMBERS	(0xffffffff)

#define TASK_THREAD 					0





extern vec3d
	terminator_point;





extern entity *create_task
						(
							entity_sub_types sub_type,
							entity_sides side,
							movement_types movement_type,
							entity *start_keysite,
							entity *end_keysite,
							entity *originator,
							int critical_task,
							float expire_timer,
							float stop_timer,
							entity *task_objective,
							float task_priority,
							...
						);

extern entity *create_anti_ship_strike_task (entity_sides side, entity *target, entity *originator, int critical, float priority, entity *start_keysite, entity *end_keysite);

extern entity *create_bai_task (entity_sides side, entity *target_group, entity *originator, int critical, float priority, entity *start_keysite, entity *end_keysite);

extern entity *create_barcap_task (entity_sides side, entity *this_keysite, entity *originator, int critical, vec3d *centre_position, float priority, float duration, entity *start_keysite, entity *end_keysite);

extern entity *create_bda_task (entity_sides side, entity *objective, entity *originator, int critical, float priority, entity *start_keysite, entity *end_keysite);

extern entity *create_cap_task (entity_sides side, entity *this_keysite, entity *originator, int critical, float priority, float duration, entity *start_keysite, entity *end_keysite);

extern entity *create_close_air_support_task (entity_sides side, entity *target_group, entity *originator, int critical, float priority, entity *start_keysite, entity *end_keysite);

extern entity *create_coastal_patrol_task (entity_sides side, vec3d *source_position, vec3d *destination_position, float priority, entity *start_keysite, entity *end_keysite);

extern entity *create_escort_task (entity *group, int critical, float priority, entity *start_keysite, entity *end_keysite);

extern entity *create_ground_force_task (entity_sub_types type, entity *group, int from_node, int to_node, float priority, entity *start_keysite, entity *end_keysite);

extern entity *create_ground_strike_task (entity_sides side, entity *target, entity *originator, int critical, float priority, entity *start_keysite, entity *end_keysite);

extern entity *create_oca_strike_task (entity_sides side, entity *target, entity *originator, int critical, float priority, entity *start_keysite, entity *end_keysite);

extern entity *create_oca_sweep_task (entity_sides side, entity *target, entity *originator, int critical, float priority, entity *start_keysite, entity *end_keysite);

extern entity *create_recon_task (entity_sides side, entity *objective, entity *originator, int critical, float priority, entity *start_keysite, entity *end_keysite);

extern entity *create_repair_task (entity_sides side, vec3d *pos, entity *objective, float priority, entity *start_keysite, entity *end_keysite);

extern entity *create_sead_task (entity_sides side, entity *target, entity *originator, int critical, float priority, entity *start_keysite, entity *end_keysite);

extern entity *create_supply_task (entity *requester, entity *supplier, entity *cargo, movement_types movement_type, float priority, entity *start_keysite, entity *end_keysite);

extern entity *create_transfer_task (entity_sides side, entity_sub_types task_type, float priority, entity *start_keysite, entity *end_keysite, int emergency);

extern entity *create_troop_insertion_task (entity_sides side, entity *destination_keysite, entity *originator, int critical, float priority, entity *start_keysite, entity *end_keysite);

extern entity *create_troop_movement_capture_task (entity_sides side, vec3d *start_pos, entity *insert_task, entity *keysite, entity *helicopter);

extern entity *create_troop_movement_defend_task (entity_sides side, vec3d *start_pos, vec3d *mid_pos, vec3d *end_pos, entity *insert_task, entity *keysite);

extern entity *create_troop_movement_patrol_task (entity_sides side, vec3d *start_pos, float radius, entity *keysite);

extern entity *create_user_task (entity *en, entity_sub_types type, vec3d *start_pos, entity *start_keysite, entity *end_keysite);

// creation tools

extern float get_estimated_task_duration (entity *task);

extern int validate_task_generation (entity_sides side, entity_sub_types sub_type);

extern void display_task_generation_stats (entity *force_en);







