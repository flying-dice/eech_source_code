/*
 * TEST-ONLY ground-contact characterisation (EEDCS-48, D3 G2). Compiled only with the `ground-fixture` feature; never part of the
 * shipping module. It changes no production source: it calls the ORIGINAL macros point_above_ground and point_below_ground
 * (en_world.h) and the ORIGINAL get_ground_impact_point (en_cllsn.c), the same expressions and the same call shape the hosted weapon
 * lookups of wn_move.c use (CAMPAIGN_STEP_LOOKUP_GROUND_PROBE: point_below_ground (&p); CAMPAIGN_STEP_LOOKUP_GROUND_IMPACT:
 * get_ground_impact_point (&p, &q, &q), the impact aliased with the below endpoint), against the terrain the campaign's own boot
 * loaded. The lookup wrapper itself (answer_weapon_step_lookup) is static to wn_move.c and is entered only by a launched weapon's
 * flight step, so it is not entered here; the two glue lines it adds (a bool to 1.0 or 0.0, the class to a float) are not exercised.
 *
 * What it observes, per case, without inferring anything from the source text:
 *   - the boolean or terrain class each original call returns, and the bit patterns of every vec3d it can write;
 *   - the shared terrain point record (terrelev.c, system_point_data: the record every NULL-point-data terrain query writes). Before
 *     each call the record is set to a known sentinel; afterwards it is compared with that sentinel, byte for byte. A record that
 *     is still the sentinel shows that the call never queried the terrain (the sector-maximum clause or the off-map clause
 *     short-circuited); a changed record shows the elevation query ran, and the value words it holds (elevation, type, face normal,
 *     whether a sector and a face are held) are folded and sampled. No pointer or address is ever emitted;
 *   - the impact point with the output aliased with the below endpoint (the caller's shape) and with a separate output, and
 *     whether the two agree;
 *   - a warm repeat (the very next query, on exactly the record the first call left, which is checked to be unchanged before the repeat)
 *     and whether it agrees;
 *   - whether the endpoints, after the call, satisfy the original function's own ASSERT conditions (checked by the same macros,
 *     outside the function, because ASSERT is a non-fatal write_debug_log in this build and would otherwise go unseen). These are
 *     extra queries and are made only after every call being characterised, because point_above_ground can write the shared record.
 *
 * Three kinds of fact are kept apart in the report (docs/m5-ground-characterization.md, "What can be compared"):
 *   - PHYSICAL: the inputs a query was given, its booleans and classes, and the vectors it wrote. `ground-physical` folds only these.
 *   - SHORT-CIRCUIT and CACHE: whether a call read the terrain (record touched), and whether a warm repeat agrees. Counters only.
 *   - LEGACY record diagnostics: the shared record's value words, sector and triangle indices and held-reference bits. `ground-hash`, the
 *     `record_*` words and the `elev`/`type` sample fields carry them; they exist only because the original record exists.
 *
 * The points come from a private xorshift generator, so the probe never touches the game's random stream; the terrain heights it
 * uses to place points come from a private point-data record. The probe runs inside the engine entry, so it sees EECH's
 * round-toward-zero mode. At its start it snapshots the shared record and at its end writes the snapshot back byte for byte.
 *
 * Preconditions (a probe that does not hold them is refused, with its reason in the report, before the record is altered):
 *   - a terrain is loaded, and the world map's extents are the terrain's own (the macros read the world map, the terrain functions
 *     read the terrain's: off the terrain, point_above_ground stops the engine in debug_fatal);
 *   - the FPU is in round-toward-zero mode, as inside every engine entry;
 *   - the shared record can be located and written through the public accessors and reads back what was written.
 * Unsupported, never probed: any point outside the map area for point_above_ground or for get_ground_impact_point (the sector
 * maximum is read first and stops the engine); NaN or infinite coordinates. point_below_ground IS probed outside the map area:
 * its first clause answers without reading the terrain.
 */

#ifdef EECH_GROUND_FIXTURE

#include <fenv.h>
#include <math.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "project.h"

#include "eech_engine.h"
#include "eech_headless.h"

#define FNV_OFFSET 14695981039346656037ull
#define FNV_PRIME 1099511628211ull

#define IMPACT_TYPES 8
#define SENTINEL_ELEVATION_BITS 0x5a5a5a5au
#define SENTINEL_SECTOR (-1)

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* hashing and report text */

static uint64_t fold64 (uint64_t hash, uint32_t word)
{
	int byte;
	for (byte = 0; byte < 4; byte++)
	{
		hash ^= (uint64_t) ((word >> (8 * byte)) & 0xffu);
		hash *= FNV_PRIME;
	}
	return hash;
}

static uint32_t float_bits (float value)
{
	uint32_t bits;
	memcpy (&bits, &value, sizeof (bits));
	return bits;
}

static float bits_float (uint32_t bits)
{
	float value;
	memcpy (&value, &bits, sizeof (value));
	return value;
}

/*
 * Two separate folds, never mixed (the report names them separately, and a difference in either is classified on its own):
 *   - the LEGACY fold (`combined`, `above`, `below`, `impact`; report line `ground-hash`) is the original diagnostic fold. It includes the
 *     shared record's private bookkeeping (sector and triangle indices, whether a sector and a face are held, via fold_view), so it is only
 *     comparable between two builds that keep that very record. It is unchanged by the physical fold below.
 *   - the PHYSICAL fold (`phys_*`; report line `ground-physical`) folds only what a query was given and what it answered: the input
 *     vectors, the booleans and classes, the vectors the call wrote (endpoint corrections and the impact point). It excludes the record's
 *     bookkeeping, the short-circuit facts (whether the record was touched) and the cache facts (warm repeats), which are reported as
 *     counters of their own, so it can compare a later candidate that has no such record.
 */
struct hashes
{
	uint64_t combined, above, below, impact;
	uint64_t phys_combined, phys_above, phys_below, phys_impact;
};

enum { KIND_ABOVE, KIND_BELOW, KIND_IMPACT };

static void pfold (struct hashes *h, int kind, uint32_t word)
{
	h->phys_combined = fold64 (h->phys_combined, word);
	switch (kind)
	{
	case KIND_ABOVE:
		h->phys_above = fold64 (h->phys_above, word);
		break;
	case KIND_BELOW:
		h->phys_below = fold64 (h->phys_below, word);
		break;
	default:
		h->phys_impact = fold64 (h->phys_impact, word);
		break;
	}
}

static void pfold_vec (struct hashes *h, int kind, const vec3d *v)
{
	pfold (h, kind, float_bits (v->x));
	pfold (h, kind, float_bits (v->y));
	pfold (h, kind, float_bits (v->z));
}

static void fold (struct hashes *h, int kind, uint32_t word)
{
	h->combined = fold64 (h->combined, word);
	switch (kind)
	{
	case KIND_ABOVE:
		h->above = fold64 (h->above, word);
		break;
	case KIND_BELOW:
		h->below = fold64 (h->below, word);
		break;
	default:
		h->impact = fold64 (h->impact, word);
		break;
	}
}

static void fold_vec (struct hashes *h, int kind, const vec3d *v)
{
	fold (h, kind, float_bits (v->x));
	fold (h, kind, float_bits (v->y));
	fold (h, kind, float_bits (v->z));
}

struct out
{
	char *text;
	int capacity, used, overflow;
};

static void emit (struct out *out, const char *format, ...) __attribute__ ((format (printf, 2, 3)));

static void emit (struct out *out, const char *format, ...)
{
	va_list args;
	int n;
	if (out->overflow)
	{
		return;
	}
	va_start (args, format);
	n = vsnprintf (out->text + out->used, (size_t) (out->capacity - out->used), format, args);
	va_end (args);
	if (n < 0 || n >= out->capacity - out->used)
	{
		out->overflow = 1;
		return;
	}
	out->used += n;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* private generator, as the line-of-sight probe's: never the game's random stream */

static uint32_t next_random (uint32_t *state)
{
	uint32_t x = *state;
	x ^= x << 13;
	x ^= x >> 17;
	x ^= x << 5;
	*state = x;
	return x;
}

static float unit (uint32_t *state)
{
	return (float) (next_random (state) & 0xffffffu) / 16777216.0f;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the shared terrain point record (terrelev.c system_point_data), reached through the public accessor that returns the address of its
   face normal; the record's own layout is public in terrdata.h, so the start of the record is that address less the member's offset */

static terrain_3d_point_data *shared_record (void)
{
	return (terrain_3d_point_data *) ((char *) get_3d_terrain_point_data_normal (NULL) - offsetof (terrain_3d_point_data, face_normal));
}

static void make_sentinel (terrain_3d_point_data *sentinel)
{
	memset (sentinel, 0, sizeof (*sentinel));
	sentinel->x_sector = SENTINEL_SECTOR;
	sentinel->z_sector = SENTINEL_SECTOR;
	sentinel->last_elevation = bits_float (SENTINEL_ELEVATION_BITS);
	sentinel->terrain_triangle_offset = SENTINEL_SECTOR;
	sentinel->terrain_type = TERRAIN_TYPE_INVALID;
}

/* what a record holds, as value words: never an address. `held` is whether a sector and a face are referenced. */
struct record_view
{
	int touched;
	uint32_t elevation_bits, normal_bits[3];
	int type, x_sector, z_sector, triangle_offset, sector_held, face_held;
};

static void view_record (const terrain_3d_point_data *record, const terrain_3d_point_data *sentinel, struct record_view *view)
{
	view->touched = memcmp (record, sentinel, sizeof (*record)) != 0;
	view->elevation_bits = float_bits (record->last_elevation);
	view->normal_bits[0] = float_bits (record->face_normal.x);
	view->normal_bits[1] = float_bits (record->face_normal.y);
	view->normal_bits[2] = float_bits (record->face_normal.z);
	view->type = (int) record->terrain_type;
	view->x_sector = record->x_sector;
	view->z_sector = record->z_sector;
	view->triangle_offset = record->terrain_triangle_offset;
	view->sector_held = record->terrain_sector != NULL;
	view->face_held = record->terrain_face != NULL;
}

static void fold_view (struct hashes *h, int kind, const struct record_view *view)
{
	fold (h, kind, (uint32_t) view->touched);
	fold (h, kind, view->elevation_bits);
	fold (h, kind, view->normal_bits[0]);
	fold (h, kind, view->normal_bits[1]);
	fold (h, kind, view->normal_bits[2]);
	fold (h, kind, (uint32_t) view->type);
	fold (h, kind, (uint32_t) view->x_sector);
	fold (h, kind, (uint32_t) view->z_sector);
	fold (h, kind, (uint32_t) view->triangle_offset);
	fold (h, kind, (uint32_t) (view->sector_held | (view->face_held << 1)));
}

/* the value words of a record, folded: the same fold before and after the probe shows that the probe left the record as it found it */
static uint64_t record_words_hash (const terrain_3d_point_data *record)
{
	struct record_view view;
	uint64_t hash = FNV_OFFSET;
	view_record (record, record, &view);
	hash = fold64 (hash, view.elevation_bits);
	hash = fold64 (hash, view.normal_bits[0]);
	hash = fold64 (hash, view.normal_bits[1]);
	hash = fold64 (hash, view.normal_bits[2]);
	hash = fold64 (hash, (uint32_t) view.type);
	hash = fold64 (hash, (uint32_t) view.x_sector);
	hash = fold64 (hash, (uint32_t) view.z_sector);
	hash = fold64 (hash, (uint32_t) view.triangle_offset);
	hash = fold64 (hash, (uint32_t) (view.sector_held | (view.face_held << 1)));
	return hash;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* terrain facts used to PLACE points; they come from a private record, never the shared one */

static float terrain_height (float x, float z)
{
	terrain_3d_point_data point;
	memset (&point, 0, sizeof (point));
	return get_3d_terrain_point_data (x, z, &point);
}

static float next_down (float v)
{
	return nextafterf (v, -INFINITY);
}

static float next_up (float v)
{
	return nextafterf (v, INFINITY);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* counters and samples */

/* the first POINT_SAMPLES_PER_TAG point cases of each tag and the first IMPACT_SAMPLES_PER_TYPE impact cases of each type are sampled in full */
#define POINT_TAGS 6
#define POINT_SAMPLES_PER_TAG 2
#define IMPACT_SAMPLES_PER_TYPE 4

struct sample_book
{
	int point_by_tag[POINT_TAGS], impact_by_type[IMPACT_TYPES];
};

struct tally
{
	/* point cases */
	int points, off_map_cases, input_mutated;
	int above_cases, above_true, above_touched, above_true_untouched, above_warm_same;
	int below_cases, below_true, below_touched, below_true_untouched, below_warm_same, off_map_below_true, off_map_below_touched;
	int in_map_both, complement, not_complement, both_true, both_false;
	/* impact cases */
	int impacts, by_type[IMPACT_TYPES], class_count[8], class_other;
	int a_changed, b_changed, xz_changed, alias_equal, alias_unequal, warm_same, warm_different, touched, untouched;
	int impact_is_below_endpoint, post_above_not_above, assert_below_violations;
	/* the warm repeat began on exactly the record the cold call left (nothing ran between them), per family */
	int above_warm_intact, below_warm_intact, impact_warm_intact;
};

struct probe
{
	struct hashes hashes;
	struct tally tally;
	struct sample_book samples;
	struct out *out;
	terrain_3d_point_data *record;
	terrain_3d_point_data sentinel;
	float max_x, max_z;
	/* the PHYSICAL terrain's sector grid (TERRAIN_3D_SECTOR_SIDE_LENGTH metres a side): the grid the sector boundaries and the sector maximum belong to */
	int sectors_x, sectors_z;
	/* the CAMPAIGN's grid (world_map): it only fixes the map extents, and need not be the terrain's grid; reported, never used to place a physical boundary */
	int world_sectors_x, world_sectors_z, world_side;
	uint32_t rng;
};

static int bits_equal3 (const vec3d *a, const vec3d *b)
{
	return float_bits (a->x) == float_bits (b->x) && float_bits (a->y) == float_bits (b->y) && float_bits (a->z) == float_bits (b->z);
}

static void arm (struct probe *p)
{
	memcpy (p->record, &p->sentinel, sizeof (*p->record));
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* point cases: the two original macros */

static void point_case (struct probe *p, int n, int tag, vec3d point)
{
	struct tally *t = &p->tally;
	vec3d before = point;
	struct record_view above_view, below_view;
	terrain_3d_point_data left;
	int inside = point_inside_map_area (&point) ? 1 : 0;
	int above = -1, below, above_again = -1, below_again;

	t->points++;
	fold (&p->hashes, KIND_ABOVE, 0x50000000u | (uint32_t) tag);
	fold_vec (&p->hashes, KIND_ABOVE, &point);
	fold (&p->hashes, KIND_BELOW, 0x50000000u | (uint32_t) tag);
	fold_vec (&p->hashes, KIND_BELOW, &point);

	memset (&above_view, 0, sizeof (above_view));
	if (inside)
	{
		/* point_above_ground reads the sector maximum, which stops the engine off the terrain: only ever asked inside the map area */
		arm (p);
		above = point_above_ground (&point) ? 1 : 0;
		/* the warm repeat is the next query after the cold one, on exactly the record the cold call left: the record is copied at once, the
		   (read-only) observation runs, and the warm call is made only after the record is checked to be that copy */
		memcpy (&left, p->record, sizeof (left));
		view_record (p->record, &p->sentinel, &above_view);
		t->above_warm_intact += memcmp (p->record, &left, sizeof (left)) == 0;
		above_again = point_above_ground (&point) ? 1 : 0;
		t->above_cases++;
		t->above_true += above;
		t->above_touched += above_view.touched;
		if (above && !above_view.touched)
		{
			t->above_true_untouched++;
		}
		if (above_again == above)
		{
			t->above_warm_same++;
		}
		fold (&p->hashes, KIND_ABOVE, (uint32_t) above);
		fold (&p->hashes, KIND_ABOVE, (uint32_t) above_again);
		fold_view (&p->hashes, KIND_ABOVE, &above_view);
		/* physical: the input as given, the cold answer, the input after the call (a mutation would show) */
		pfold_vec (&p->hashes, KIND_ABOVE, &before);
		pfold (&p->hashes, KIND_ABOVE, (uint32_t) above);
		pfold_vec (&p->hashes, KIND_ABOVE, &point);
	}
	else
	{
		t->off_map_cases++;
		fold (&p->hashes, KIND_ABOVE, 0xffffffffu);
		pfold_vec (&p->hashes, KIND_ABOVE, &before);
		pfold (&p->hashes, KIND_ABOVE, 0xffffffffu);
	}

	arm (p);
	below = point_below_ground (&point) ? 1 : 0;
	memcpy (&left, p->record, sizeof (left));
	view_record (p->record, &p->sentinel, &below_view);
	t->below_warm_intact += memcmp (p->record, &left, sizeof (left)) == 0;
	below_again = point_below_ground (&point) ? 1 : 0;
	t->below_cases++;
	t->below_true += below;
	t->below_touched += below_view.touched;
	if (below && !below_view.touched)
	{
		t->below_true_untouched++;
	}
	if (below_again == below)
	{
		t->below_warm_same++;
	}
	if (!inside)
	{
		t->off_map_below_true += below;
		t->off_map_below_touched += below_view.touched;
	}
	fold (&p->hashes, KIND_BELOW, (uint32_t) below);
	fold (&p->hashes, KIND_BELOW, (uint32_t) below_again);
	fold (&p->hashes, KIND_BELOW, (uint32_t) inside);
	fold_view (&p->hashes, KIND_BELOW, &below_view);
	pfold_vec (&p->hashes, KIND_BELOW, &before);
	pfold (&p->hashes, KIND_BELOW, (uint32_t) below);
	pfold_vec (&p->hashes, KIND_BELOW, &point);

	if (inside)
	{
		t->in_map_both++;
		if (above && below)
		{
			t->both_true++;
		}
		if (!above && !below)
		{
			t->both_false++;
		}
		if (above == !below)
		{
			t->complement++;
		}
		else
		{
			t->not_complement++;
		}
	}
	if (!bits_equal3 (&before, &point))
	{
		t->input_mutated++;
	}

	if (tag >= 0 && tag < POINT_TAGS && p->samples.point_by_tag[tag] < POINT_SAMPLES_PER_TAG)
	{
		p->samples.point_by_tag[tag]++;
		emit (p->out,
			"ground-sample kind=point n=%d tag=%d x=%08x y=%08x z=%08x inside=%d above=%d above_touched=%d below=%d below_touched=%d above_elev=%08x above_type=%d "
			"below_elev=%08x below_type=%d\n",
			n, tag, float_bits (point.x), float_bits (point.y), float_bits (point.z), inside, above, above_view.touched, below, below_view.touched,
			above_view.elevation_bits, above_view.type, below_view.elevation_bits, below_view.type);
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* impact cases: the original function, aliased (the caller's shape) and separate */

static void impact_case (struct probe *p, int n, int type, vec3d a0, vec3d b0)
{
	struct tally *t = &p->tally;
	vec3d pa = a0, qa = b0;
	vec3d pb = a0, qb = b0, rb;
	vec3d pw = a0, qw = b0, rw;
	struct record_view alias_view, separate_view;
	terrain_3d_point_data left;
	int class_alias, class_separate, class_warm;
	int equal, warm, a_changed, b_changed, xz_changed;

	rb.x = rb.y = rb.z = -777.0f;
	rw.x = rw.y = rw.z = -777.0f;

	t->impacts++;
	t->by_type[type]++;

	arm (p);
	class_alias = (int) get_ground_impact_point (&pa, &qa, &qa);
	view_record (p->record, &p->sentinel, &alias_view);

	arm (p);
	class_separate = (int) get_ground_impact_point (&pb, &qb, &rb);
	/* the warm call is the very next query after the separate one, on exactly the record the separate call left: the record is copied at once,
	   the (read-only) observation runs, and the warm call is made only after the record is checked to be that copy. The diagnostic predicate
	   checks below run AFTER the warm call, because point_above_ground can write the shared record. */
	memcpy (&left, p->record, sizeof (left));
	view_record (p->record, &p->sentinel, &separate_view);
	t->impact_warm_intact += memcmp (p->record, &left, sizeof (left)) == 0;

	/* warm: the same separate call again, on the record as the separate call left it */
	class_warm = (int) get_ground_impact_point (&pw, &qw, &rw);

	/* the original function's ASSERT (!point_above_ground (below_ground)) on the corrected below endpoint, asked outside it because the build's
	   ASSERT is a no-op log; and, not an original assert, whether the corrected above endpoint is above the ground. They are extra queries, made
	   after every call being characterised, so they cannot change what those calls saw. */
	if (!point_above_ground (&pb))
	{
		t->post_above_not_above++;
	}
	if (point_above_ground (&qb))
	{
		t->assert_below_violations++;
	}

	a_changed = !bits_equal3 (&pb, &a0);
	b_changed = !bits_equal3 (&qb, &b0);
	xz_changed = (float_bits (pb.x) != float_bits (a0.x)) || (float_bits (pb.z) != float_bits (a0.z)) || (float_bits (qb.x) != float_bits (b0.x))
		|| (float_bits (qb.z) != float_bits (b0.z));
	equal = (class_alias == class_separate) && bits_equal3 (&pa, &pb) && bits_equal3 (&qa, &rb);
	warm = (class_warm == class_separate) && bits_equal3 (&rw, &rb) && bits_equal3 (&pw, &pb) && bits_equal3 (&qw, &qb);

	t->a_changed += a_changed;
	t->b_changed += b_changed;
	t->xz_changed += xz_changed;
	t->alias_equal += equal;
	t->alias_unequal += !equal;
	t->warm_same += warm;
	t->warm_different += !warm;
	t->touched += alias_view.touched && separate_view.touched;
	t->untouched += !(alias_view.touched && separate_view.touched);
	if (bits_equal3 (&rb, &qb))
	{
		t->impact_is_below_endpoint++;
	}
	if (class_separate >= 0 && class_separate < 8)
	{
		t->class_count[class_separate]++;
	}
	else
	{
		t->class_other++;
	}

	fold (&p->hashes, KIND_IMPACT, 0x49000000u | (uint32_t) type);
	fold_vec (&p->hashes, KIND_IMPACT, &a0);
	fold_vec (&p->hashes, KIND_IMPACT, &b0);
	fold (&p->hashes, KIND_IMPACT, (uint32_t) class_alias);
	fold_vec (&p->hashes, KIND_IMPACT, &pa);
	fold_vec (&p->hashes, KIND_IMPACT, &qa);
	fold_view (&p->hashes, KIND_IMPACT, &alias_view);
	fold (&p->hashes, KIND_IMPACT, (uint32_t) class_separate);
	fold_vec (&p->hashes, KIND_IMPACT, &pb);
	fold_vec (&p->hashes, KIND_IMPACT, &qb);
	fold_vec (&p->hashes, KIND_IMPACT, &rb);
	fold_view (&p->hashes, KIND_IMPACT, &separate_view);
	fold (&p->hashes, KIND_IMPACT, (uint32_t) (equal | (warm << 1)));

	/* physical: what the calls were given and answered, both output shapes, and the endpoint corrections; no record bookkeeping, no
	   short-circuit or cache facts (those are the counters) */
	pfold_vec (&p->hashes, KIND_IMPACT, &a0);
	pfold_vec (&p->hashes, KIND_IMPACT, &b0);
	pfold (&p->hashes, KIND_IMPACT, (uint32_t) class_alias);
	pfold_vec (&p->hashes, KIND_IMPACT, &pa);
	pfold_vec (&p->hashes, KIND_IMPACT, &qa);
	pfold (&p->hashes, KIND_IMPACT, (uint32_t) class_separate);
	pfold_vec (&p->hashes, KIND_IMPACT, &pb);
	pfold_vec (&p->hashes, KIND_IMPACT, &qb);
	pfold_vec (&p->hashes, KIND_IMPACT, &rb);

	if (p->samples.impact_by_type[type] < IMPACT_SAMPLES_PER_TYPE)
	{
		p->samples.impact_by_type[type]++;
		emit (p->out,
			"ground-sample kind=impact n=%d type=%d a0=%08x,%08x,%08x b0=%08x,%08x,%08x class_alias=%d pa=%08x,%08x,%08x qa=%08x,%08x,%08x class_separate=%d "
			"pb=%08x,%08x,%08x qb=%08x,%08x,%08x rb=%08x,%08x,%08x alias_equal=%d warm_same=%d touched=%d elev=%08x type_word=%d\n",
			n, type, float_bits (a0.x), float_bits (a0.y), float_bits (a0.z), float_bits (b0.x), float_bits (b0.y), float_bits (b0.z), class_alias,
			float_bits (pa.x), float_bits (pa.y), float_bits (pa.z), float_bits (qa.x), float_bits (qa.y), float_bits (qa.z), class_separate,
			float_bits (pb.x), float_bits (pb.y), float_bits (pb.z), float_bits (qb.x), float_bits (qb.y), float_bits (qb.z), float_bits (rb.x),
			float_bits (rb.y), float_bits (rb.z), equal, warm, separate_view.touched, separate_view.elevation_bits, separate_view.type);
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* case generation */

/* a coordinate at, or one float either side of, a sector boundary or a map edge, or in the interior */
static float interior_coord (struct probe *p, float maximum)
{
	return unit (&p->rng) * maximum;
}

static float edge_coord (struct probe *p, float maximum)
{
	switch (next_random (&p->rng) % 8)
	{
	case 0:
		return 0.0f;
	case 1:
		return -0.0f;
	case 2:
		return next_up (0.0f);
	case 3:
		return 1.0f;
	case 4:
		return maximum;
	case 5:
		return next_down (maximum);
	case 6:
		return maximum - 1.0f;
	default:
		return maximum < 2048.0f ? maximum : 2048.0f;
	}
}

static float boundary_coord (struct probe *p, float maximum, int sectors)
{
	float v = (float) (next_random (&p->rng) % (uint32_t) (sectors + 1)) * (float) TERRAIN_3D_SECTOR_SIDE_LENGTH;
	if (v > maximum)
	{
		v = maximum;
	}
	switch (next_random (&p->rng) % 3)
	{
	case 0:
		return v;
	case 1:
		return v > 0.0f ? next_down (v) : v;
	default:
		return v < maximum ? next_up (v) : v;
	}
}

static float off_map_coord (struct probe *p, float maximum)
{
	switch (next_random (&p->rng) % 7)
	{
	case 0:
		return -1.0f;
	case 1:
		return next_down (0.0f);
	case 2:
		return next_up (maximum);
	case 3:
		return maximum + 1.0f;
	case 4:
		return maximum * 2.0f;
	case 5:
		return -1.0e9f;
	default:
		return 1.0e9f;
	}
}

/* heights relative to the terrain under the point and to its sector maximum: below, at and above each, and a float either side */
static float point_height (struct probe *p, float x, float z, int choice)
{
	float e = terrain_height (x, z);
	float m = get_terrain_3d_point_sector_max_elevation (x, z);
	switch (choice)
	{
	case 0:
		return e - 500.0f;
	case 1:
		return e - 1.0f;
	case 2:
		return e - 0.001f;
	case 3:
		return next_down (e);
	case 4:
		return e;
	case 5:
		return next_up (e);
	case 6:
		return e + 0.001f;
	case 7:
		return e + 1.0f;
	case 8:
		return e + 500.0f;
	case 9:
		return next_down (m);
	case 10:
		return m;
	case 11:
		return next_up (m);
	case 12:
		return m - 1.0f;
	case 13:
		return m + 1.0f;
	case 14:
		return MIN_MAP_Y;
	default:
		return MAX_MAP_Y;
	}
}

static void make_point (struct probe *p, int n)
{
	static const float off_map_heights[] = { -100.0f, 0.0f, 100.0f, 5000.0f };
	vec3d v;
	int tag = (int) (next_random (&p->rng) % 6);
	int choice = (int) (next_random (&p->rng) % 16);
	switch (tag)
	{
	case 0:
	case 1:
		v.x = interior_coord (p, p->max_x);
		v.z = interior_coord (p, p->max_z);
		break;
	case 2:
		v.x = boundary_coord (p, p->max_x, p->sectors_x);
		v.z = (next_random (&p->rng) & 1) ? boundary_coord (p, p->max_z, p->sectors_z) : interior_coord (p, p->max_z);
		break;
	case 3:
		v.x = edge_coord (p, p->max_x);
		v.z = (next_random (&p->rng) & 1) ? edge_coord (p, p->max_z) : interior_coord (p, p->max_z);
		break;
	case 4:
		v.x = interior_coord (p, p->max_x);
		v.z = edge_coord (p, p->max_z);
		break;
	default:
		/* outside the map area: only point_below_ground is asked; its first clause answers before any terrain read */
		if (next_random (&p->rng) & 1)
		{
			v.x = off_map_coord (p, p->max_x);
			v.z = interior_coord (p, p->max_z);
		}
		else
		{
			v.x = interior_coord (p, p->max_x);
			v.z = off_map_coord (p, p->max_z);
		}
		v.y = off_map_heights[next_random (&p->rng) % 4];
		point_case (p, n, tag, v);
		return;
	}
	if (!point_inside_map_area (&v))
	{
		/* a generated point that is not inside the map area is never given to the terrain functions */
		v.y = 0.0f;
		point_case (p, n, tag, v);
		return;
	}
	v.y = point_height (p, v.x, v.z, choice);
	point_case (p, n, tag, v);
}

static float clamp_to (float v, float maximum)
{
	if (v < 0.0f)
	{
		return 0.0f;
	}
	return v > maximum ? maximum : v;
}

static void make_impact (struct probe *p, int n)
{
	static const float ranges[] = { 0.25f, 0.9f, 1.0f, 1.1f, 2.0f, 10.0f, 100.0f, 1000.0f, 10000.0f };
	static const float above_heights[] = { 0.5f, 1.0f, 5.0f, 50.0f, 500.0f, 3000.0f };
	static const float below_depths[] = { 0.5f, 1.0f, 5.0f, 50.0f, 500.0f };
	int type = (int) (next_random (&p->rng) % IMPACT_TYPES);
	float range = ranges[next_random (&p->rng) % (sizeof (ranges) / sizeof (ranges[0]))];
	float h = above_heights[next_random (&p->rng) % (sizeof (above_heights) / sizeof (above_heights[0]))];
	float d = below_depths[next_random (&p->rng) % (sizeof (below_depths) / sizeof (below_depths[0]))];
	vec3d a, b;
	float ea, eb;

	a.x = interior_coord (p, p->max_x);
	a.z = interior_coord (p, p->max_z);
	b.x = clamp_to (a.x + (unit (&p->rng) - 0.5f) * 2.0f * range, p->max_x);
	b.z = clamp_to (a.z + (unit (&p->rng) - 0.5f) * 2.0f * range, p->max_z);
	if (type == 4)
	{
		b.x = a.x;
		b.z = a.z;
	}
	else if (type == 7 && p->sectors_x >= 2)
	{
		/* an above endpoint one float short of a sector boundary and a below endpoint on it */
		float boundary = (float) (1 + next_random (&p->rng) % (uint32_t) (p->sectors_x - 1)) * (float) TERRAIN_3D_SECTOR_SIDE_LENGTH;
		a.x = next_down (boundary);
		b.x = boundary;
	}
	ea = terrain_height (a.x, a.z);
	eb = terrain_height (b.x, b.z);
	a.y = ea + h;
	b.y = eb - d;
	switch (type)
	{
	case 1: /* the above endpoint is below the ground: the first correction */
		a.y = ea - d;
		break;
	case 2: /* the below endpoint is above the ground: the second correction */
		b.y = eb + h;
		break;
	case 3: /* both misclassified */
		a.y = ea - d;
		b.y = eb + h;
		break;
	case 5: /* both exactly at the terrain elevation: equality */
		a.y = ea;
		b.y = eb;
		break;
	case 6: /* the above endpoint exactly at its sector maximum */
		a.y = get_terrain_3d_point_sector_max_elevation (a.x, a.z);
		break;
	default:
		break;
	}
	impact_case (p, n, type, a, b);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */

static void refuse (char *text, int capacity, const char *reason)
{
	snprintf (text, (size_t) capacity, "ground-refused reason=%s\n", reason);
}

/*
 * Runs the point cases and the impact cases and writes the report. Returns 0, or -1 with `ground-refused reason=...` in the
 * report (a precondition failed before the record was altered) or with nothing useful when the report does not fit.
 */
int eech_ground_probe_run (uint32_t seed, int points, int impacts, char *text, int capacity)
{
	static struct probe probe;
	struct probe *p = &probe;
	struct out out;
	terrain_3d_point_data snapshot;
	struct tally *t;
	uint64_t record_hash_before, record_hash_after;
	int i, restored, anchored;

	text[0] = '\0';
	if (fegetround () != FE_TOWARDZERO)
	{
		refuse (text, capacity, "fpu_not_round_toward_zero");
		return -1;
	}
	if (!terrain_sectors || terrain_3d_sector_x_max <= 0 || terrain_3d_sector_z_max <= 0)
	{
		refuse (text, capacity, "no_terrain_loaded");
		return -1;
	}
	/* The macros read the world map (set from the campaign's own grid: x sectors * sector side - 1) and the terrain functions read the terrain's extents (terrain
	   sectors * TERRAIN_3D_SECTOR_SIDE_LENGTH - 1). Equal extents do NOT need equal grids: a campaign grid of N sectors of 4096 m covers the same ground as a
	   terrain grid of 2N sectors of 2048 m. Only the extents matter to the off-map fatal, so only they are required to agree (bit for bit); the two grids are
	   different things and are reported separately. */
	if (float_bits (MIN_MAP_X) != float_bits (terrain_3d_min_map_x) || float_bits (MIN_MAP_Z) != float_bits (terrain_3d_min_map_z)
		|| float_bits (MAX_MAP_X) != float_bits (terrain_3d_max_map_x) || float_bits (MAX_MAP_Z) != float_bits (terrain_3d_max_map_z))
	{
		refuse (text, capacity, "world_map_is_not_the_terrain_extents");
		return -1;
	}
	if (!(MAX_MAP_X > 0.0f) || !(MAX_MAP_Z > 0.0f) || NUM_MAP_X_SECTORS < 1 || NUM_MAP_Z_SECTORS < 1 || SECTOR_SIDE_LENGTH < 1)
	{
		refuse (text, capacity, "empty_map");
		return -1;
	}

	memset (p, 0, sizeof (*p));
	p->hashes.combined = p->hashes.above = p->hashes.below = p->hashes.impact = FNV_OFFSET;
	p->hashes.phys_combined = p->hashes.phys_above = p->hashes.phys_below = p->hashes.phys_impact = FNV_OFFSET;
	p->record = shared_record ();
	p->max_x = MAX_MAP_X;
	p->max_z = MAX_MAP_Z;
	/* sector boundaries are PHYSICAL terrain boundaries, so they are counted in the terrain's grid, not the campaign's */
	p->sectors_x = terrain_3d_sector_x_max;
	p->sectors_z = terrain_3d_sector_z_max;
	p->world_sectors_x = NUM_MAP_X_SECTORS;
	p->world_sectors_z = NUM_MAP_Z_SECTORS;
	p->world_side = SECTOR_SIDE_LENGTH;
	p->rng = seed ? seed : 1u;
	make_sentinel (&p->sentinel);

	/* locate the record and prove the address: write distinct words through it and read them back through the public accessors, then restore */
	memcpy (&snapshot, p->record, sizeof (snapshot));
	record_hash_before = record_words_hash (&snapshot);
	p->record->last_elevation = bits_float (SENTINEL_ELEVATION_BITS);
	p->record->terrain_type = (terrain_types) 1;
	anchored = (float_bits (get_3d_terrain_point_data_elevation (NULL)) == SENTINEL_ELEVATION_BITS) && ((int) get_3d_terrain_point_data_type (NULL) == 1)
		&& (get_3d_terrain_point_data_normal (NULL) == &p->record->face_normal);
	memcpy (p->record, &snapshot, sizeof (snapshot));
	if (!anchored)
	{
		refuse (text, capacity, "shared_record_not_reachable_through_accessors");
		return -1;
	}

	out.text = text;
	out.capacity = capacity;
	out.used = 0;
	out.overflow = 0;
	p->out = &out;
	t = &p->tally;

	for (i = 0; i < points; i++)
	{
		make_point (p, i);
	}
	for (i = 0; i < impacts; i++)
	{
		make_impact (p, i);
	}

	/* write the snapshot back byte for byte and check it */
	memcpy (p->record, &snapshot, sizeof (snapshot));
	restored = memcmp (p->record, &snapshot, sizeof (snapshot)) == 0;
	record_hash_after = record_words_hash (p->record);

	emit (&out, "ground-probe seed=%u points=%d impacts=%d samples_per_point_tag=%d samples_per_impact_type=%d\n", (unsigned) seed, points, impacts,
		POINT_SAMPLES_PER_TAG, IMPACT_SAMPLES_PER_TYPE);
	emit (&out, "ground-map world_sectors_x=%d world_sectors_z=%d world_sector_side=%d terrain_sectors_x=%d terrain_sectors_z=%d terrain_sector_side=%d\n", p->world_sectors_x,
		p->world_sectors_z, p->world_side, p->sectors_x, p->sectors_z, (int) TERRAIN_3D_SECTOR_SIDE_LENGTH);
	emit (&out, "ground-record anchored=1 restored=%d words_before=%016llx words_after=%016llx\n", restored, (unsigned long long) record_hash_before,
		(unsigned long long) record_hash_after);
	emit (&out, "ground-points cases=%d in_map=%d off_map=%d input_mutated=%d\n", t->points, t->above_cases, t->off_map_cases, t->input_mutated);
	emit (&out, "ground-above cases=%d true=%d false=%d record_touched=%d true_record_untouched=%d warm_same=%d warm_start_intact=%d\n", t->above_cases, t->above_true,
		t->above_cases - t->above_true, t->above_touched, t->above_true_untouched, t->above_warm_same, t->above_warm_intact);
	emit (&out, "ground-below cases=%d true=%d false=%d record_touched=%d true_record_untouched=%d warm_same=%d warm_start_intact=%d off_map_true=%d "
		"off_map_record_touched=%d\n", t->below_cases,
		t->below_true, t->below_cases - t->below_true, t->below_touched, t->below_true_untouched, t->below_warm_same, t->below_warm_intact, t->off_map_below_true,
		t->off_map_below_touched);
	emit (&out, "ground-complement in_map_cases=%d complement=%d not_complement=%d both_true=%d both_false=%d\n", t->in_map_both, t->complement, t->not_complement,
		t->both_true, t->both_false);
	emit (&out, "ground-impact cases=%d a_corrected=%d b_corrected=%d xz_changed=%d alias_equal=%d alias_unequal=%d warm_same=%d warm_different=%d record_touched=%d "
		"record_untouched=%d impact_equals_corrected_below=%d warm_start_intact=%d\n",
		t->impacts, t->a_changed, t->b_changed, t->xz_changed, t->alias_equal, t->alias_unequal, t->warm_same, t->warm_different, t->touched, t->untouched,
		t->impact_is_below_endpoint, t->impact_warm_intact);
	emit (&out, "ground-impact-classes invalid=%d water=%d forest=%d land=%d other=%d\n", t->class_count[TERRAIN_CLASS_INVALID], t->class_count[TERRAIN_CLASS_WATER],
		t->class_count[TERRAIN_CLASS_FOREST], t->class_count[TERRAIN_CLASS_LAND], t->class_other + t->class_count[4] + t->class_count[5] + t->class_count[6] + t->class_count[7]);
	emit (&out, "ground-impact-types t0=%d t1=%d t2=%d t3=%d t4=%d t5=%d t6=%d t7=%d\n", t->by_type[0], t->by_type[1], t->by_type[2], t->by_type[3], t->by_type[4],
		t->by_type[5], t->by_type[6], t->by_type[7]);
	emit (&out, "ground-asserts original_below_endpoint_still_above=%d extra_above_endpoint_not_above=%d\n", t->assert_below_violations, t->post_above_not_above);
	emit (&out, "ground-hash combined=%016llx above=%016llx below=%016llx impact=%016llx\n", (unsigned long long) p->hashes.combined,
		(unsigned long long) p->hashes.above, (unsigned long long) p->hashes.below, (unsigned long long) p->hashes.impact);
	emit (&out, "ground-physical combined=%016llx above=%016llx below=%016llx impact=%016llx\n", (unsigned long long) p->hashes.phys_combined,
		(unsigned long long) p->hashes.phys_above, (unsigned long long) p->hashes.phys_below, (unsigned long long) p->hashes.phys_impact);

	if (out.overflow)
	{
		text[0] = '\0';
		refuse (text, capacity, "report_does_not_fit_the_capacity");
		return -1;
	}
	return restored ? 0 : -1;
}

#endif
