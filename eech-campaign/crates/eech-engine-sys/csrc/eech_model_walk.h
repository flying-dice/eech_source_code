/*
 * TEST-ONLY (EEDCS-50, feature `model-fixture`): the read-safety walk of the ORIGINAL collision accessor, as a pure function over described memory.
 *
 * `get_object_3d_collision_object_geometry_triangle` (modules/3d/3dobjdb.c) steps through an object's faces, point list, surface-point references and points by
 * integer arithmetic and reads each by pointer, with no limit. Whether a call is memory-safe is therefore a question about WHICH addresses it reads. This header
 * answers it without calling the accessor: it replays the accessor's integer path exactly (the unconditional read of surface 0, the loop on
 * `faces[count].number_of_points - 3 < depth`, the depth, point-list and surface-point stepping, the surface advance when a surface's face count reaches zero with
 * 0 points meaning 256, the read of the next surface even after the last face, then the three fan points through the point list, the surface-point reference and
 * the point) and checks every address it would read against what the caller says is backed. It never computes a triangle and never touches the float part.
 *
 * What the caller describes (all of it from observation of the real allocations, never from a guess):
 *   - for each array a `region`: where the readable initialised bytes start, how many there are, how many bytes were allocated (>= initialised), and whether the
 *     region is KNOWN at all. A read inside the allocation but past the initialised bytes is `MW_UNINITIALISED`; a read outside is `MW_UNBACKED`; an array whose
 *     backing was not observed is `MW_UNOBSERVED`;
 *   - the object's own array pointers (the walk reads from them, not from the region start: a mixed-layout walk may leave the object's own extent and stay inside the
 *     shared region, which is correct and permitted, or leave the region, which is refused);
 *   - the consolidated surface block: its observed base and extent, one initialised bit per 16-byte record, and whether it was observed at all. Every surface read goes through
 *     `mw_surface_gate` first (inside the block, on a record boundary, 4-byte aligned, initialised); the caller uses the same gate for its own metadata reads.
 *
 * The result is the longest run of depths 0 .. safe_depths-1 whose reads are all backed, why the walk stops if it does, and the extents it read. All address arithmetic
 * is on `uintptr_t` with explicit overflow checks and alignment checks; no pointer is compared with a pointer of another allocation, no read is made before the check,
 * and every loop is bounded by the caller's step budget.
 *
 * The file includes only <stdint.h> and <string.h>; it holds no engine type, so the same text is exercised by an isolated self-test (eech_model_walkcheck.c) that
 * calls no original code.
 */
#ifndef EECH_MODEL_WALK_H
#define EECH_MODEL_WALK_H

#include <stdint.h>
#include <string.h>

/* one array's backing */
struct mw_region
{
	int known;           /* the backing of this array was observed; 0: nothing can be said, every read is MW_UNOBSERVED */
	uintptr_t lo;        /* first byte of the allocation */
	uint64_t allocated;  /* bytes allocated from lo */
	uint64_t initialised; /* leading bytes of the allocation that hold data (<= allocated) */
};

struct mw_input
{
	uintptr_t surfaces_ptr; /* the object's first surface (obj->surfaces) */
	uintptr_t surfaces_lo;  /* the consolidated surface block, as the loader allocated it */
	uintptr_t surfaces_hi;  /* one past its last record */
	int surfaces_known;     /* the block's identity, extent and filled records were OBSERVED (and the object's pointer is the one the loader bound); 0: every surface read is MW_UNOBSERVED */
	const unsigned char *surfaces_init; /* one bit per 16-byte record from surfaces_lo (bit r of byte r/8): 1 = that record's 16 bytes were all initialised by the loader */
	uint64_t surfaces_init_records;     /* the records the bitmap describes */
	uintptr_t faces_ptr;    /* obj->faces */
	struct mw_region faces;
	uintptr_t list_ptr;     /* obj->object_faces_point_plain_list */
	struct mw_region list;
	uintptr_t sp_ptr;       /* obj->surface_points */
	struct mw_region sp;
	uintptr_t points_ptr;   /* obj->points */
	struct mw_region points;
	uint64_t number_of_faces; /* obj->number_of_faces: how many faces the walk may step through */
	uint64_t step_budget;     /* the most elementary steps the walk may take */
};

enum mw_reason
{
	MW_OK = 0,
	MW_NULL_SURFACES,
	MW_SURFACES_OUTSIDE, /* a surface the walk reads is outside the consolidated block */
	MW_NULL_ARRAY,       /* an array the walk must read is NULL */
	MW_UNOBSERVED,       /* the array's backing was not observed */
	MW_UNBACKED,         /* the read is outside the allocation */
	MW_UNINITIALISED,    /* the read is inside the allocation but past the data */
	MW_MISALIGNED,
	MW_OVERFLOW,         /* the address arithmetic would wrap */
	MW_BUDGET            /* the step budget ran out */
};

struct mw_result
{
	int reason;          /* MW_OK: every read of the whole walk, including the end call's, is backed */
	const char *array;   /* "faces", "point_list", "surface_points", "points", "surfaces" or "-" */
	int64_t safe_depths; /* depths 0 .. safe_depths-1 may be asked of the original; when reason is not MW_OK or end_backed is 0, depth safe_depths may not */
	int64_t triangles;   /* the depth count the faces consumed so far make TRUE (P below): the whole fan when complete */
	int complete;        /* every face was consumed: the walk reached count == number_of_faces */
	int end_backed;      /* the end call (depth == triangles) is safe: complete, and the next-surface read after the last face is backed */
	uint64_t faces_read, list_read, sp_read, points_read; /* the extents read, in elements, from the object's own pointers */
	uint64_t steps;
	int64_t face_points_sum;
	int face_points_min, face_points_max; /* over the faces consumed; -1, 0 when none */
};

/* surface record layout of `face_surface_description` (16 bytes): the flags word, whose byte 3 is number_of_points (0 means 256), and the u16 number_of_faces at byte 8 */
#define MW_SURFACE_BYTES 16u
#define MW_SURFACE_POINTS_AT 3u
#define MW_SURFACE_FACES_AT 8u

/* Which record bytes the loader's own freads filled (pure bookkeeping over the fread RESULTS; nothing is read from memory).
   The object loop fills a common surface record with seven separate freads whose destination bytes overlap (an int read into the 16-bit number_of_faces member spills over
   reflectivity and specularity). A field counts only when its fread returned 1. A record is fully initialised when the byte masks of the fields that succeeded cover all 16
   bytes; `prefix` is the number of leading records that are, counted in order, so a hole (a failed record followed by a complete one) never extends it. */
#define MW_FILL_FIELDS 7
static const uint16_t mw_fill_mask[MW_FILL_FIELDS] = { 0x000F, 0x0030, 0x00C0, 0x0F00, 0xF000, 0x0400, 0x0800 };

struct mw_fill
{
	uint32_t prefix;   /* leading fully initialised records */
	int32_t current;   /* the record being read + 1 (0: none) */
	uint16_t mask;     /* bytes of that record filled so far */
	uint16_t seen;     /* fields of that record already reported */
	int bad;           /* a field out of range, or one reported twice for the same record */
};

static void mw_fill_field (struct mw_fill *f, int surface, int field, size_t returned)
{
	if (field < 0 || field >= MW_FILL_FIELDS || surface < 0 || surface == 0x7fffffff)
	{
		f->bad = 1;
		return;
	}
	if (f->current != surface + 1)
	{
		f->current = surface + 1;
		f->mask = 0;
		f->seen = 0;
	}
	if (f->seen & (uint16_t) (1u << field))
	{
		f->bad = 1;
	}
	f->seen = (uint16_t) (f->seen | (1u << field));
	if (returned == 1)
	{
		f->mask = (uint16_t) (f->mask | mw_fill_mask[field]);
	}
	if (f->mask == 0xFFFFu && (uint32_t) surface == f->prefix)
	{
		f->prefix++;
	}
}

/* sets bits first .. first + count - 1 of a bitmap (the caller guarantees the bitmap holds them) */
static void mw_mark_records (unsigned char *bits, uint64_t first, uint64_t count)
{
	uint64_t r;
	for (r = 0; r < count; r++)
	{
		uint64_t record = first + r;
		bits[record >> 3] = (unsigned char) (bits[record >> 3] | (1u << (record & 7u)));
	}
}

/* 0: [addr, addr+size) is readable data in `r`; otherwise the reason */
static int mw_check (const struct mw_region *r, uintptr_t addr, uint64_t size, uint64_t align)
{
	uint64_t offset;

	if (!r->known)
	{
		return MW_UNOBSERVED;
	}
	if (align > 1 && (addr % align) != 0)
	{
		return MW_MISALIGNED;
	}
	if (addr < r->lo)
	{
		return MW_UNBACKED;
	}
	offset = (uint64_t) (addr - r->lo);
	if (offset > r->allocated || size > r->allocated - offset)
	{
		return MW_UNBACKED;
	}
	if (offset > r->initialised || size > r->initialised - offset)
	{
		return MW_UNINITIALISED;
	}
	return 0;
}

/* base + index * size, or 0 with *wrapped set */
static uintptr_t mw_address (uintptr_t base, uint64_t index, uint64_t size, int *wrapped)
{
	if (size == 0 || index > ((uint64_t) UINTPTR_MAX - (uint64_t) base) / size)
	{
		*wrapped = 1;
		return 0;
	}
	return base + (uintptr_t) (index * size);
}

static void mw_stop (struct mw_result *out, int reason, const char *array, int64_t safe_depths)
{
	out->reason = reason;
	out->array = array;
	out->safe_depths = safe_depths;
}

/* one read of one array element: the checks, the failure recorded when there is one. Returns 1 when readable. */
static int mw_read (struct mw_result *out, int64_t depth, uintptr_t ptr, const struct mw_region *r, uint64_t index, uint64_t size, uint64_t align, const char *array,
	uintptr_t *at)
{
	int wrapped = 0, why;

	if (!ptr)
	{
		mw_stop (out, MW_NULL_ARRAY, array, depth);
		return 0;
	}
	*at = mw_address (ptr, index, size, &wrapped);
	if (wrapped)
	{
		mw_stop (out, MW_OVERFLOW, array, depth);
		return 0;
	}
	why = mw_check (r, *at, size, align);
	if (why)
	{
		mw_stop (out, why, array, depth);
		return 0;
	}
	return 1;
}

/* The gate every read of a surface record passes BEFORE it is dereferenced (by the walk and by the caller's own metadata reads): k surfaces from the object's first. 0: the
   whole 16-byte record is inside the observed block, on a record boundary of it, 4-byte aligned, and was initialised by the loader; *at is its address. Otherwise the reason. */
static int mw_surface_gate (const struct mw_input *in, uint64_t k, uintptr_t *at_out)
{
	int wrapped = 0;
	uintptr_t at;
	uint64_t record;

	if (!in->surfaces_known)
	{
		return MW_UNOBSERVED;
	}
	at = mw_address (in->surfaces_ptr, k, MW_SURFACE_BYTES, &wrapped);
	if (wrapped)
	{
		return MW_OVERFLOW;
	}
	if (at < in->surfaces_lo || at >= in->surfaces_hi || (uint64_t) (in->surfaces_hi - at) < MW_SURFACE_BYTES)
	{
		return MW_SURFACES_OUTSIDE;
	}
	if ((in->surfaces_lo % 4u) != 0 || (at % 4u) != 0 || ((uint64_t) (at - in->surfaces_lo)) % MW_SURFACE_BYTES != 0)
	{
		return MW_MISALIGNED;
	}
	record = ((uint64_t) (at - in->surfaces_lo)) / MW_SURFACE_BYTES;
	if (!in->surfaces_init || record >= in->surfaces_init_records)
	{
		return MW_UNOBSERVED;
	}
	if (!((in->surfaces_init[record >> 3] >> (record & 7u)) & 1u))
	{
		return MW_UNINITIALISED;
	}
	*at_out = at;
	return 0;
}

/* a surface record's walk metadata (after the gate): 0, or the gate's reason */
static int mw_surface (const struct mw_input *in, uint64_t k, uint64_t *points, int64_t *faces)
{
	uintptr_t at = 0;
	const unsigned char *p;
	uint16_t number_of_faces;
	int why = mw_surface_gate (in, k, &at);

	if (why)
	{
		return why;
	}
	p = (const unsigned char *) at;
	memcpy (&number_of_faces, p + MW_SURFACE_FACES_AT, sizeof (number_of_faces));
	*points = p[MW_SURFACE_POINTS_AT] ? p[MW_SURFACE_POINTS_AT] : 256u;
	*faces = (int64_t) number_of_faces;
	return 0;
}

static void mw_walk (const struct mw_input *in, struct mw_result *out)
{
	uint64_t count, refs = 0, surface = 0, spb = 0, nsp = 0;
	int64_t nsf = 0;
	/* The accessor, called with depth D, consumes face k while (n_k - 3) < its running depth, subtracting (n_k - 2) from it each time (a face of 0 or 1 points ADDS 2 or 1).
	   With C_k = the sum of (n_j - 2) over the faces j < k consumed, it therefore stops at the first face k with D < C_{k+1}, and then reads that face's triangle t = D - C_k,
	   0 <= t <= n_k - 3 (the stop face always has n_k >= 3). If no face stops it the call consumes every face and returns FALSE. So the depths that return TRUE are
	   0 .. P-1 with P = the running maximum of C (never below 0), face k claims exactly the depths P_{k-1} .. C_{k+1} - 1 (none when C_{k+1} <= P_{k-1}), and the first FALSE
	   is depth P. `c` is C_k and `p` is P_{k-1}; neither assumes every increment is positive. */
	int64_t c = 0, p = 0;
	int why;

	memset (out, 0, sizeof (*out));
	out->array = "-";
	out->face_points_min = -1;
	out->end_backed = 0;

	/* obj->surfaces[0] is read unconditionally */
	if (!in->surfaces_ptr)
	{
		mw_stop (out, MW_NULL_SURFACES, "surfaces", 0);
		return;
	}
	why = mw_surface (in, 0, &nsp, &nsf);
	if (why)
	{
		mw_stop (out, why, "surfaces", 0);
		return;
	}
	if (in->number_of_faces == 0)
	{
		/* the loop body never runs: only surface 0 was read */
		out->complete = 1;
		out->end_backed = 1;
		return;
	}

	for (count = 0; count < in->number_of_faces; count++)
	{
		uintptr_t at;
		unsigned n;
		int64_t t, c_next, p_next;

		if (++out->steps > in->step_budget)
		{
			mw_stop (out, MW_BUDGET, "-", p);
			out->triangles = p;
			return;
		}
		/* faces[count].number_of_points: every depth >= p reaches this read */
		if (!mw_read (out, p, in->faces_ptr, &in->faces, count, 1, 1, "faces", &at))
		{
			out->triangles = p;
			return;
		}
		n = *(const unsigned char *) at;
		if (count + 1 > out->faces_read)
		{
			out->faces_read = count + 1;
		}
		c_next = c + (int64_t) n - 2;
		p_next = c_next > p ? c_next : p;
		/* The triangles this face claims (none for a face of 0, 1 or 2 points, or when earlier faces already claimed its depths): depth c + t, t from p - c, reads refs[0],
		   refs[t + 1], refs[t + 2] and, for each, the surface-point reference and the point. A face of 0 or 1 points is stepped exactly as the accessor steps it
		   (depth grows by 2 or 1, refs advance by n, the face count and the surface count advance); it is refused only if a read it causes is not backed. */
		for (t = p - c; t < (int64_t) n - 2; t++)
		{
			int k;
			for (k = 0; k < 3; k++)
			{
				uint64_t offset = refs + (k == 0 ? 0u : (uint64_t) t + (uint64_t) k);
				uintptr_t at_list, at_sp, at_point;
				uint64_t reference, point;
				uint16_t word;

				if (++out->steps > in->step_budget)
				{
					mw_stop (out, MW_BUDGET, "-", c + t);
					out->triangles = p;
					return;
				}
				if (!mw_read (out, c + t, in->list_ptr, &in->list, offset, 1, 1, "point_list", &at_list))
				{
					out->triangles = p;
					return;
				}
				reference = *(const unsigned char *) at_list;
				if (offset + 1 > out->list_read)
				{
					out->list_read = offset + 1;
				}
				if (!mw_read (out, c + t, in->sp_ptr, &in->sp, spb + reference, 2, 2, "surface_points", &at_sp))
				{
					out->triangles = p;
					return;
				}
				memcpy (&word, (const void *) at_sp, sizeof (word));
				point = word;
				if (spb + reference + 1 > out->sp_read)
				{
					out->sp_read = spb + reference + 1;
				}
				if (!mw_read (out, c + t, in->points_ptr, &in->points, point, 6, 2, "points", &at_point))
				{
					out->triangles = p;
					return;
				}
				if (point + 1 > out->points_read)
				{
					out->points_read = point + 1;
				}
			}
		}
		/* consume the face */
		out->face_points_sum += n;
		if (out->face_points_min < 0 || (int) n < out->face_points_min)
		{
			out->face_points_min = (int) n;
		}
		if ((int) n > out->face_points_max)
		{
			out->face_points_max = (int) n;
		}
		c = c_next;
		p = p_next;
		refs += n;
		nsf--;
		if (nsf == 0)
		{
			/* the next surface, read here even after the last face (the end call reads it too); every depth >= p passes this face and so makes the read */
			spb += nsp;
			surface++;
			why = mw_surface (in, surface, &nsp, &nsf);
			if (why)
			{
				out->triangles = p;
				if (count + 1 == in->number_of_faces)
				{
					/* the walk is whole; only the end call's read of the next surface is not backed */
					out->complete = 1;
					out->end_backed = 0;
				}
				mw_stop (out, why, "surfaces", p);
				return;
			}
		}
	}
	out->complete = 1;
	out->end_backed = 1;
	out->triangles = p;
	out->safe_depths = p;
}

#endif
