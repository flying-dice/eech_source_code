

































































enum MOBILE_LOS_CHECK_CRITERIA
{
	MOBILE_LOS_CHECK_NONE						= 0,

	MOBILE_LOS_CHECK_COURSE_TERRAIN			= (1 << 0),
	MOBILE_LOS_CHECK_SOURCE_END_TERRAIN		= (1 << 1),
	MOBILE_LOS_CHECK_TARGET_END_TERRAIN		= (1 << 2),
	MOBILE_LOS_CHECK_SOURCE_END_OBJECTS		= (1 << 3),
	MOBILE_LOS_CHECK_TARGET_END_OBJECTS		= (1 << 4),

	NUM_MOBILE_LOS_CHECKS						= (1 << 5)
};

typedef enum MOBILE_LOS_CHECK_CRITERIA mobile_los_check_criteria;

#define MOBILE_LOS_CHECK_ALL	((mobile_los_check_criteria)(NUM_MOBILE_LOS_CHECKS - 1))





extern int check_entity_line_of_sight (entity *source, entity *target, mobile_los_check_criteria criteria);

extern int check_position_line_of_sight (entity *source, entity *target, vec3d *source_position, vec3d *target_position, mobile_los_check_criteria criteria);






