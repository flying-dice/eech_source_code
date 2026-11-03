

































































typedef enum
{
	BRIDGE_SEGMENT_TYPE_NONE,

	BOAT_BRIDGE_START,
	BOAT_BRIDGE_END,
	BOAT_BRIDGE_MID,
	BOAT_BRIDGE_DECK,

	CAUSEWAY_BRIDGE_START,
	CAUSEWAY_BRIDGE_END,
	CAUSEWAY_BRIDGE_MID,

	CONCRETE_ARCH_BRIDGE_START,
	CONCRETE_ARCH_BRIDGE_END,
	CONCRETE_ARCH_BRIDGE_DECK,

	CONCRETE_WIDE_BRIDGE_START,
	CONCRETE_WIDE_BRIDGE_END,
	CONCRETE_WIDE_BRIDGE_DECK,

	DESERT_BRIDGE_START,
	DESERT_BRIDGE_END,
	DESERT_BRIDGE_MID,

	GIRDER_BRIDGE_START,
	GIRDER_BRIDGE_END,
	GIRDER_BRIDGE_MID,
	GIRDER_BRIDGE_DECK,

	STONE_BRIDGE_START,
	STONE_BRIDGE_END,
	STONE_BRIDGE_MID,

	SUSPENSION_BRIDGE_START,
	SUSPENSION_BRIDGE_END,
	SUSPENSION_BRIDGE_MID,
	SUSPENSION_BRIDGE_DECK,

	NUM_BRIDGE_SEGMENT_TYPES

} bridge_segment_types;





struct BRIDGE_SEGMENT_DATA
{

	object_3d_index_numbers
		normal_object,
		destroyed_object,
		pred_destroyed_object,
		succ_destroyed_object,
		both_destroyed_object;

	entity_sub_types
		sub_type;

	float
		length;

	int
		initial_damage_level,
		target_type,
		armor_level;
};

typedef struct BRIDGE_SEGMENT_DATA bridge_segment_data;





extern bridge_segment_data
	segment_database[NUM_BRIDGE_SEGMENT_TYPES];

#define bridge_segment_normal_object(a) segment_database[(a)].normal_object

#define bridge_segment_length(a) segment_database[(a)].length

#define bridge_segment_sub_type(a) segment_database[(a)].sub_type

#define bridge_segment_initial_damage_level(a) segment_database[(a)].initial_damage_level





extern void initialise_segment_database();








