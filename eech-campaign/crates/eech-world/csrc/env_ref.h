/*
 * The reference host's weather and wind field (EEDCS-51, D3).  The kernels are the ORIGINAL C of aphavoc/source/entity/special/session/session.c
 * (get_session_weather_at_point, get_simple_session_weather_at_point, get_session_wind_velocity_at_point, the gust modulation,
 * set_session_random_weather_mode, set_session_fixed_weather_mode) and ss_updt.c (the physical part of update_transitional_weather and the lightning timer of
 * update_lightning_effect), with the same C types and the same expressions in the same order, so the same arithmetic gives the same bits.  The field is
 * the HOST's: the campaign module keeps only a read-only mirror of it (ss_hostenv.h).  See env_ref.c.
 *
 * Every function runs under a complete floating-point environment of its own: before its first statement the caller's COMPLETE state (the x87 environment with fnstenv: control, status and
 * tag words and the exception pointers; and the whole of MXCSR, control AND status bits, with stmxcsr -- fegetenv/fesetenv alone leave MXCSR as the kernel put it on the shipping
 * MinGW toolchain) is saved, round-toward-zero is set for both units and read back, and the caller's state is restored (fldenv, ldmxcsr) and read back on EVERY exit, the early
 * refusals included.  Every function returns 0, or nonzero with a message in `error`; a refusal changes nothing, an environment that does not read back as saved included.
 * See env_ref.c for the provenance of the guard.  This is the guard of THIS component, not a claim about the campaign DLL's own floating-point handling.
 */
#ifndef ENV_REF_H
#define ENV_REF_H

#include <stddef.h>

#define ENV_REF_FLOATS 23
#define ENV_REF_INTS 5
#define ENV_REF_DRAWS 10

typedef struct env_ref env_ref;

env_ref *env_ref_new (void);

void env_ref_free (env_ref *field);

/* the field: ENV_REF_FLOATS floats and ENV_REF_INTS integers in the order of ss_hostenv.h (the modes, then the three flags) */
int env_ref_import (env_ref *field, const float *f, const int *n, char *error, size_t error_size);

int env_ref_export (const env_ref *field, float *f, int *n, char *error, size_t error_size);

/* set_session_random_weather_mode: the ten draws, the configured radii and speeds (rain minimum, maximum, speed; wind minimum, maximum, speed) and the map extents
   (minimum x, maximum x, minimum z, maximum z, middle x, middle z) */
int env_ref_init_random (env_ref *field, const double *draw, const float *config, const float *map, char *error, size_t error_size);

int env_ref_init_fixed (env_ref *field, int mode, char *error, size_t error_size);

/* get_session_weather_at_point and the simple choice; modes = { current, target, simple } */
int env_ref_weather (const env_ref *field, float x, float z, int winter, int *modes, float *transitional_state, char *error, size_t error_size);

/* get_session_wind_velocity_at_point: the speed and, when wanted, the velocity vector */
int env_ref_wind (const env_ref *field, float x, float z, int want_vector, float *speed, float *vector, char *error, size_t error_size);

/* the gust modulation at a phase in [0, 1) */
int env_ref_gust (float phase, float *modulation, char *error, size_t error_size);

/* The physical update of one session update.
   in_f  = { delta, camera x, y, z, map minimum x, maximum x, minimum z, maximum z, middle x, middle z }
   in_i  = { winter, server, in_cockpit, have_camera, have_draw }
   draw  = the one draw of the lightning timer (used only when have_draw)
   out_i = { need_draw, expired }, out_f = { strike x, y, z, new timer }
   With need_draw the field is unchanged and out_f[3] is the timer after the decrement. */
int env_ref_step (env_ref *field, const float *in_f, const int *in_i, double draw, int *out_i, float *out_f, char *error, size_t error_size);

#endif
