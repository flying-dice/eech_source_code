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

/* The physical rain and wind levels of the session's ambient sound children (EEDCS-51, ss_updt.c update_session_sound_effects), from the host's own field.
   in_f = { x, z, y, alt, cloud base } (the camera, or the gunship in the cockpit, and its radar altitude and the cloud height: the campaign's observations; the floats are read INSIDE the guard,
           from memory, never as parameters)
   in_i = { winter, in_cockpit }
   out  = { rain light, rain heavy, wind light, wind heavy }, written only on success.
   The original's expressions in the original's order: the weather is asked only inside the rain branch; outdoors the target mode's entry and then the current mode's entry are attenuated
   separately (equal modes alias one entry: the altitude factor is applied TWICE).
   Unlike the closed field kernels, this entry runs in the CANONICAL arithmetic environment (x87 extended precision, round toward zero, MXCSR round toward zero with FTZ and DAZ off, every exception
   masked, set before any floating-point work and read back), so its answer does not depend on the caller's denormal modes or precision; the caller's complete state is restored on every exit. */
int env_ref_ambient (const env_ref *field, const float *in_f, const int *in_i, float *out, char *error, size_t error_size);

/* The physical update of one session update.
   in_f  = { delta, camera x, y, z, map minimum x, maximum x, minimum z, maximum z, middle x, middle z }
   in_i  = { winter, server, in_cockpit, have_camera, have_draw }
   draw  = the ADDRESS of the one draw of the lightning timer (a readable double; read only when have_draw, and only inside the entry's floating-point guard).  It is a pointer and
           not a double by value on purpose: a 32-bit callee homes a by-value double through the x87 stack in its prologue, BEFORE the guard has saved the caller's state, which clears
           condition code C1 of the caller (the actual disassembly of the frozen d4e5c34d2 entry: fld qword [esp+..]; fstp qword [esp+..] before fp_save).  It may be NULL when no draw is given.
   out_i = { need_draw, expired }, out_f = { strike x, y, z, new timer }
   With need_draw the field is unchanged and out_f[3] is the timer after the decrement. */
int env_ref_step (env_ref *field, const float *in_f, const int *in_i, const double *draw, int *out_i, float *out_f, char *error, size_t error_size);


/* The physical queue of lightning strikes (EEDCS-51, modules/3d/3dstrike.c): the host owns it.  One slot, by value: the in_use flag, the type, the position, the start time and length and the current frame.
   The queue is empty when the field is created, as the original zeroed it at start-up. */
#define ENV_REF_STRIKES 32

typedef struct env_ref_strike
{
	int in_use;
	int type;
	float x, y, z;
	float time_started;
	float time_length;
	int current_frame_number;
} env_ref_strike;

/* add_3d_lightning_strike: the first free slot is filled, and no slot is filled with no active environment or with every slot in use.
   in_f = { length, x, z, time of day }, in_i = { type, whether an active 3D environment exists } (the floats are read INSIDE the guard, from memory, never as parameters)
   out_slot = the slot filled, or -1.  Runs in the CANONICAL arithmetic environment (see env_ref_ambient). */
int env_ref_lightning_add (env_ref *field, const float *in_f, const int *in_i, int *out_slot, char *error, size_t error_size);

/* update_3d_lightning_strikes: every strike in use is aged by the time of day and the finished ones are freed.
   in_f = { time of day }, in_i = { have_frames, frames } (the strike animation's frame count, which only the campaign's texture table holds, needed only to age a forked strike)
   out_need_frames = 1 when a forked strike is to be aged and no count was given: the queue is then unchanged.  An unknown strike type refuses and changes nothing. */
int env_ref_lightning_update (env_ref *field, const float *in_f, const int *in_i, int *out_need_frames, char *error, size_t error_size);

/* the queue as it stands (ENV_REF_STRIKES slots), as plain bits */
int env_ref_lightning_queue (const env_ref *field, env_ref_strike *out, char *error, size_t error_size);

/* The PHYSICAL LIFETIME of a sound effect (EEDCS-51, D3; soundeff.c and se_updt.c).  No state is kept: the stored fields travel by value.  The samples of a sequence: ENV_REF_SOUND_SAMPLES at most. */
#define ENV_REF_SOUND_SAMPLES 16

/* set_local_sound_effect_sample_indices: in_i = { looping, music, count, rate[ENV_REF_SOUND_SAMPLES], size[ENV_REF_SOUND_SAMPLES] } (the sample database's ints); out_lifetime = the effect's lifetime.  CANONICAL arithmetic environment. */
int env_ref_sound_init (const int *in_i, float *out_lifetime, char *error, size_t error_size);

/* one call of update_server (in_i[0] set) or update_client: in_f = { delta, lifetime, valid lifetime }, in_i = { server, valid, looping, has handle, have playing, playing };
   out_f = { lifetime, valid lifetime }, out_i = { expired, action (0 none, 1 kill, 2 unlink), need playing }.  need playing = 1 (nothing changed) when a server's expired effect holds a handle and the campaign has not
   said whether it is playing.  CANONICAL arithmetic environment; the floats are read inside the guard from memory. */
int env_ref_sound_update (const float *in_f, const int *in_i, float *out_f, int *out_i, char *error, size_t error_size);

/* soundeff.c pause_local_entity_sound_type / resume_local_entity_sound_type for ONE matching effect: in_w = { delay, valid lifetime } as 32-bit WORDS, in_i = { resume, valid, member of the update list };
   out_w = { valid lifetime } as a word (the delay for a pause that changed the effect, else the word it came in as), out_i = { changed, valid, play (the parent is asked to play), relink (the effect goes back in the update list) }.
   No floating-point work: the caller's x87 and MXCSR state is never touched.  A valid lifetime, and a delay that would be written, that is not finite is refused (nothing changed). */
int env_ref_sound_pause_resume (const unsigned int *in_w, const int *in_i, unsigned int *out_w, int *out_i, char *error, size_t error_size);

#endif
