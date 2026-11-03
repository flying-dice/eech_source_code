

































































enum ENTITY_ATTRIBUTES
{
	entity_attr_end,
	entity_attr_attitude_angles,
	entity_attr_char_value,
	entity_attr_child_pred,
	entity_attr_float_value,
	entity_attr_int_value,
	entity_attr_parent,
	entity_attr_ptr_value,
	entity_attr_string,
	entity_attr_vec3d,
	NUM_ENTITY_ATTRIBUTES
};

typedef enum ENTITY_ATTRIBUTES entity_attributes;





#define NUM_ENTITY_ATTRIBUTE_PACK_BITS	(4)





struct ENTITY_ATTRIBUTE_DATA
{
	char
		*name;

	int
		debug_pack;
};

typedef struct ENTITY_ATTRIBUTE_DATA entity_attribute_data;





extern entity_attribute_data
	entity_attribute_database[NUM_ENTITY_ATTRIBUTES];

#define get_entity_attribute_name(ATTR) (entity_attribute_database[(ATTR)].name)





#define ENTITY_ATTR_END entity_attr_end

#define ENTITY_ATTR_ATTITUDE_ANGLES(HEADING,PITCH,ROLL) entity_attr_attitude_angles, ((double) (HEADING)), ((double) (PITCH)), ((double) (ROLL))

#define ENTITY_ATTR_CHAR_VALUE(CHAR_TYPE,VALUE) entity_attr_char_value, ((char_types) (CHAR_TYPE)), ((char) (VALUE))

#define ENTITY_ATTR_CHILD_PRED(LIST_TYPE,CHILD_PRED) entity_attr_child_pred, ((list_types) (LIST_TYPE)), ((entity *) (CHILD_PRED))

#define ENTITY_ATTR_FLOAT_VALUE(FLOAT_TYPE,VALUE) entity_attr_float_value, ((float_types) (FLOAT_TYPE)), ((float) (VALUE))

#define ENTITY_ATTR_INT_VALUE(INT_TYPE,VALUE) entity_attr_int_value, ((int_types) (INT_TYPE)), ((int) (VALUE))

#define ENTITY_ATTR_PARENT(LIST_TYPE,PARENT) entity_attr_parent, ((list_types) (LIST_TYPE)), ((entity *) (PARENT))

#define ENTITY_ATTR_PTR_VALUE(PTR_TYPE,PTR) entity_attr_ptr_value, ((ptr_types) (PTR_TYPE)), ((void *) (PTR))

#define ENTITY_ATTR_STRING(STRING_TYPE,STRING) entity_attr_string, ((string_types) (STRING_TYPE)), ((char *) (STRING))

#define ENTITY_ATTR_VEC3D(VEC3D_TYPE,X,Y,Z) entity_attr_vec3d, ((vec3d_types) (VEC3D_TYPE)), ((float) (X)), ((float) (Y)), ((float) (Z))





extern void set_local_entity_attributes (entity *en, char *pargs);












