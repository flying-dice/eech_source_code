

































































enum PERSON_ANIMATIONS
{
	PERSON_ANIM_NONE,
	
	PERSON_ANIM_WALK,
	PERSON_ANIM_RUN,
	PERSON_ANIM_STAND1,
	PERSON_ANIM_STAND2,
	PERSON_ANIM_STAND3,
	PERSON_ANIM_STAND4,
	PERSON_ANIM_AIM_STANDING,
	PERSON_ANIM_SHOOT_STANDING,
	PERSON_ANIM_AIM_CROUCHING,
	PERSON_ANIM_SHOOT_CROUCHING,
	PERSON_ANIM_DEATH_STANDING1,
	PERSON_ANIM_DEATH_STANDING2,
	PERSON_ANIM_DEATH_CROUCHING1,
	PERSON_ANIM_DEATH_CROUCHING2,
	
	NUM_PERSON_ANIMS,
};

typedef enum PERSON_ANIMATIONS person_animations;





extern void animate_person (entity *en);

extern void update_person_animation (entity *en);

extern void damage_person_3d_object (entity *en);

extern float get_person_database_velocity (person_animations anim);




