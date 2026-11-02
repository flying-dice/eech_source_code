/*
 * rotor_control.h -- types, constants and collaborator declarations of the TEST-ONLY numeric control of the original rotor sound routine (EEDCS-51, D3; ENV-HOST-ROTOR-CI-1). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
 *
 * The collaborators have the SHAPE of the engine's, not its bodies or layouts (see rotor_control_body.h):
 *   - get_local_entity_data is the engine's own direct macro ((EN)->data) (en_types.h);
 *   - the int-value, first-child, child-successor and float-value accessors are typed function-pointer-table dispatch macros indexed by the entity type and the value type, as in en_int.h, en_list.h and
 *     en_float.h (the validate_* range checks are omitted), the tables being filled at run time so that every call is an indirect call the compiler cannot resolve;
 *   - pause_local_entity_sound_type, resume_local_entity_sound_type and get_3d_vector_magnitude are plain external functions of another translation unit (soundeff.c, modules/maths/vector.c).
 * The collaborators that are functions get a private symbol prefix so that nothing here can collide with the engine's. Not part of the shipping host or module.
 */

#ifndef ROTOR_CONTROL_H
#define ROTOR_CONTROL_H

#define pause_local_entity_sound_type control_pause_sound
#define resume_local_entity_sound_type control_resume_sound
#define get_3d_vector_magnitude control_magnitude
#define current_flight_dynamics control_current_flight_dynamics

#define TRUE 1
#define FALSE 0

#define INT_TYPE_PLAYER 1
#define INT_TYPE_ENTITY_SUB_TYPE 2
#define FLOAT_TYPE_AMPLIFICATION 0
#define FLOAT_TYPE_SOUNDPITCH 1
#define LIST_TYPE_SPECIAL_EFFECT 7
#define ENTITY_PLAYER_AI 0
#define ENTITY_PLAYER_LOCAL 1
#define ENTITY_PLAYER_REMOTE 2
#define ENTITY_TYPE_HELICOPTER 20
#define ENTITY_TYPE_SOUND_EFFECT 32

/* the sub-types of en_sbtyp.h that the routine names (ENTITY_SUB_TYPE_EFFECT_SOUND_*) */
#define ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_LOOPING 33
#define ENTITY_SUB_TYPE_EFFECT_SOUND_APU_TURBINE 34
#define ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_TURBINE1 35
#define ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_TURBINE2 36
#define ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_SLAP 37
#define ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_WIND_DOWN1 38
#define ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_WIND_DOWN2 39
#define ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_WIND_UP1 40
#define ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_WIND_UP2 41

#define HELICOPTER_ROTOR_SOUNDS_LOOPING_MAX_RPM_VALUE	45
#define HELICOPTER_ROTOR_SOUNDS_LOOPING_MIN_RPM_VALUE	30

/* the same macro as the original's (modules/system/ or the env_ref kernel) */
#define bound(VALUE,LOWER,UPPER) ( ( VALUE ) < ( LOWER ) ? ( LOWER ) : ( ( VALUE ) > ( UPPER ) ? ( UPPER ) : ( VALUE ) ) )
#define ASSERT(CONDITION)
#define get_current_flight_dynamics() (current_flight_dynamics)

#define CONTROL_COHORT 29
#define CONTROL_SUB_TYPES 64
#define CONTROL_STAGES 16
#define CONTROL_ENTITY_TYPES 64
#define CONTROL_INT_TYPES 4
#define CONTROL_LIST_TYPES 8
#define CONTROL_FLOAT_TYPES 4

typedef struct vec3d
{
	float x, y, z;
} vec3d;

typedef struct control_float
{
	float value;
} control_float;

typedef struct dynamics_type
{
	vec3d model_acceleration_vector;
	control_float left_engine_n1_rpm, right_engine_n1_rpm, apu_rpm;
} dynamics_type;

typedef struct helicopter
{
	float main_rotor_rpm;
} helicopter;

typedef struct entity
{
	int type;
	int sub_type;
	int player;
	struct entity *next;
	void *data;
} entity;

/* the engine's accessors, in their shape */
#define get_local_entity_type(EN) ((EN)->type)
#define get_local_entity_data(EN) ((EN)->data)

extern int (*control_fn_get_int[CONTROL_ENTITY_TYPES][CONTROL_INT_TYPES]) (entity *en, int type);
extern entity *(*control_fn_first_child[CONTROL_ENTITY_TYPES][CONTROL_LIST_TYPES]) (entity *en, int type);
extern entity *(*control_fn_child_succ[CONTROL_ENTITY_TYPES][CONTROL_LIST_TYPES]) (entity *en, int type);
extern void (*control_fn_set_float[CONTROL_ENTITY_TYPES][CONTROL_FLOAT_TYPES]) (entity *en, int type, float value);

#define get_local_entity_int_value(EN,INT_TYPE) (control_fn_get_int[get_local_entity_type ((EN))][(INT_TYPE)] ((EN), (int)(INT_TYPE)))
#define get_local_entity_first_child(EN,LIST_TYPE) (control_fn_first_child[get_local_entity_type ((EN))][(LIST_TYPE)] ((EN), (LIST_TYPE)))
#define get_local_entity_child_succ(EN,LIST_TYPE) (control_fn_child_succ[get_local_entity_type ((EN))][(LIST_TYPE)] ((EN), (LIST_TYPE)))
#define set_local_entity_float_value(EN,FLOAT_TYPE,VALUE) (control_fn_set_float[get_local_entity_type ((EN))][(FLOAT_TYPE)] ((EN), (FLOAT_TYPE), (VALUE)))

extern dynamics_type *current_flight_dynamics;

void control_reset (void);
void control_written_words (unsigned int *to);
void control_stage_words (unsigned int *to);
void control_stage (int tag, float value);

int pause_local_entity_sound_type (entity *en, int sub_type, float delay);
int resume_local_entity_sound_type (entity *en, int sub_type);
float get_3d_vector_magnitude (const vec3d *vector);

#endif
