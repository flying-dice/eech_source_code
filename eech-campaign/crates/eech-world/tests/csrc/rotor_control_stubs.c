/*
 * rotor_control_stubs.c -- the collaborators of the TEST-ONLY numeric control of the original rotor sound routine (EEDCS-51, D3; ENV-HOST-ROTOR-CI-1). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
 *
 * In the engine the routine calls other translation units: the entity accessors (through typed function-pointer tables), the pause and resume helpers of soundeff.c and the vector magnitude of
 * modules/maths/vector.c. Here they are another translation unit too, so the compiler sees an opaque call at each of them, as it does in the original: floats live across a call are spilled and a float
 * argument is passed as a float. The bodies are STUBS (the reduced entity layout is documented in rotor_control_body.h); they record what they are given as 32-bit words (a float is only stored or copied
 * here, never computed on), except the vector magnitude, whose expression is copied from modules/maths/vector.c.
 *
 * Not part of the shipping host or module: it is compiled for the test of eech-world only (build.rs), and nothing in the host references it.
 */

#include <math.h>
#include <string.h>

#include "rotor_control.h"

dynamics_type *current_flight_dynamics;

int (*control_fn_get_int[CONTROL_ENTITY_TYPES][CONTROL_INT_TYPES]) (entity *en, int type);
entity *(*control_fn_first_child[CONTROL_ENTITY_TYPES][CONTROL_LIST_TYPES]) (entity *en, int type);
entity *(*control_fn_child_succ[CONTROL_ENTITY_TYPES][CONTROL_LIST_TYPES]) (entity *en, int type);
void (*control_fn_set_float[CONTROL_ENTITY_TYPES][CONTROL_FLOAT_TYPES]) (entity *en, int type, float value);

/* the children of the helicopter the control walks: the cohort of the measured original (the sub-types in the order the measured record lists them, the unhandled ones, the winding group, then the looping, slap,
   turbine and APU group, the second group again, and the children that are not sound effects) */
static entity control_cohort[CONTROL_COHORT];
static const int control_cohort_types[CONTROL_COHORT] = {
	32, 32, 32, 32, 32,
	32, 32, 32, 32, 32, 32, 32, 32, 32,
	32, 32, 32, 32, 32, 32, 32, 32, 32,
	31, 31, 31, 31, 31, 31
};
static const int control_cohort_subs[CONTROL_COHORT] = {
	25, 30, 31, 26, 27,
	39, 38, 41, 40, 37, 34, 36, 35, 33,
	39, 38, 41, 40, 37, 34, 36, 35, 33,
	0, 0, 0, 0, 0, 0
};

static float control_written[CONTROL_SUB_TYPES][2];
static float control_stages[CONTROL_STAGES];
static float control_sentinel;

static int control_stub_get_int (entity *en, int what)
{
	if (what == INT_TYPE_PLAYER)
	{
		return en->player;
	}

	return en->sub_type;
}

static entity *control_stub_first_child (entity *en, int list)
{
	(void) en;
	(void) list;

	return &control_cohort[0];
}

static entity *control_stub_child_succ (entity *en, int list)
{
	(void) list;

	return en->next;
}

static void control_stub_set_float (entity *en, int what, float value)
{
	if (en->sub_type >= 0 && en->sub_type < CONTROL_SUB_TYPES && what >= 0 && what < 2)
	{
		control_written[en->sub_type][what] = value;
	}
}

void control_reset (void)
{
	int type, i;

	memset (control_written, 0xde, sizeof (control_written));
	memset (control_stages, 0xde, sizeof (control_stages));

	/* the tables are filled at run time: every call through them is an indirect call */
	for (type = 0; type < CONTROL_ENTITY_TYPES; type++)
	{
		for (i = 0; i < CONTROL_INT_TYPES; i++)
		{
			control_fn_get_int[type][i] = control_stub_get_int;
		}

		for (i = 0; i < CONTROL_LIST_TYPES; i++)
		{
			control_fn_first_child[type][i] = control_stub_first_child;
			control_fn_child_succ[type][i] = control_stub_child_succ;
		}

		for (i = 0; i < CONTROL_FLOAT_TYPES; i++)
		{
			control_fn_set_float[type][i] = control_stub_set_float;
		}
	}

	for (i = 0; i < CONTROL_COHORT; i++)
	{
		control_cohort[i].type = control_cohort_types[i];
		control_cohort[i].sub_type = control_cohort_subs[i];
		control_cohort[i].player = 0;
		control_cohort[i].next = (i + 1 < CONTROL_COHORT) ? &control_cohort[i + 1] : 0;
		control_cohort[i].data = 0;
	}
}

void control_written_words (unsigned int *to)
{
	int channel;
	static const int subs[6] = { 33, 37, 35, 36, 34, 38 };

	for (channel = 0; channel < 6; channel++)
	{
		memcpy (&to[channel], &control_written[subs[channel]][0], sizeof (float));
		memcpy (&to[6 + channel], &control_written[subs[channel]][1], sizeof (float));
	}
}

void control_stage_words (unsigned int *to)
{
	memcpy (to, control_stages, sizeof (control_stages));
}

int pause_local_entity_sound_type (entity *en, int sub_type, float delay)
{
	(void) en;
	control_sentinel = delay;

	return sub_type;
}

int resume_local_entity_sound_type (entity *en, int sub_type)
{
	(void) en;

	return sub_type;
}

void control_stage (int tag, float value)
{
	if (tag >= 0 && tag < CONTROL_STAGES)
	{
		control_stages[tag] = value;
	}
}

float get_3d_vector_magnitude (const vec3d *vector)
{
	float
		x,
		y,
		z;

	x = (vector->x * vector->x);
	y = (vector->y * vector->y);
	z = (vector->z * vector->z);

	return (sqrt (x + y + z));
}
