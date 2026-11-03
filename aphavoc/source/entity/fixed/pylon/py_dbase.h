

































































typedef enum PYLON_TYPES
{

	PYLON_TYPE_STANDARD,
	PYLON_TYPE_START_TERMINATOR,
	PYLON_TYPE_END_TERMINATOR,

	NUM_PYLON_TYPES

} pylon_types;





struct PYLON_ATTACHMENT_DATA
{
	object_3d_index_numbers
		default_object_3d_shape;

	char
		attachment_levels[ 6 ];

	vec3d
		attachment_offset[ 6 ];
};

typedef struct PYLON_ATTACHMENT_DATA pylon_attachment_data;


#define pylon_default_object_3d_shape(a) pylon_attachment_database[(a)].default_object_3d_shape

#define pylon_attachment_offset(a,b) pylon_attachment_database[(a)].attachment_offset[(b)]

#define pylon_attachment_level(a,b) pylon_attachment_database[(a)].attachment_levels[(b)]



#define CABLE_APPROXIMATION_DISTANCE_1		800.0

#define CABLE_APPROXIMATION_DISTANCE_2		1600.0

#define CABLE_APPROXIMATION_DISTANCE_3		4000.0

#define NUM_PYLON_ATTACHMENT_POINTS			6





extern pylon_attachment_data
	pylon_attachment_database[NUM_PYLON_TYPES];



extern void initialise_pylon_database();

extern float get_pylon_sag_value( int index );

extern pylon_types get_pylon_sub_type( entity_sub_types sub_type );




