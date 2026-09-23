/*
 * INVESTIGATION ONLY (issue #7), Investigation 6: the original expressions of
 * Slices 1-3, isolated, run under a given x87 control word / SSE rounding.
 *
 *   probes <x87 control word, hex> <mxcsr rounding 0-3> <probe> <args...>
 *
 * Each probe copies its expression verbatim from the cited original file and
 * prints the float results as IEEE 754 single bit patterns. Arguments are
 * parsed (and narrowed to float) before the control word is installed.
 */

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define max(a,b) (((a) > (b)) ? (a) : (b))
#define bound(VALUE,LOWER,UPPER) ((VALUE) < (LOWER) ? (LOWER) : ((VALUE) > (UPPER) ? (UPPER) : (VALUE)))
#define AMMO_USAGE_ACCELERATOR (1.0)

typedef struct { float x, y, z; } vec3d;

static unsigned int bits (float f) { unsigned int u; memcpy (&u, &f, 4); return u; }

static void set_env (unsigned short cw, unsigned int rc)
{
	unsigned int mxcsr;
	__asm__ __volatile__ ("fclex; fldcw %0" : : "m" (cw));
	__asm__ __volatile__ ("stmxcsr %0" : "=m" (mxcsr));
	mxcsr = (mxcsr & ~0x6000u) | (rc << 13);
	__asm__ __volatile__ ("ldmxcsr %0" : : "m" (mxcsr));
}

/* modules/system/fpu.h (WATCOM/MSVC): fistp dword ptr */
static void convert_float_to_int (float value, int *integer)
{
	__asm__ __volatile__ ("fistpl (%1)" : : "t" (value), "r" (integer) : "memory", "st");
}

/* modules/system/fpu.h (__GNUC__), verbatim: see the disassembly in the report */
static inline void gnuc_asm_convert_float_to_int (float value, int *integer)
{
	__asm__ __volatile__ ("fistp (%1);" : : "t" (value), "d" (integer) : "memory", "st");
}

/* gp_updt.c :: update_server, the sleep timer */
static __attribute__((noinline)) float probe_timer (float sleep, float delta_time)
{
	sleep -= delta_time;
	sleep = max (sleep, 0.0f);
	return sleep;
}

/* up_update.c :: set_entity_update_frame_rate */
static __attribute__((noinline)) void probe_subdivide (float delta_time, int frame_rate, int *iterations, float *sub)
{
	int entity_update_iterations;
	float entity_update_delta_time;
	entity_update_iterations = (int) (delta_time * frame_rate + 1.0);
	entity_update_delta_time = delta_time / entity_update_iterations;
	*iterations = entity_update_iterations;
	*sub = entity_update_delta_time;
}

/* group.c :: assess_group_supplies (ammo arm) */
static __attribute__((noinline)) void probe_supply (float group_level, float keysite_level, float *required_out, float *level_out, float *group_out)
{
	float required, level = keysite_level;
	required = 100.0 - (group_level * AMMO_USAGE_ACCELERATOR);
	required = bound (required, 0.0, level);
	level -= required;
	*required_out = required;
	*level_out = level;
	*group_out = group_level + required; /* the float argument of set_client_server_entity_float_value */
}

/* modules/maths/range.c :: get_2d_range */
static __attribute__((noinline)) float get_2d_range (const vec3d *v1, const vec3d *v2)
{
	float dx, dz, range;
	dx = v1->x - v2->x;
	dz = v1->z - v2->z;
	range = sqrt ((dx * dx) + (dz * dz));
	return (range);
}

/* modules/maths/range.c :: get_approx_2d_range */
static __attribute__((noinline)) float get_approx_2d_range (const vec3d *v1, const vec3d *v2)
{
	float dx, dz, range;
	dx = fabs (v1->x - v2->x);
	dz = fabs (v1->z - v2->z);
	if (dx > dz)
		range = ((dx * 4.0) + dz) * (1.0 / 4.0);
	else
		range = ((dz * 4.0) + dx) * (1.0 / 4.0);
	return (range);
}

/* en_world.c :: set_entity_world_map_size (x) */
static __attribute__((noinline)) void probe_map (int n, int side, float *max_out, float *mid_out)
{
	float min_map_x = 0.0, max_map_x, mid_map_x;
	max_map_x = (float) (n * side) - 1.0;
	mid_map_x = min_map_x + ((max_map_x - min_map_x) * 0.5);
	*max_out = max_map_x;
	*mid_out = mid_map_x;
}

/* keysite.c :: update_keysite_cargo, lines 428 and 464 (the crate-row step; #9 canary) */
struct OBJECT_3D_BOUNDS { float xmin, xmax, ymin, ymax, zmin, zmax; };

static __attribute__((noinline)) float probe_crate_row (float x, float xmin, float xmax)
{
	struct OBJECT_3D_BOUNDS box, *bounding_box = &box;
	vec3d position;
	position.x = x;
	box.xmin = xmin;
	box.xmax = xmax;
	position.x += (bounding_box->xmax - bounding_box->xmin) + 1.0;
	return position.x;
}

/* en_world.h :: get_x_sector */
#define get_x_sector(X_SEC,X) {convert_float_to_int ((X), &(X_SEC)); (X_SEC) /= SECTOR_SIDE_LENGTH;}

static __attribute__((noinline)) int probe_sector (float x, int SECTOR_SIDE_LENGTH)
{
	int x_sec;
	get_x_sector (x_sec, x);
	return x_sec;
}

int main (int argc, char **argv)
{
	unsigned short cw = (unsigned short) strtoul (argv[1], NULL, 16);
	unsigned int rc = (unsigned int) atoi (argv[2]);
	const char *probe = argv[3];

	if (strcmp (probe, "timer") == 0)
	{
		float s = strtof (argv[4], NULL), d = strtof (argv[5], NULL);
		int frames = atoi (argv[6]), i;
		set_env (cw, rc);
		for (i = 0; i < frames && s > 0.0; i++) s = probe_timer (s, d);
		printf ("%08x %d\n", bits (s), i);
	}
	else if (strcmp (probe, "subdivide") == 0)
	{
		float d = strtof (argv[4], NULL); int r = atoi (argv[5]), it; float sub;
		set_env (cw, rc);
		probe_subdivide (d, r, &it, &sub);
		printf ("%d %08x\n", it, bits (sub));
	}
	else if (strcmp (probe, "supply") == 0)
	{
		float g = strtof (argv[4], NULL), k = strtof (argv[5], NULL), r, l, o;
		set_env (cw, rc);
		probe_supply (g, k, &r, &l, &o);
		printf ("%08x %08x %08x %d\n", bits (r), bits (l), bits (o), o < 100.0);
	}
	else if (strcmp (probe, "range") == 0)
	{
		vec3d a = { strtof (argv[4], NULL), 0.0f, strtof (argv[5], NULL) };
		vec3d b = { strtof (argv[6], NULL), 0.0f, strtof (argv[7], NULL) };
		set_env (cw, rc);
		printf ("%08x %08x\n", bits (get_2d_range (&a, &b)), bits (get_approx_2d_range (&a, &b)));
	}
	else if (strcmp (probe, "map") == 0)
	{
		int n = atoi (argv[4]), side = atoi (argv[5]); float mx, md;
		set_env (cw, rc);
		probe_map (n, side, &mx, &md);
		printf ("%08x %08x\n", bits (mx), bits (md));
	}
	else if (strcmp (probe, "sector") == 0)
	{
		float x = strtof (argv[4], NULL); int side = atoi (argv[5]);
		set_env (cw, rc);
		printf ("%d\n", probe_sector (x, side));
	}
	else if (strcmp (probe, "crate-row") == 0)
	{
		float x = strtof (argv[4], NULL), xmin = strtof (argv[5], NULL), xmax = strtof (argv[6], NULL);
		set_env (cw, rc);
		printf ("%08x\n", bits (probe_crate_row (x, xmin, xmax)));
	}
	else if (strcmp (probe, "gnuc-fistp") == 0)
	{
		/* EECH's __GNUC__ asm_convert_float_to_int writes 16 bits of a 32-bit int */
		float x = strtof (argv[4], NULL); int out = 0x7eadbeef;
		set_env (cw, rc);
		gnuc_asm_convert_float_to_int (x, &out);
		printf ("%08x\n", (unsigned int) out);
	}
	else
	{
		return 2;
	}

	return 0;
}
