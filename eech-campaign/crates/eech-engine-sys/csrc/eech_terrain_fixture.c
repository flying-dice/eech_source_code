/*
 * TEST-ONLY terrain surface characterisation (EEDCS-47). Compiled only with the `terrain-fixture` feature; never part of the shipping
 * module.
 *
 * The campaign reads the terrain through get_3d_terrain_point_data (and its wrappers). The query returns an elevation and, as a side
 * effect, fills a caller-supplied "find-fast" record (or, when the caller passes NULL, one shared static record) with the last elevation,
 * the face normal, the surface type and a face hint that the next query on the same record tries first. terrelev.c, built with
 * EECH_TERRAIN_FIXTURE, calls eech_terrain_record at each return. This file keeps two things:
 *
 *   1. The run-wide trace. Every query the campaign's own update loop makes since boot or the last reset: how many, how many used the
 *      shared record or a private one, how many tried and hit the face hint, how many scanned the sector, how many failed to find a face,
 *      how many were clamped to the map or fell in the Cuba flaw area, a histogram of the surface type read, a per-call-site table
 *      (file and line, the query macros pass them) and an order-dependent FNV-1a hash over the inputs' bit patterns and the answers.
 *      It only observes: nothing it reads is altered.
 *
 *   2. The probe sweep. A deterministic set of points over the loaded map (four corners inset by 1 cm, the centre, then points from a
 *      private xorshift generator, one in eight snapped to a multiple or half-multiple of the sector side length, +-1 mm). For each
 *      point the answer of a query on a fresh zeroed record (no hint) is the reference, and the same point is asked again:
 *        - on the same record straight after (an exact-hint hit),
 *        - on a private record and on the shared record that were each first primed at a point 5 cm to 500 m away (the hint a previous
 *          query would have left),
 *        - through the wrappers the campaign calls (the elevation macro, elevation-and-normal, face normal), which use the shared record.
 *      A difference between a hinted answer and the fresh answer is order dependence in the original solver, and is counted and reported,
 *      never corrected. Surface type, class and ground-vehicle suitability are read for every type met.
 *      The sweep's points never touch the game's random stream, and the shared record is saved and restored byte for byte, so the sweep
 *      leaves EECH's own hint state alone. Its trace is separate from the run-wide one.
 *
 * The same script and the same hashes are used before and after any extraction, so a difference is a difference in answers. Points are
 * kept at least 1 cm inside the map: the off-map path asserts in the original and is not probed.
 */

#ifdef EECH_TERRAIN_FIXTURE

#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "project.h"

#include "eech_engine.h"
#include "eech_headless.h"

#define SITE_SLOTS 2048
#define FNV_OFFSET 14695981039346656037ull
#define FNV_PRIME 1099511628211ull
#define PRIMER_OFFSETS 5
#define MAX_EMIT 256

struct site
{
	const char *file;
	int line;
	uint64_t calls, hint_null, fast_hit, not_found;
};

struct terrain_trace
{
	uint64_t hash, calls, hint_null, hint_private, fast_attempted, fast_hit, slow_scans, not_found, cuba, clamped;
	uint64_t by_type[TERRAIN_TYPE_LAST];
	uint64_t site_overflow;
	struct site sites[SITE_SLOTS];
};

static struct terrain_trace run_trace;
static struct terrain_trace probe_trace;
static struct terrain_trace *active_trace = &run_trace;

static void fold (struct terrain_trace *trace, uint32_t word)
{
	int byte;
	for (byte = 0; byte < 4; byte++)
	{
		trace->hash ^= (uint64_t) ((word >> (8 * byte)) & 0xffu);
		trace->hash *= FNV_PRIME;
	}
}

static uint32_t float_bits (float value)
{
	uint32_t bits;
	memcpy (&bits, &value, sizeof (bits));
	return bits;
}

static void trace_reset (struct terrain_trace *trace)
{
	memset (trace, 0, sizeof (*trace));
	trace->hash = FNV_OFFSET;
}

static struct terrain_trace *ready (void)
{
	if (active_trace->calls == 0 && active_trace->hash == 0)
	{
		trace_reset (active_trace);
	}
	return active_trace;
}

static struct site *find_site (struct terrain_trace *trace, const char *file, int line)
{
	unsigned slot = (unsigned) ((((uintptr_t) file >> 3) * 2654435761u) ^ ((unsigned) line * 40503u)) & (SITE_SLOTS - 1);
	int probes;
	for (probes = 0; probes < SITE_SLOTS; probes++)
	{
		struct site *site = &trace->sites[slot];
		if (site->file == NULL)
		{
			site->file = file;
			site->line = line;
			return site;
		}
		if (site->file == file && site->line == line)
		{
			return site;
		}
		slot = (slot + 1) & (SITE_SLOTS - 1);
	}
	return NULL;
}

/*
 * Called by the query at each return. `hint_null` is whether the caller passed no record (so the shared one was used);
 * `fast_attempted` that the record held a face hint in the same sector; `fast_hit` that the hint's triangle held the point.
 */
void eech_terrain_record (const char *file, int line, float x, float z, int hint_null, int fast_attempted, int fast_hit, int found, int clamped, int cuba, float result,
	const terrain_3d_point_data *point_data)
{
	struct terrain_trace *trace = ready ();
	struct site *site = find_site (trace, file, line);
	int type = (int) point_data->terrain_type;

	trace->calls++;
	if (hint_null)
	{
		trace->hint_null++;
	}
	else
	{
		trace->hint_private++;
	}
	if (fast_attempted)
	{
		trace->fast_attempted++;
	}
	if (fast_hit)
	{
		trace->fast_hit++;
	}
	else if (!cuba && found)
	{
		trace->slow_scans++;
	}
	if (!found)
	{
		trace->not_found++;
	}
	if (cuba)
	{
		trace->cuba++;
	}
	if (clamped)
	{
		trace->clamped++;
	}
	if (type >= 0 && type < TERRAIN_TYPE_LAST)
	{
		trace->by_type[type]++;
	}
	if (site)
	{
		site->calls++;
		site->hint_null += hint_null ? 1 : 0;
		site->fast_hit += fast_hit ? 1 : 0;
		site->not_found += found ? 0 : 1;
	}
	else
	{
		trace->site_overflow++;
	}

	fold (trace, float_bits (x));
	fold (trace, float_bits (z));
	fold (trace, (uint32_t) ((hint_null ? 1u : 0u) | (fast_attempted ? 2u : 0u) | (fast_hit ? 4u : 0u) | (found ? 8u : 0u) | (clamped ? 16u : 0u) | (cuba ? 32u : 0u)));
	fold (trace, float_bits (result));
	fold (trace, float_bits (point_data->last_elevation));
	fold (trace, float_bits (point_data->face_normal.x));
	fold (trace, float_bits (point_data->face_normal.y));
	fold (trace, float_bits (point_data->face_normal.z));
	fold (trace, (uint32_t) type);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* report text: key=value lines, never cut short */

struct text
{
	char *buffer;
	int capacity;
	int used;
	int failed;
};

static void put (struct text *out, const char *format, ...) __attribute__ ((format (printf, 2, 3)));

static void put (struct text *out, const char *format, ...)
{
	va_list args;
	int n;
	if (out->failed)
	{
		return;
	}
	va_start (args, format);
	n = vsnprintf (out->buffer + out->used, (size_t) (out->capacity - out->used), format, args);
	va_end (args);
	if (n < 0 || n >= out->capacity - out->used)
	{
		out->failed = 1;
		return;
	}
	out->used += n;
}

/* the part of a source path that identifies the file: from "aphavoc" or "modules" (or "csrc"), '\' shown as '/' */
static void short_path (const char *file, char *out, size_t size)
{
	const char *start = file;
	const char *roots[] = { "aphavoc", "modules", "csrc" };
	size_t i, n;
	for (i = 0; i < sizeof (roots) / sizeof (roots[0]); i++)
	{
		const char *hit = strstr (file, roots[i]);
		if (hit && (hit == file || hit[-1] == '/' || hit[-1] == '\\'))
		{
			start = hit;
			break;
		}
	}
	snprintf (out, size, "%s", start);
	for (n = 0; out[n]; n++)
	{
		if (out[n] == '\\')
		{
			out[n] = '/';
		}
	}
}

struct merged
{
	char path[160];
	int line;
	uint64_t calls, hint_null, fast_hit, not_found;
};

static int merged_before (const struct merged *a, const struct merged *b)
{
	int c = strcmp (a->path, b->path);
	return c < 0 || (c == 0 && a->line < b->line);
}

static void report_trace (struct text *out, const char *scope, const struct terrain_trace *trace)
{
	static struct merged rows[SITE_SLOTS];
	int count = 0, i, j;

	put (out, "terrain-hash scope=%s run=%016llx\n", scope, (unsigned long long) trace->hash);
	put (out, "terrain-calls scope=%s calls=%llu hint_null=%llu hint_private=%llu fast_attempted=%llu fast_hit=%llu slow_scans=%llu not_found=%llu cuba=%llu clamped=%llu\n",
		scope, (unsigned long long) trace->calls, (unsigned long long) trace->hint_null, (unsigned long long) trace->hint_private,
		(unsigned long long) trace->fast_attempted, (unsigned long long) trace->fast_hit, (unsigned long long) trace->slow_scans,
		(unsigned long long) trace->not_found, (unsigned long long) trace->cuba, (unsigned long long) trace->clamped);
	for (i = 0; i < TERRAIN_TYPE_LAST; i++)
	{
		if (trace->by_type[i])
		{
			put (out, "terrain-type scope=%s id=%d name=%s count=%llu\n", scope, i, get_terrain_type_name ((terrain_types) i), (unsigned long long) trace->by_type[i]);
		}
	}
	/* one row per (file, line): sites whose __FILE__ literals differ in address but not in text are merged */
	for (i = 0; i < SITE_SLOTS; i++)
	{
		const struct site *site = &trace->sites[i];
		struct merged row;
		if (site->file == NULL)
		{
			continue;
		}
		short_path (site->file, row.path, sizeof (row.path));
		row.line = site->line;
		row.calls = site->calls;
		row.hint_null = site->hint_null;
		row.fast_hit = site->fast_hit;
		row.not_found = site->not_found;
		for (j = 0; j < count; j++)
		{
			if (rows[j].line == row.line && strcmp (rows[j].path, row.path) == 0)
			{
				rows[j].calls += row.calls;
				rows[j].hint_null += row.hint_null;
				rows[j].fast_hit += row.fast_hit;
				rows[j].not_found += row.not_found;
				break;
			}
		}
		if (j == count)
		{
			rows[count++] = row;
		}
	}
	for (i = 1; i < count; i++)
	{
		struct merged held = rows[i];
		for (j = i - 1; j >= 0 && merged_before (&held, &rows[j]); j--)
		{
			rows[j + 1] = rows[j];
		}
		rows[j + 1] = held;
	}
	for (i = 0; i < count; i++)
	{
		put (out, "terrain-site scope=%s file=%s line=%d calls=%llu hint_null=%llu fast_hit=%llu not_found=%llu\n", scope, rows[i].path, rows[i].line,
			(unsigned long long) rows[i].calls, (unsigned long long) rows[i].hint_null, (unsigned long long) rows[i].fast_hit, (unsigned long long) rows[i].not_found);
	}
	put (out, "terrain-sites scope=%s distinct=%d overflow=%llu\n", scope, count, (unsigned long long) trace->site_overflow);
}

/* the run-wide report. Returns 0, or -1 when it does not fit */
int eech_terrain_trace_get (char *text, int capacity)
{
	struct text out = { text, capacity, 0, 0 };
	if (capacity <= 0)
	{
		return -1;
	}
	text[0] = '\0';
	ready ();
	report_trace (&out, "run", &run_trace);
	return out.failed ? -1 : 0;
}

void eech_terrain_trace_clear (void)
{
	trace_reset (&run_trace);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the probe sweep */

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

struct answer
{
	float result, last, nx, ny, nz;
	int type;
	int fast_hit;
};

static int same_answer (const struct answer *a, const struct answer *b)
{
	return float_bits (a->result) == float_bits (b->result) && float_bits (a->last) == float_bits (b->last) && float_bits (a->nx) == float_bits (b->nx)
		&& float_bits (a->ny) == float_bits (b->ny) && float_bits (a->nz) == float_bits (b->nz) && a->type == b->type;
}

/* one query through `record` (NULL = the shared record) and what it left in the record */
static void ask (float x, float z, terrain_3d_point_data *record, struct answer *answer)
{
	uint64_t before = active_trace->fast_hit;
	vec3d normal;
	answer->result = get_3d_terrain_point_data (x, z, record);
	answer->last = get_3d_terrain_point_data_elevation (record);
	normal = *get_3d_terrain_point_data_normal (record);
	answer->nx = normal.x;
	answer->ny = normal.y;
	answer->nz = normal.z;
	answer->type = (int) get_3d_terrain_point_data_type (record);
	answer->fast_hit = active_trace->fast_hit != before;
}

static void fold_answer (uint64_t *hash, const struct answer *a)
{
	uint32_t words[7];
	int w, byte;
	words[0] = float_bits (a->result);
	words[1] = float_bits (a->last);
	words[2] = float_bits (a->nx);
	words[3] = float_bits (a->ny);
	words[4] = float_bits (a->nz);
	words[5] = (uint32_t) a->type;
	words[6] = (uint32_t) a->fast_hit;
	for (w = 0; w < 7; w++)
	{
		for (byte = 0; byte < 4; byte++)
		{
			*hash ^= (uint64_t) ((words[w] >> (8 * byte)) & 0xffu);
			*hash *= FNV_PRIME;
		}
	}
}

static float inside (float value, float low, float high)
{
	return value < low ? low : (value > high ? high : value);
}

/*
 * Runs the sweep. Returns 0 and writes the report, -1 when the report does not fit, -2 when the map is not loaded. `count` random points
 * follow the six fixed ones; the first `emit` samples (at most 256) are written in full.
 */
int eech_terrain_probe_run (uint32_t seed, int count, int emit, char *text, int capacity)
{
	static const float primer_offsets[PRIMER_OFFSETS] = { 0.05f, 0.5f, 5.0f, 50.0f, 500.0f };
	struct text out = { text, capacity, 0, 0 };
	struct terrain_trace *saved = active_trace;
	terrain_3d_point_data *shared = eech_terrain_system_point_data ();
	terrain_3d_point_data shared_saved = *shared;
	uint32_t state = seed ? seed : 1u;
	uint64_t sample_hash = FNV_OFFSET, primed_hash = FNV_OFFSET;
	uint64_t repeat_mismatch = 0, shared_prime_mismatch = 0, private_prime_mismatch = 0, shared_prime_fast = 0, private_prime_fast = 0, macro_mismatch = 0, normal_wrapper_mismatch = 0,
		face_normal_mismatch = 0, result_ne_last = 0, snapped = 0, primer_skipped = 0;
	uint64_t type_seen[TERRAIN_TYPE_LAST], class_seen[TERRAIN_CLASS_LAST];
	const float margin = 0.01f;
	const float min_x = terrain_3d_min_map_x + margin, max_x = terrain_3d_max_map_x - margin, min_z = terrain_3d_min_map_z + margin, max_z = terrain_3d_max_map_z - margin;
	const float side = (float) TERRAIN_3D_SECTOR_SIDE_LENGTH;
	int total = 6 + count, i, rc;

	if (capacity <= 0)
	{
		return -1;
	}
	text[0] = '\0';
	if (!(terrain_3d_max_map_x > terrain_3d_min_map_x) || !(terrain_3d_max_map_z > terrain_3d_min_map_z))
	{
		return -2;
	}
	if (emit > MAX_EMIT)
	{
		emit = MAX_EMIT;
	}
	memset (type_seen, 0, sizeof (type_seen));
	memset (class_seen, 0, sizeof (class_seen));
	trace_reset (&probe_trace);
	active_trace = &probe_trace;

	for (i = 0; i < total; i++)
	{
		float x, z, primer_x, primer_z, distance;
		struct answer fresh, repeat, shared_primed, private_primed, scratch;
		terrain_3d_point_data record, primer_record;
		vec3d face;
		float height;
		int sign_x, sign_z;

		if (i < 4)
		{
			x = (i & 1) ? max_x : min_x;
			z = (i & 2) ? max_z : min_z;
		}
		else if (i == 4)
		{
			x = 0.5f * (min_x + max_x);
			z = 0.5f * (min_z + max_z);
		}
		else if (i == 5)
		{
			/* the corner of a sector that is not on the map edge, 1 mm either side of the line */
			x = side * 4.0f + 0.001f;
			z = side * 4.0f - 0.001f;
		}
		else
		{
			x = min_x + unit (&state) * (max_x - min_x);
			z = min_z + unit (&state) * (max_z - min_z);
			if ((next_random (&state) & 7u) == 0)
			{
				float base_x = (float) ((int) (x / side)) * side + ((next_random (&state) & 1u) ? 0.5f * side : 0.0f);
				float base_z = (float) ((int) (z / side)) * side + ((next_random (&state) & 1u) ? 0.5f * side : 0.0f);
				x = base_x + ((int) (next_random (&state) % 3u) - 1) * 0.001f;
				z = base_z + ((int) (next_random (&state) % 3u) - 1) * 0.001f;
				snapped++;
			}
		}
		x = inside (x, min_x, max_x);
		z = inside (z, min_z, max_z);

		distance = primer_offsets[next_random (&state) % PRIMER_OFFSETS];
		sign_x = (next_random (&state) & 1u) ? 1 : -1;
		sign_z = (next_random (&state) & 1u) ? 1 : -1;
		primer_x = x + sign_x * distance;
		primer_z = z + sign_z * distance * 0.5f;
		if (primer_x < min_x || primer_x > max_x || primer_z < min_z || primer_z > max_z)
		{
			/* a primer off the map is not asked; the primed queries then use the target as its own primer */
			primer_skipped++;
			primer_x = x;
			primer_z = z;
		}

		/* reference: a fresh zeroed record holds no hint */
		memset (&record, 0, sizeof (record));
		ask (x, z, &record, &fresh);
		fold_answer (&sample_hash, &fresh);
		if (float_bits (fresh.result) != float_bits (fresh.last))
		{
			result_ne_last++;
		}
		if (fresh.type >= 0 && fresh.type < TERRAIN_TYPE_LAST)
		{
			type_seen[fresh.type]++;
			class_seen[(int) get_terrain_type_class ((terrain_types) fresh.type)]++;
		}

		/* the same record straight after: an exact-hint hit */
		ask (x, z, &record, &repeat);
		if (!same_answer (&fresh, &repeat))
		{
			repeat_mismatch++;
		}

		/* a private record primed elsewhere */
		memset (&primer_record, 0, sizeof (primer_record));
		ask (primer_x, primer_z, &primer_record, &scratch);
		ask (x, z, &primer_record, &private_primed);
		private_prime_fast += private_primed.fast_hit;
		if (!same_answer (&fresh, &private_primed))
		{
			private_prime_mismatch++;
		}
		fold_answer (&primed_hash, &private_primed);

		/* the shared record primed elsewhere, as the campaign's callers that pass NULL leave it */
		ask (primer_x, primer_z, NULL, &scratch);
		ask (x, z, NULL, &shared_primed);
		shared_prime_fast += shared_primed.fast_hit;
		if (!same_answer (&fresh, &shared_primed))
		{
			shared_prime_mismatch++;
		}
		fold_answer (&primed_hash, &shared_primed);

		/* the wrappers the campaign calls, all on the shared record */
		if (float_bits (get_3d_terrain_elevation (x, z)) != float_bits (fresh.result))
		{
			macro_mismatch++;
		}
		height = get_3d_terrain_elevation_and_face_normal (x, z, &face, NULL);
		if (float_bits (height) != float_bits (fresh.result) || float_bits (face.x) != float_bits (fresh.nx) || float_bits (face.y) != float_bits (fresh.ny)
			|| float_bits (face.z) != float_bits (fresh.nz))
		{
			normal_wrapper_mismatch++;
		}
		get_3d_terrain_face_normal (&face, x, z);
		if (float_bits (face.x) != float_bits (fresh.nx) || float_bits (face.y) != float_bits (fresh.ny) || float_bits (face.z) != float_bits (fresh.nz))
		{
			face_normal_mismatch++;
		}

		if (i < emit)
		{
			int type = fresh.type;
			int valid = type >= 0 && type < TERRAIN_TYPE_LAST;
			put (&out, "terrain-sample i=%d x=%08x z=%08x result=%08x last=%08x nx=%08x ny=%08x nz=%08x type=%d class=%d vehicle=%d\n", i, float_bits (x), float_bits (z),
				float_bits (fresh.result), float_bits (fresh.last), float_bits (fresh.nx), float_bits (fresh.ny), float_bits (fresh.nz), type,
				valid ? (int) get_terrain_type_class ((terrain_types) type) : -1, valid ? get_terrain_type_suitable_for_ground_vehicle ((terrain_types) type) : -1);
		}
	}

	active_trace = saved;
	*shared = shared_saved;

	put (&out, "terrain-probe seed=%u count=%d fixed=6 snapped=%llu primer_skipped=%llu emitted=%d\n", (unsigned) seed, count, (unsigned long long) snapped,
		(unsigned long long) primer_skipped, emit < total ? emit : total);
	put (&out, "terrain-probe-hash sample=%016llx primed=%016llx\n", (unsigned long long) sample_hash, (unsigned long long) primed_hash);
	put (&out, "terrain-probe-diff repeat=%llu private_primed=%llu shared_primed=%llu private_primed_fast=%llu shared_primed_fast=%llu elevation_macro=%llu elevation_and_normal=%llu face_normal=%llu result_ne_last=%llu\n",
		(unsigned long long) repeat_mismatch, (unsigned long long) private_prime_mismatch, (unsigned long long) shared_prime_mismatch, (unsigned long long) private_prime_fast,
		(unsigned long long) shared_prime_fast, (unsigned long long) macro_mismatch, (unsigned long long) normal_wrapper_mismatch, (unsigned long long) face_normal_mismatch,
		(unsigned long long) result_ne_last);
	for (i = 0; i < TERRAIN_TYPE_LAST; i++)
	{
		if (type_seen[i])
		{
			put (&out, "terrain-probe-type id=%d name=%s count=%llu class=%d vehicle=%d\n", i, get_terrain_type_name ((terrain_types) i), (unsigned long long) type_seen[i],
				(int) get_terrain_type_class ((terrain_types) i), get_terrain_type_suitable_for_ground_vehicle ((terrain_types) i));
		}
	}
	for (i = 0; i < TERRAIN_CLASS_LAST; i++)
	{
		if (class_seen[i])
		{
			put (&out, "terrain-probe-class id=%d count=%llu\n", i, (unsigned long long) class_seen[i]);
		}
	}
	put (&out, "terrain-probe-state shared_record_restored=%d\n", memcmp (shared, &shared_saved, sizeof (*shared)) == 0);
	report_trace (&out, "probe", &probe_trace);
	return out.failed ? -1 : 0;
}

#endif
