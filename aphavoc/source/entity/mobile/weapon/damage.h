

































































#define WARHEAD_EFFECTIVE_NONE						0

//
// soft targets
//

#define WARHEAD_EFFECTIVE_SOFT_TARGETS				1

#define WARHEAD_EFFECTIVE_HELICOPTER				1
#define WARHEAD_EFFECTIVE_FIXED_WING				2

#define WARHEAD_EFFECTIVE_INFANTRY					4
#define WARHEAD_EFFECTIVE_LIGHT_VEHICLE			8
#define WARHEAD_EFFECTIVE_LIGHT_SHIP				16

#define WARHEAD_EFFECTIVE_WOOD_BRIDGE				32
#define WARHEAD_EFFECTIVE_STONE_BRIDGE				64

#define WARHEAD_EFFECTIVE_LIGHT_BUILDING			128
#define WARHEAD_EFFECTIVE_STONE_BUILDING			256

//
// hard targets
//

#define WARHEAD_EFFECTIVE_HARD_TARGETS				512

#define WARHEAD_EFFECTIVE_ARMOURED_VEHICLE		512

#define WARHEAD_EFFECTIVE_FRIGATE					1024
#define WARHEAD_EFFECTIVE_CARRIER					2048

#define WARHEAD_EFFECTIVE_METAL_BRIDGE				4096

#define WARHEAD_EFFECTIVE_HARDENED_BUILDING		8192





extern int damage_client_server_entity (entity *en, entity *weapon, float damage_modifier);

extern void display_debug_kill_info (entity *victim, entity *aggressor);

extern int weapon_damage_capability (entity *wpn, entity *target, float damage_modifier, entity_sub_types weapon_type, float velocity);

extern void initialise_destroyed_object_database (void);

extern void deinitialise_destroyed_object_database (void);

extern object_3d_index_numbers get_3d_object_destroyed_object_index (object_3d_index_numbers index);

extern object_3d_index_numbers get_3d_object_restored_object_index (object_3d_index_numbers index);

extern int get_damage_debug_invulnerable_flag (void);

extern int toggle_damage_debug_invulnerable_flag (void);




