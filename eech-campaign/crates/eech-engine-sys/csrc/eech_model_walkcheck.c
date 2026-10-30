/*
 * TEST-ONLY (EEDCS-50, feature `model-fixture`): the isolated control of the read-safety walk (eech_model_walk.h).
 *
 * It builds small synthetic arrays in this file's own static memory, describes them to mw_walk exactly as the fixture describes the loader's backing, and checks the
 * verdict against what the original accessor's integer path would read. It calls NO original code, reads no model, and the walk never dereferences an address the
 * description does not cover, so every case, including the failing ones, is memory-safe here. It proves the verifier's own bounds logic (missing, short, stale,
 * misaligned, overflowing and uninitialised backing, the 256-point surface, the end read, the budget); it says nothing about any retail model.
 *
 * eech_model_walkcheck_run returns the number of failed expectations (0: all held) and writes one line per failure; *checks is the number of expectations made.
 */

#ifdef EECH_MODEL_FIXTURE

#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "eech_model_walk.h"

struct wc
{
	char *text;
	int capacity, used, failures, checks;
};

static void wc_expect (struct wc *w, int condition, const char *label, int line)
{
	w->checks++;
	if (!condition)
	{
		int n;
		w->failures++;
		n = snprintf (w->text + w->used, (size_t) (w->capacity - w->used), "FAIL line %d: %s\n", line, label);
		if (n > 0 && n < w->capacity - w->used)
		{
			w->used += n;
		}
	}
}

#define EXPECT(w, condition) wc_expect ((w), (condition), #condition, __LINE__)

/* the synthetic memory: a block of sixteen-byte surface records, and the four arrays, each 4-byte aligned by its type */
static uint32_t wc_surfaces[4 * 8];     /* eight records */
static unsigned char wc_faces[16];
static unsigned char wc_list[32];
static uint16_t wc_sp[300];
static uint16_t wc_points[3 * 8];       /* eight points of three shorts */
static unsigned char wc_init[8];        /* the initialised-record bitmap of the surface block (one bit per record) */

static void set_surface (int index, unsigned points_byte, unsigned faces)
{
	unsigned char *record = (unsigned char *) wc_surfaces + 16 * index;
	uint16_t f = (uint16_t) faces;
	memset (record, 0, 16);
	record[MW_SURFACE_POINTS_AT] = (unsigned char) points_byte;
	memcpy (record + MW_SURFACE_FACES_AT, &f, sizeof (f));
}

static void region (struct mw_region *r, const void *base, uint64_t allocated, uint64_t initialised)
{
	r->known = 1;
	r->lo = (uintptr_t) base;
	r->allocated = allocated;
	r->initialised = initialised;
}

/* a model of `faces_n` faces in one surface (records 0 and 1 in the block unless `block_records` says fewer), every array fully backed */
static void base_input (struct mw_input *in, int block_records, uint64_t number_of_faces)
{
	memset (in, 0, sizeof (*in));
	in->surfaces_ptr = (uintptr_t) wc_surfaces;
	in->surfaces_lo = (uintptr_t) wc_surfaces;
	in->surfaces_hi = (uintptr_t) wc_surfaces + 16u * (unsigned) block_records;
	in->surfaces_known = 1;
	memset (wc_init, 0, sizeof (wc_init));
	mw_mark_records (wc_init, 0, (uint64_t) block_records);
	in->surfaces_init = wc_init;
	in->surfaces_init_records = (uint64_t) block_records;
	in->faces_ptr = (uintptr_t) wc_faces;
	region (&in->faces, wc_faces, sizeof (wc_faces), sizeof (wc_faces));
	in->list_ptr = (uintptr_t) wc_list;
	region (&in->list, wc_list, sizeof (wc_list), sizeof (wc_list));
	in->sp_ptr = (uintptr_t) wc_sp;
	region (&in->sp, wc_sp, sizeof (wc_sp), sizeof (wc_sp));
	in->points_ptr = (uintptr_t) wc_points;
	region (&in->points, wc_points, sizeof (wc_points), sizeof (wc_points));
	in->number_of_faces = number_of_faces;
	in->step_budget = 100000;
}

static void reset_memory (void)
{
	memset (wc_surfaces, 0, sizeof (wc_surfaces));
	memset (wc_faces, 0, sizeof (wc_faces));
	memset (wc_list, 0, sizeof (wc_list));
	memset (wc_sp, 0, sizeof (wc_sp));
	memset (wc_points, 0, sizeof (wc_points));
}

int eech_model_walkcheck_run (char *text, int capacity, int *checks)
{
	struct wc w;
	struct mw_input in;
	struct mw_result r;
	int i;

	w.text = text;
	w.capacity = capacity;
	w.used = 0;
	w.failures = 0;
	w.checks = 0;
	if (capacity > 0)
	{
		text[0] = 0;
	}

	/* one triangle, the end read backed by a second record */
	reset_memory ();
	set_surface (0, 3, 1);
	set_surface (1, 3, 0);
	wc_faces[0] = 3;
	wc_list[0] = 0;
	wc_list[1] = 1;
	wc_list[2] = 2;
	wc_sp[0] = 0;
	wc_sp[1] = 1;
	wc_sp[2] = 2;
	base_input (&in, 2, 1);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_OK && r.complete && r.end_backed && r.safe_depths == 1 && r.triangles == 1);
	EXPECT (&w, r.faces_read == 1 && r.list_read == 3 && r.sp_read == 3 && r.points_read == 3);
	EXPECT (&w, r.face_points_sum == 3 && r.face_points_min == 3 && r.face_points_max == 3);

	/* the same, the block ending with the object's own surface: every depth is safe, the end call's read of the next surface is not */
	base_input (&in, 1, 1);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_SURFACES_OUTSIDE && r.complete && !r.end_backed && r.safe_depths == 1 && r.triangles == 1);
	EXPECT (&w, !strcmp (r.array, "surfaces"));

	/* a quad: two triangles, the fan reads refs[0], refs[2], refs[3] at depth 1 */
	reset_memory ();
	set_surface (0, 4, 1);
	set_surface (1, 4, 0);
	wc_faces[0] = 4;
	wc_list[0] = 0;
	wc_list[1] = 1;
	wc_list[2] = 2;
	wc_list[3] = 3;
	for (i = 0; i < 4; i++)
	{
		wc_sp[i] = (uint16_t) i;
	}
	base_input (&in, 2, 1);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_OK && r.safe_depths == 2 && r.triangles == 2 && r.list_read == 4 && r.sp_read == 4 && r.points_read == 4);

	/* a face array whose backing was not observed */
	base_input (&in, 2, 1);
	in.faces.known = 0;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_UNOBSERVED && r.safe_depths == 0 && !r.end_backed && !strcmp (r.array, "faces"));

	/* NULL arrays the walk must read */
	base_input (&in, 2, 1);
	in.list_ptr = 0;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_NULL_ARRAY && r.safe_depths == 0 && !strcmp (r.array, "point_list"));
	base_input (&in, 2, 1);
	in.surfaces_ptr = 0;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_NULL_SURFACES && r.safe_depths == 0);

	/* two faces, the face array backing only the first: depth 0 (and 1) is safe, the second face's read is not */
	reset_memory ();
	set_surface (0, 4, 2);
	set_surface (1, 4, 0);
	wc_faces[0] = 3;
	wc_faces[1] = 3;
	for (i = 0; i < 6; i++)
	{
		wc_list[i] = (unsigned char) (i % 3);
	}
	for (i = 0; i < 4; i++)
	{
		wc_sp[i] = (uint16_t) i;
	}
	base_input (&in, 2, 2);
	region (&in.faces, wc_faces, 1, 1);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_UNBACKED && r.safe_depths == 1 && r.triangles == 1 && !r.complete && !r.end_backed && !strcmp (r.array, "faces"));
	/* the same allocation, the second face allocated but never filled */
	base_input (&in, 2, 2);
	region (&in.faces, wc_faces, 2, 1);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_UNINITIALISED && r.safe_depths == 1 && !strcmp (r.array, "faces"));
	/* and with both faces filled the walk is whole */
	base_input (&in, 2, 2);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_OK && r.safe_depths == 2 && r.triangles == 2 && r.faces_read == 2 && r.list_read == 6);

	/* a point list that ends inside the second triangle's fan */
	base_input (&in, 2, 2);
	region (&in.list, wc_list, 5, 5);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_UNBACKED && r.safe_depths == 1 && !strcmp (r.array, "point_list"));

	/* a surface-point array that ends before the highest reference */
	base_input (&in, 2, 2);
	region (&in.sp, wc_sp, 4, 4);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_UNBACKED && r.safe_depths == 0 && !strcmp (r.array, "surface_points"));

	/* a point index past the points allocation: three points backed, the highest surface-point entry names the fourth */
	reset_memory ();
	set_surface (0, 3, 1);
	set_surface (1, 3, 0);
	wc_faces[0] = 3;
	wc_list[0] = 0;
	wc_list[1] = 1;
	wc_list[2] = 2;
	wc_sp[0] = 0;
	wc_sp[1] = 1;
	wc_sp[2] = 3;
	base_input (&in, 2, 1);
	region (&in.points, wc_points, 3 * 6, 3 * 6);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_UNBACKED && r.safe_depths == 0 && !strcmp (r.array, "points"));

	/* a surface-point pointer on an odd address is misaligned (a stale or forged pointer), never read */
	base_input (&in, 2, 1);
	in.sp_ptr = (uintptr_t) wc_sp + 1;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_MISALIGNED && r.safe_depths == 0 && !strcmp (r.array, "surface_points"));

	/* a pointer below its region's base (a stale pointer into a freed neighbour) */
	base_input (&in, 2, 1);
	in.list_ptr = (uintptr_t) wc_list;
	region (&in.list, wc_list + 4, sizeof (wc_list) - 4, sizeof (wc_list) - 4);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_UNBACKED && r.safe_depths == 0 && !strcmp (r.array, "point_list"));

	/* address arithmetic that would wrap is refused before any check or read */
	{
		struct mw_region any;
		uintptr_t at = 0;
		region (&any, wc_faces, sizeof (wc_faces), sizeof (wc_faces));
		memset (&r, 0, sizeof (r));
		EXPECT (&w, !mw_read (&r, 0, UINTPTR_MAX, &any, 1, 1, 1, "faces", &at) && r.reason == MW_OVERFLOW);
		memset (&r, 0, sizeof (r));
		EXPECT (&w, !mw_read (&r, 0, (uintptr_t) wc_faces, &any, UINT64_MAX, 6, 2, "points", &at) && r.reason == MW_OVERFLOW);
		memset (&r, 0, sizeof (r));
		EXPECT (&w, mw_read (&r, 0, (uintptr_t) wc_faces, &any, 3, 1, 1, "faces", &at) && at == (uintptr_t) wc_faces + 3);
	}

	/* a surface outside the consolidated block, or misaligned within it: surface 0 is never read */
	base_input (&in, 2, 1);
	in.surfaces_ptr = (uintptr_t) wc_surfaces + 16u * 2u;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_SURFACES_OUTSIDE && r.safe_depths == 0 && !r.end_backed);
	base_input (&in, 2, 1);
	in.surfaces_ptr = (uintptr_t) wc_surfaces + 2u;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_MISALIGNED && r.safe_depths == 0 && !strcmp (r.array, "surfaces"));
	base_input (&in, 2, 1);
	in.surfaces_ptr = (uintptr_t) wc_surfaces + 8u; /* the record straddles the end of the block */
	in.surfaces_hi = (uintptr_t) wc_surfaces + 16u;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_SURFACES_OUTSIDE && r.safe_depths == 0);

	/* no faces at all: only surface 0 is read, the end call is safe */
	reset_memory ();
	set_surface (0, 3, 0);
	base_input (&in, 1, 0);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_OK && r.complete && r.end_backed && r.safe_depths == 0 && r.faces_read == 0);

	/* a line (two points) is consumed at every depth and holds no triangle; the face after it is walked from its own references */
	reset_memory ();
	set_surface (0, 4, 2);
	set_surface (1, 4, 0);
	wc_faces[0] = 2;
	wc_faces[1] = 3;
	wc_list[0] = 0;
	wc_list[1] = 1;
	wc_list[2] = 0;
	wc_list[3] = 1;
	wc_list[4] = 2;
	for (i = 0; i < 4; i++)
	{
		wc_sp[i] = (uint16_t) i;
	}
	base_input (&in, 2, 2);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_OK && r.safe_depths == 1 && r.triangles == 1 && r.faces_read == 2 && r.list_read == 5);
	EXPECT (&w, r.face_points_sum == 5 && r.face_points_min == 2 && r.face_points_max == 3);

	/* a face of one point AFTER a triangle: the original consumes it at every depth, its depth growing by one, and it holds no triangle. Walked exactly; nothing is refused */
	wc_faces[0] = 3;
	wc_faces[1] = 1;
	wc_list[0] = 0;
	wc_list[1] = 1;
	wc_list[2] = 2;
	base_input (&in, 2, 2);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_OK && r.complete && r.end_backed && r.safe_depths == 1 && r.triangles == 1 && r.faces_read == 2 && r.list_read == 3);
	EXPECT (&w, r.face_points_sum == 4 && r.face_points_min == 1 && r.face_points_max == 3);

	/* faces of 1 and 3 points only: the depth the first adds (+1) is spent by the second (-1); no depth returns TRUE, depth 0 is the first FALSE (the original at depth 0:
	   face 0 consumed, depth 1, face 1 consumed, depth 0, no face left), no point-list entry is read, and the end call is backed */
	reset_memory ();
	set_surface (0, 4, 2);
	set_surface (1, 4, 0);
	wc_faces[0] = 1;
	wc_faces[1] = 3;
	base_input (&in, 2, 2);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_OK && r.complete && r.end_backed && r.safe_depths == 0 && r.triangles == 0 && r.faces_read == 2 && r.list_read == 0 && r.sp_read == 0);
	EXPECT (&w, r.face_points_sum == 4 && r.face_points_min == 1 && r.face_points_max == 3);

	/* a face of one point BEFORE a quad: the original at depth 0 consumes it (depth 1) and stops at the quad (4 - 3 < 1 is false) with triangle 1 of the quad, so it reads
	   refs[0], refs[2 + 1], refs[2 + 2] from the quad's own references at offset 1, i.e. list entries 1, 3 and 4. Depth 1 consumes both faces: the first FALSE is depth 1 */
	reset_memory ();
	set_surface (0, 4, 2);
	set_surface (1, 4, 0);
	wc_faces[0] = 1;
	wc_faces[1] = 4;
	wc_list[1] = 0;
	wc_list[3] = 2;
	wc_list[4] = 3;
	for (i = 0; i < 5; i++)
	{
		wc_sp[i] = (uint16_t) i;
	}
	base_input (&in, 2, 2);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_OK && r.complete && r.end_backed && r.safe_depths == 1 && r.triangles == 1 && r.faces_read == 2 && r.list_read == 5);
	EXPECT (&w, r.sp_read == 4 && r.points_read == 4);
	/* the same, the point list backing entries 0..3 only: the quad's last read (entry 4) is the first unbacked read, and it is depth 0's */
	base_input (&in, 2, 2);
	region (&in.list, wc_list, 4, 4);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_UNBACKED && r.safe_depths == 0 && r.triangles == 0 && !r.complete && !r.end_backed && !strcmp (r.array, "point_list"));
	/* and the face array backing only the one-point face: the quad's face read is unbacked at the first depth that reaches it (depth 0) */
	base_input (&in, 2, 2);
	region (&in.faces, wc_faces, 1, 1);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_UNBACKED && r.safe_depths == 0 && !r.complete && r.faces_read == 1 && !strcmp (r.array, "faces"));

	/* a face of ZERO points adds 2 to the depth and advances no reference: before a 5-point face (3 triangles) the original at depth 0 stops at triangle 2 of it
	   (depth 2, 5 - 3 < 2 is false), so only depth 0 is TRUE (the face claims depths 0 only: its triangles 0 and 1 are spent by the zero-point face) and depth 1 is the first FALSE */
	reset_memory ();
	set_surface (0, 5, 2);
	set_surface (1, 5, 0);
	wc_faces[0] = 0;
	wc_faces[1] = 5;
	for (i = 0; i < 5; i++)
	{
		wc_list[i] = (unsigned char) i;
		wc_sp[i] = (uint16_t) i;
	}
	base_input (&in, 2, 2);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_OK && r.complete && r.end_backed && r.safe_depths == 1 && r.triangles == 1 && r.faces_read == 2 && r.list_read == 5);
	EXPECT (&w, r.sp_read == 5 && r.points_read == 5 && r.face_points_sum == 5 && r.face_points_min == 0 && r.face_points_max == 5);

	/* the count of faces is the only bound of a run of 0-point faces, and the walk is bounded by what is backed and by the step budget, never by trust */
	reset_memory ();
	set_surface (0, 3, 0);
	base_input (&in, 1, (uint64_t) 1 << 40);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_UNBACKED && r.safe_depths == 0 && !r.complete && r.faces_read == 16 && !strcmp (r.array, "faces"));
	base_input (&in, 1, (uint64_t) 1 << 40);
	in.step_budget = 10;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_BUDGET && r.safe_depths == 0 && !r.complete);

	/* the surface advance: two surfaces of one face each; the second face's references index past the first surface's points */
	reset_memory ();
	set_surface (0, 3, 1);
	set_surface (1, 3, 1);
	set_surface (2, 3, 0);
	wc_faces[0] = 3;
	wc_faces[1] = 3;
	for (i = 0; i < 6; i++)
	{
		wc_list[i] = (unsigned char) (i % 3);
		wc_sp[i] = (uint16_t) (i % 3);
	}
	base_input (&in, 3, 2);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_OK && r.safe_depths == 2 && r.sp_read == 6 && r.end_backed);
	/* the next surface is read even after the last face: with the block ending at the second surface the end call is unbacked */
	base_input (&in, 2, 2);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_SURFACES_OUTSIDE && r.complete && !r.end_backed && r.safe_depths == 2);
	/* and with a third face the advance to a surface outside the block stops the depths there */
	reset_memory ();
	set_surface (0, 3, 1);
	set_surface (1, 3, 1);
	wc_faces[0] = 3;
	wc_faces[1] = 3;
	wc_faces[2] = 3;
	base_input (&in, 2, 3);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_SURFACES_OUTSIDE && !r.complete && r.safe_depths == 2);

	/* a surface of 0 recorded points stands for 256: the second surface's references start 256 entries on */
	reset_memory ();
	set_surface (0, 0, 1);
	set_surface (1, 3, 1);
	set_surface (2, 3, 0);
	wc_faces[0] = 3;
	wc_faces[1] = 3;
	for (i = 0; i < 6; i++)
	{
		wc_list[i] = (unsigned char) (i % 3);
	}
	for (i = 0; i < 300; i++)
	{
		wc_sp[i] = 0;
	}
	base_input (&in, 3, 2);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_OK && r.safe_depths == 2 && r.sp_read == 256 + 3);
	base_input (&in, 3, 2);
	region (&in.sp, wc_sp, 10 * sizeof (uint16_t), 10 * sizeof (uint16_t));
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_UNBACKED && r.safe_depths == 1 && !strcmp (r.array, "surface_points"));

	/* the step budget bounds the walk */
	reset_memory ();
	set_surface (0, 4, 1);
	set_surface (1, 4, 0);
	wc_faces[0] = 4;
	base_input (&in, 2, 1);
	in.step_budget = 3;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_BUDGET && !r.end_backed && r.safe_depths == 0);

	/* ---- the surface provenance gate: nothing about a surface record is read before it is shown observed, inside the block, on a record boundary, aligned and filled.
	   The pointers below are chosen so that a dereference before the gate would fault (0x10, an unmapped-looking address) or would change the answer (poisoned bytes). ---- */
	reset_memory ();
	set_surface (0, 3, 1);
	set_surface (1, 3, 0);
	wc_faces[0] = 3;
	wc_list[1] = 1;
	wc_list[2] = 2;
	for (i = 0; i < 3; i++)
	{
		wc_sp[i] = (uint16_t) i;
	}
	/* a block whose provenance was not observed: refused at once, the (non-NULL, unreadable) pointer never touched */
	base_input (&in, 2, 1);
	in.surfaces_known = 0;
	in.surfaces_ptr = 0x10;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_UNOBSERVED && r.safe_depths == 0 && !r.end_backed && !strcmp (r.array, "surfaces"));
	/* an object whose surfaces are not in the block */
	base_input (&in, 2, 1);
	in.surfaces_ptr = 0x10;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_SURFACES_OUTSIDE && r.safe_depths == 0);
	base_input (&in, 2, 1);
	in.surfaces_ptr = UINTPTR_MAX - 7u;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_SURFACES_OUTSIDE && r.safe_depths == 0);
	/* 4-byte aligned but not on a record boundary of the block, and a block whose own base is misaligned */
	base_input (&in, 2, 1);
	in.surfaces_ptr = (uintptr_t) wc_surfaces + 4u;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_MISALIGNED && r.safe_depths == 0);
	base_input (&in, 2, 1);
	in.surfaces_lo = (uintptr_t) wc_surfaces + 2u;
	in.surfaces_ptr = (uintptr_t) wc_surfaces + 2u;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_MISALIGNED && r.safe_depths == 0);
	/* the bitmap missing, or shorter than the block */
	base_input (&in, 2, 1);
	in.surfaces_init = NULL;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_UNOBSERVED && r.safe_depths == 0);
	base_input (&in, 2, 1);
	in.surfaces_init_records = 1;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_UNOBSERVED && r.complete && !r.end_backed && r.safe_depths == 1 && r.triangles == 1);
	/* the object's own first record not initialised (a short read): refused before its poisoned bytes (number of faces 0xFFFF) can be believed */
	base_input (&in, 2, 1);
	wc_init[0] = 0x02;
	{
		unsigned char *record = (unsigned char *) wc_surfaces;
		record[8] = 0xFF;
		record[9] = 0xFF;
	}
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_UNINITIALISED && r.safe_depths == 0 && !r.complete && !strcmp (r.array, "surfaces"));
	set_surface (0, 3, 1);
	/* the record after the last face (the end call's read) not initialised: every depth is safe, the end call is not */
	base_input (&in, 2, 1);
	wc_init[0] = 0x01;
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_UNINITIALISED && r.complete && !r.end_backed && r.safe_depths == 1 && r.triangles == 1);
	/* and the healthy case on the same memory */
	base_input (&in, 2, 1);
	mw_walk (&in, &r);
	EXPECT (&w, r.reason == MW_OK && r.complete && r.end_backed && r.safe_depths == 1);
	/* the gate itself: it reports without touching its output on a refusal, and for a wrapping index */
	{
		uintptr_t at = 0;
		base_input (&in, 2, 1);
		EXPECT (&w, mw_surface_gate (&in, 0, &at) == 0 && at == (uintptr_t) wc_surfaces);
		at = 0;
		EXPECT (&w, mw_surface_gate (&in, 1, &at) == 0 && at == (uintptr_t) wc_surfaces + 16u);
		at = 0;
		EXPECT (&w, mw_surface_gate (&in, 2, &at) == MW_SURFACES_OUTSIDE && at == 0);
		EXPECT (&w, mw_surface_gate (&in, UINT64_MAX, &at) == MW_OVERFLOW && at == 0);
		wc_init[0] = 0x01;
		EXPECT (&w, mw_surface_gate (&in, 1, &at) == MW_UNINITIALISED && at == 0);
	}

	/* ---- which records the loader filled: the bookkeeping over fread results, then carried through a consolidation into the block's bitmap ---- */
	{
		struct mw_fill a, b, c, d;
		unsigned char bits[8];
		int k;

		memset (&a, 0, sizeof (a));
		memset (&b, 0, sizeof (b));
		memset (&c, 0, sizeof (c));
		memset (&d, 0, sizeof (d));
		/* object A: three records, every field read in full */
		for (i = 0; i < 3; i++)
		{
			for (k = 0; k < MW_FILL_FIELDS; k++)
			{
				mw_fill_field (&a, i, k, 1);
			}
		}
		EXPECT (&w, a.prefix == 3 && !a.bad);
		/* object B: record 0 whole; in record 1 the number-of-faces fread fails (bytes 8 and 9 are covered by nothing else), so it is not initialised; record 2 is whole but
		   follows a hole, so it does not extend the run */
		for (k = 0; k < MW_FILL_FIELDS; k++)
		{
			mw_fill_field (&b, 0, k, 1);
		}
		for (k = 0; k < MW_FILL_FIELDS; k++)
		{
			mw_fill_field (&b, 1, k, k == 3 ? 0 : 1);
		}
		EXPECT (&w, b.prefix == 1);
		for (k = 0; k < MW_FILL_FIELDS; k++)
		{
			mw_fill_field (&b, 2, k, 1);
		}
		EXPECT (&w, b.prefix == 1 && !b.bad);
		/* the int read of number_of_faces spills over reflectivity and specularity, so those two failing leaves every byte covered; the colour failing does not */
		for (k = 0; k < MW_FILL_FIELDS; k++)
		{
			mw_fill_field (&c, 0, k, (k == 5 || k == 6) ? 0 : 1);
		}
		EXPECT (&w, c.prefix == 1);
		for (k = 0; k < MW_FILL_FIELDS; k++)
		{
			mw_fill_field (&c, 1, k, k == 4 ? 0 : 1);
		}
		EXPECT (&w, c.prefix == 1 && !c.bad);
		/* a field reported twice for one record, and one that does not exist, are inconsistencies, never filled bytes */
		mw_fill_field (&d, 0, 0, 1);
		mw_fill_field (&d, 0, 0, 1);
		EXPECT (&w, d.bad);
		memset (&d, 0, sizeof (d));
		mw_fill_field (&d, 0, MW_FILL_FIELDS, 1);
		EXPECT (&w, d.bad && d.prefix == 0);

		/* the consolidation: A's three records are the block's records 0..2, B's records 3..4 of which only B's first is initialised */
		memset (bits, 0, sizeof (bits));
		mw_mark_records (bits, 0, a.prefix);
		mw_mark_records (bits, 3, b.prefix);
		EXPECT (&w, bits[0] == 0x0F);
		/* a mixed walk from A's last record (3 points, 1 face) through B's first (1 face) to B's second, which is not initialised */
		reset_memory ();
		set_surface (2, 3, 1);
		set_surface (3, 3, 1);
		set_surface (4, 3, 0);
		wc_faces[0] = 3;
		wc_faces[1] = 3;
		for (i = 0; i < 6; i++)
		{
			wc_list[i] = (unsigned char) (i % 3);
			wc_sp[i] = (uint16_t) (i % 3);
		}
		base_input (&in, 5, 2);
		in.surfaces_ptr = (uintptr_t) wc_surfaces + 2u * 16u;
		in.surfaces_init = bits;
		in.surfaces_init_records = 5;
		mw_walk (&in, &r);
		EXPECT (&w, r.reason == MW_UNINITIALISED && r.complete && !r.end_backed && r.safe_depths == 2 && r.triangles == 2 && !strcmp (r.array, "surfaces"));
		/* the same with B's second record initialised: the walk crosses from one object's records into the next's and its end read is backed */
		mw_mark_records (bits, 4, 1);
		mw_walk (&in, &r);
		EXPECT (&w, r.reason == MW_OK && r.complete && r.end_backed && r.safe_depths == 2);
	}

	if (checks)
	{
		*checks = w.checks;
	}
	return w.failures;
}

#endif
