

































































enum ENTITY_SIDES
{
	ENTITY_SIDE_NEUTRAL,
	ENTITY_SIDE_BLUE_FORCE,
	ENTITY_SIDE_RED_FORCE,
	NUM_ENTITY_SIDES
};

typedef enum ENTITY_SIDES entity_sides;





#define entity_side_valid(SIDE) (((SIDE) >= 0) && ((SIDE) < NUM_ENTITY_SIDES))





#define ENTITY_SIDE_UNINITIALISED	(NUM_ENTITY_SIDES)





extern const char *entity_side_names [];

extern const char *entity_side_short_names [];





#define get_enemy_side(SIDE) (((SIDE) == ENTITY_SIDE_BLUE_FORCE) ? ENTITY_SIDE_RED_FORCE : ENTITY_SIDE_BLUE_FORCE)




