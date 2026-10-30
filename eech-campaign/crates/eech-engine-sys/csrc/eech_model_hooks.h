/*
 * TEST-ONLY (EEDCS-50, feature `model-fixture`): the observer hooks the model-fixture build's patched modules/3d/3dobjdb.c calls (build/patches.rs,
 * MODEL_FIXTURE_PATCHES). This header is included by the staged 3dobjdb.c ONLY when the feature is on; with it off none of the hooks, the patches or the
 * definitions (eech_model_fixture.c) exist in the build.
 *
 * Every hook only RECORDS what the real loader has just allocated, filled or consumed, in bounded private tables (eech_model_fixture.c). None allocates,
 * frees, writes a model byte, draws a random number, reads the clock, changes an entity or does floating-point arithmetic; each takes plain pointers and
 * integers and compares nothing but integers.
 */
#ifndef EECH_MODEL_HOOKS_H
#define EECH_MODEL_HOOKS_H

#include <stddef.h>

/* initialise_3d_objects and deinitialise_3d_objects begin: every record is dropped and the generation moves on, so a record of an earlier load can never match */
extern void eech_mf_reset (void);

/* one fread of the common face (which = 0) or point-list (which = 1) block in the object loop: where the block starts, where this read began, how many items were asked
   for and how many fread returned (the original ignores the return value; a short read leaves the rest of the block uninitialised) */
extern void eech_mf_common_read (int which, const void *base, const void *cursor, long long requested, long long returned);

/* the object loop is over, before any override mutates an object: the exact common allocations (item counts as allocated) and every cursor the loader advanced
   (one past the last item it consumed), and the bases of the two file-mapped arrays */
extern void eech_mf_common_done (const void *faces, long long faces_allocated, const void *faces_cursor, const void *list, long long list_allocated,
	const void *list_cursor, const void *points, const void *points_cursor, const void *surface_points, const void *surface_points_cursor);

/* read_object: how many items fread returned for array `which` (0 points, 1 faces, 2 point list, 3 surface-point references); returns it unchanged */
extern size_t eech_mf_filled (int which, size_t returned);

/* read_object has just made `object` custom: the local counts it allocated by (polygon faces, point references, surface-point references) are the extents; the
   pointers and the effective point count are read from the object itself */
extern void eech_mf_custom (const void *object, long long polygoned_faces, long long point_references, long long surface_point_references);

#endif
