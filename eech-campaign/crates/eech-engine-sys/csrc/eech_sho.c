/*
 * TEST-ONLY (EEDCS-49, D3, sector-height public baseline; feature `sector-height-observer`): the READ-ONLY observer of the sector tallest-structure high-water value on the
 * real public path (Lua host -> public native module -> the real en_list funnel, the real sector response, the host roster statements, the hosted scan and candidate questions,
 * the fixed-wing clearance and the session pack). The plan is docs/d3-sector-height-public-baseline-plan.md. This file holds the recorder core (plain data, no entity access, driven
 * directly by the controls), the hook definitions (eech_sho_hooks.h: they read the original's own getters, then call the core) and the report formatting; the engine entries that
 * expose the report are in eech_engine.c.
 *
 * Observation only. Nothing here changes a word the campaign holds, the order of a statement, a clock or the random stream; floats arrive as pointers and leave as 32-bit
 * patterns (no floating-point arithmetic, so the engine's rounding mode and any signalling NaN are untouched). The tables are bounded and private; a table that fills latches an
 * overflow counter and the report says `complete=0`. Single-threaded bookkeeping: the engine runs behind one entry lock. ONE boot per process: the tables are never reset.
 *
 * JOINING. INSERT, LINK_CHILD, the host statement and the removal of one placement are joined by the observer's own record identity: an INSERT opens a record under the entity's heap
 * index (a private bookkeeping number; no production semantic identifier is allocated or changed), every later event of that entity finds the open record, and the removal closes it.
 * A LINK or a sector-placed statement with no open record, a second INSERT of an open record, a statement of another type and a removal with no open record are COUNTED as join failures
 * and make the report incomplete: nothing is guessed from a family or a sector. A statement of a city building (its place is a city's list, never a sector's) is counted apart.
 *
 * SHAPE. The shape is read only where the original block reads it (a fixed-flagged sender). Where that gate is false the shape is UNOBSERVED, never invalid.
 *
 * FACT 12. Every candidate question's raw word is related to the raw word of the LAST FIRST read of the scan (the sector the host named), by bit equality, per sector: equal and
 * differing offers are counted apart, and an offer with no FIRST before it is a join failure.
 *
 * SESSION. Every pack and unpack word is stored (the event table is the only bound); a word that did not fit is counted as omitted and makes the report incomplete.
 *
 * The report is `key=value` text. A recorded word is a hex bit pattern; a sequence number orders every record across all kinds.
 */
#ifdef EECH_SECTOR_HEIGHT_OBSERVER

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "project.h"

#include "entity/mobile/mb_hostmodel.h"

#include "eech_sho_hooks.h"

/* the statement word counts the observer stores must hold the host's contract widths */
typedef char sho_fixed_words_fit [(CAMPAIGN_FIXED_STATE_WORDS <= 10) ? 1 : -1];
typedef char sho_city_words_fit [(CAMPAIGN_CITY_STATE_WORDS <= 10) ? 1 : -1];

#define SHO_EVENTS 262144
#define SHO_CLEARANCES 4096
#define SHO_SCAN_SIDE 256
#define SHO_ENTITIES 524288

typedef char sho_entities_fit [(MAX_NUM_ENTITIES <= SHO_ENTITIES) ? 1 : -1];

enum sho_kind { SHO_INSERT = 1, SHO_LINK, SHO_STMT_FIXED, SHO_STMT_CITY, SHO_PACK, SHO_UNPACK, SHO_REMOVE, SHO_RELEASE };

/* the family of a placement: a fixed family (every fixed-flagged type but the two below), the city, a city building (placed in a city's list, never a sector's) */
enum sho_family { SHO_FAMILY_FIXED = 0, SHO_FAMILY_CITY = 1, SHO_FAMILY_CITY_BUILDING = 2 };

/* the shape state of a link: the original block reads the shape only for a fixed-flagged sender */
enum sho_shape_state { SHO_SHAPE_UNOBSERVED = 0, SHO_SHAPE_VALID = 1, SHO_SHAPE_INVALID = 2 };

struct sho_event
{
	uint64_t seq;
	int kind;
	uint32_t rec;
	int ent;
	int sx, sz;
	int etype, family, shape_state, shape;
	uint32_t pos [3];
	uint32_t ymax;
	uint32_t before, after;
	uint32_t words [10];
	int word_count;
	char key [24];
};

struct sho_clearance
{
	uint64_t seq;
	int sx, sz;
	uint32_t tallest, terrain, after_tallest, final_word;
	int final_seen;
};

struct sho_scan_cell
{
	int touched;
	int sx, sz;
	uint64_t reads, first_seq, last_seq;
	uint32_t first_word, last_word;
	uint64_t changes;
	uint64_t fact_offers, fact_equal, fact_differ, fact_changes;
	uint32_t fact_first_word, fact_last_word;
};

static struct sho_event events [SHO_EVENTS];
static struct sho_clearance clearances [SHO_CLEARANCES];
static struct sho_scan_cell scan_cells [SHO_SCAN_SIDE * SHO_SCAN_SIDE];
static int scan_order [SHO_SCAN_SIDE * SHO_SCAN_SIDE];
static int scan_touched;

static uint32_t current_rec [SHO_ENTITIES];
static int16_t current_type [SHO_ENTITIES];
static uint32_t record_count;

static uint64_t seq;
static int event_count, clearance_stored;
static uint64_t event_overflow, clearance_overflow, scan_out_of_range, mismatches, index_out_of_range;
static uint64_t inserts_total, inserts_fixed, inserts_city, inserts_mobile, double_insert;
static uint64_t removes_total, removes_recorded, removes_mobile, remove_unjoined, released_open;
static uint64_t link_total, link_fixed, link_city, link_mobile, links_joined, links_unjoined, raised;
static uint64_t shape_count [2][3];
static uint64_t statements_fixed, statements_city, stmt_joined, stmt_unjoined, stmt_type_mismatch, stmt_child;
static uint64_t pack_total, pack_stored, unpack_total, unpack_stored;
static uint64_t scan_reads, scan_firsts, fact_offers, fact_nonzero, fact_equal, fact_differ, fact_without_first;
static uint64_t leader_reached, follow_reached, clearance_total;
static int pending_active;
static uint32_t pending_before;
static int pending_clearance = -1;
static int has_first, first_x, first_z;
static uint32_t first_word;

static struct sho_event *begin_event (int kind)
{
	struct sho_event *e;

	seq++;

	if (event_count >= SHO_EVENTS)
	{
		event_overflow++;

		return NULL;
	}

	e = &events [event_count++];

	memset (e, 0, sizeof (*e));

	e->seq = seq;

	e->kind = kind;

	return e;
}

static void copy_key (char *out, const char *key)
{
	int i;

	for (i = 0; key && key [i] && i < 23; i++)
	{
		out [i] = ((key [i] >= '0' && key [i] <= '9') || (key [i] >= 'a' && key [i] <= 'z') || (key [i] >= 'A' && key [i] <= 'Z')) ? key [i] : '?';
	}

	out [i] = '\0';
}

static int in_range (int ent)
{
	return ent >= 0 && ent < SHO_ENTITIES;
}

/* the open record of an entity, or 0; an index outside the table is a join failure of its own */
static uint32_t open_record (int ent)
{
	if (!in_range (ent))
	{
		index_out_of_range++;

		mismatches++;

		return 0;
	}

	return current_rec [ent];
}

static struct sho_scan_cell *cell_of (int x, int z)
{
	if (x < 0 || z < 0 || x >= SHO_SCAN_SIDE || z >= SHO_SCAN_SIDE)
	{
		return NULL;
	}

	return &scan_cells [x + z * SHO_SCAN_SIDE];
}

/* ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- */
/* the recorder core: plain data only, so that the controls can drive it directly */

void eech_sho_core_insert (int ent, int etype, int family, int sx, int sz)
{
	struct sho_event *ev;
	uint32_t rec = 0;

	inserts_total++;

	if (family == SHO_FAMILY_CITY)
	{
		inserts_city++;
	}
	else
	{
		inserts_fixed++;
	}

	if (!in_range (ent))
	{
		index_out_of_range++;

		mismatches++;
	}
	else
	{
		if (current_rec [ent] != 0)
		{
			/* a second INSERT of a placement that was never removed or released: the join is no longer unique */
			double_insert++;

			mismatches++;
		}

		rec = ++record_count;

		current_rec [ent] = rec;

		current_type [ent] = (int16_t) etype;
	}

	ev = begin_event (SHO_INSERT);

	if (ev)
	{
		ev->ent = ent;

		ev->rec = rec;

		ev->etype = etype;

		ev->family = family;

		ev->sx = sx;

		ev->sz = sz;
	}
}

void eech_sho_core_insert_mobile (void)
{
	inserts_total++;

	inserts_mobile++;
}

void eech_sho_core_remove (int ent, int etype, int family, int fixed_or_city)
{
	struct sho_event *ev;
	uint32_t rec;

	removes_total++;

	if (!fixed_or_city)
	{
		removes_mobile++;

		return;
	}

	rec = open_record (ent);

	if (rec == 0)
	{
		remove_unjoined++;

		mismatches++;
	}
	else
	{
		current_rec [ent] = 0;
	}

	removes_recorded++;

	ev = begin_event (SHO_REMOVE);

	if (ev)
	{
		ev->ent = ent;

		ev->rec = rec;

		ev->etype = etype;

		ev->family = family;
	}
}

/* an entity's slot is freed: an open record is closed (counted, not a join failure: the removal path is the original's own), so that a reused index never joins an old record */
void eech_sho_core_release (int ent)
{
	struct sho_event *ev;

	if (!in_range (ent) || current_rec [ent] == 0)
	{
		return;
	}

	released_open++;

	ev = begin_event (SHO_RELEASE);

	if (ev)
	{
		ev->ent = ent;

		ev->rec = current_rec [ent];

		ev->etype = current_type [ent];
	}

	current_rec [ent] = 0;
}

void eech_sho_core_link_begin (uint32_t before)
{
	if (pending_active)
	{
		mismatches++;
	}

	pending_active = 1;

	pending_before = before;
}

void eech_sho_core_link_end (int ent, int etype, int family, int fixed_or_city, int shape_state, int shape, const uint32_t *pos, uint32_t ymax, uint32_t after, int sx, int sz)
{
	struct sho_event *ev;
	uint32_t rec;

	if (!pending_active)
	{
		mismatches++;
	}

	pending_active = 0;

	link_total++;

	if (!fixed_or_city)
	{
		link_mobile++;

		return;
	}

	if (family == SHO_FAMILY_CITY)
	{
		link_city++;
	}
	else
	{
		link_fixed++;
	}

	shape_count [family == SHO_FAMILY_CITY ? 1 : 0] [shape_state]++;

	if (after != pending_before)
	{
		raised++;
	}

	rec = open_record (ent);

	if (rec == 0)
	{
		links_unjoined++;

		mismatches++;
	}
	else if (current_type [ent] != etype)
	{
		links_unjoined++;

		mismatches++;
	}
	else
	{
		links_joined++;
	}

	ev = begin_event (SHO_LINK);

	if (ev)
	{
		ev->ent = ent;

		ev->rec = rec;

		ev->etype = etype;

		ev->family = family;

		ev->shape_state = shape_state;

		ev->shape = shape;

		ev->before = pending_before;

		ev->after = after;

		ev->sx = sx;

		ev->sz = sz;

		if (pos)
		{
			memcpy (ev->pos, pos, sizeof (ev->pos));
		}

		ev->ymax = ymax;
	}
}

void eech_sho_core_statement (int ent, int etype, int family, const char *key, const uint32_t *words, int count)
{
	struct sho_event *ev;
	uint32_t rec = 0;
	int i;

	if (family == SHO_FAMILY_CITY)
	{
		statements_city++;
	}
	else
	{
		statements_fixed++;
	}

	if (family == SHO_FAMILY_CITY_BUILDING)
	{
		/* a building is placed in its city's list: no sector record exists or is expected */
		stmt_child++;
	}
	else
	{
		rec = open_record (ent);

		if (rec == 0)
		{
			stmt_unjoined++;

			mismatches++;
		}
		else if (current_type [ent] != etype)
		{
			stmt_type_mismatch++;

			mismatches++;
		}
		else
		{
			stmt_joined++;
		}
	}

	ev = begin_event (family == SHO_FAMILY_CITY ? SHO_STMT_CITY : SHO_STMT_FIXED);

	if (ev)
	{
		ev->ent = ent;

		ev->rec = rec;

		ev->etype = etype;

		ev->family = family;

		copy_key (ev->key, key);

		ev->word_count = count > 10 ? 10 : count;

		for (i = 0; i < ev->word_count; i++)
		{
			ev->words [i] = words [i];
		}
	}
}

void eech_sho_core_scan_first (int x_sector, int z_sector, uint32_t word)
{
	struct sho_scan_cell *c = cell_of (x_sector, z_sector);

	seq++;

	scan_reads++;

	scan_firsts++;

	has_first = 1;

	first_x = x_sector;

	first_z = z_sector;

	first_word = word;

	if (!c)
	{
		scan_out_of_range++;

		mismatches++;

		return;
	}

	if (!c->touched)
	{
		c->touched = 1;

		c->sx = x_sector;

		c->sz = z_sector;

		c->first_seq = seq;

		c->first_word = word;

		scan_order [scan_touched++] = x_sector + z_sector * SHO_SCAN_SIDE;
	}
	else if (word != c->last_word)
	{
		c->changes++;
	}

	c->reads++;

	c->last_seq = seq;

	c->last_word = word;
}

void eech_sho_core_candidate_fact (uint32_t word)
{
	struct sho_scan_cell *c;

	seq++;

	fact_offers++;

	if (word != 0)
	{
		fact_nonzero++;
	}

	if (!has_first)
	{
		fact_without_first++;

		mismatches++;

		return;
	}

	c = cell_of (first_x, first_z);

	if (word == first_word)
	{
		fact_equal++;
	}
	else
	{
		fact_differ++;
	}

	if (c)
	{
		if (c->fact_offers == 0)
		{
			c->fact_first_word = word;
		}
		else if (word != c->fact_last_word)
		{
			c->fact_changes++;
		}

		c->fact_offers++;

		c->fact_last_word = word;

		if (word == first_word)
		{
			c->fact_equal++;
		}
		else
		{
			c->fact_differ++;
		}
	}
}

void eech_sho_core_clearance_leader (void)
{
	seq++;

	leader_reached++;
}

void eech_sho_core_clearance_follow (void)
{
	seq++;

	follow_reached++;
}

void eech_sho_core_clearance_tallest (int sx, int sz, uint32_t tallest, uint32_t terrain, uint32_t after_tallest)
{
	seq++;

	clearance_total++;

	if (pending_clearance >= 0)
	{
		mismatches++;
	}

	pending_clearance = -1;

	if (clearance_stored >= SHO_CLEARANCES)
	{
		clearance_overflow++;

		return;
	}

	pending_clearance = clearance_stored++;

	memset (&clearances [pending_clearance], 0, sizeof (clearances [0]));

	clearances [pending_clearance].seq = seq;

	clearances [pending_clearance].sx = sx;

	clearances [pending_clearance].sz = sz;

	clearances [pending_clearance].tallest = tallest;

	clearances [pending_clearance].terrain = terrain;

	clearances [pending_clearance].after_tallest = after_tallest;
}

void eech_sho_core_clearance_final (uint32_t final_word)
{
	if (pending_clearance < 0)
	{
		/* the matching record was not stored (overflow) or no tallest line preceded: the overflow counter or the mismatch counter says which */
		if (clearance_overflow == 0)
		{
			mismatches++;
		}

		return;
	}

	clearances [pending_clearance].final_word = final_word;

	clearances [pending_clearance].final_seen = 1;

	pending_clearance = -1;
}

void eech_sho_core_session (int pack, uint32_t word, int x_sector, int z_sector)
{
	struct sho_event *ev;

	if (pack)
	{
		pack_total++;
	}
	else
	{
		unpack_total++;
	}

	ev = begin_event (pack ? SHO_PACK : SHO_UNPACK);

	if (ev)
	{
		if (pack)
		{
			pack_stored++;
		}
		else
		{
			unpack_stored++;
		}

		ev->sx = x_sector;

		ev->sz = z_sector;

		ev->after = word;
	}
}

/* ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- */
/* the hooks: read what the original statement next to the hook reads, then call the core */

static uint32_t bits (const float *word)
{
	uint32_t value;

	memcpy (&value, word, sizeof (value));

	return value;
}

static void sector_coordinates (const void *sector_entity, int *x, int *z)
{
	sector *raw = (sector *) get_local_entity_data ((entity *) sector_entity);

	*x = (int) raw->x_sector;

	*z = (int) raw->z_sector;
}

static int family_of (entity *e)
{
	switch (get_local_entity_type (e))
	{
		case ENTITY_TYPE_CITY: return SHO_FAMILY_CITY;
		case ENTITY_TYPE_CITY_BUILDING: return SHO_FAMILY_CITY_BUILDING;
		default: return SHO_FAMILY_FIXED;
	}
}

void eech_sho_sector_insert (const void *en, const void *sector_entity)
{
	entity *e = (entity *) en;
	int family = family_of (e);
	int fixed = get_local_entity_int_value (e, INT_TYPE_IDENTIFY_FIXED) ? 1 : 0;
	int sx, sz;

	if (!fixed && family != SHO_FAMILY_CITY)
	{
		eech_sho_core_insert_mobile ();

		return;
	}

	sector_coordinates (sector_entity, &sx, &sz);

	eech_sho_core_insert ((int) get_local_entity_index (e), (int) get_local_entity_type (e), family, sx, sz);
}

void eech_sho_sector_remove (const void *en)
{
	entity *e = (entity *) en;
	int family = family_of (e);
	int fixed = get_local_entity_int_value (e, INT_TYPE_IDENTIFY_FIXED) ? 1 : 0;

	eech_sho_core_remove ((int) get_local_entity_index (e), (int) get_local_entity_type (e), family, fixed || family == SHO_FAMILY_CITY);
}

void eech_sho_link_begin (const void *sector_entity, const void *sender, const float *word)
{
	(void) sector_entity;
	(void) sender;

	eech_sho_core_link_begin (bits (word));
}

void eech_sho_link_end (const void *sector_entity, const void *sender, const float *word)
{
	entity *s = (entity *) sender;
	int family = family_of (s);
	int fixed = get_local_entity_int_value (s, INT_TYPE_IDENTIFY_FIXED) ? 1 : 0;
	int shape_state = SHO_SHAPE_UNOBSERVED, shape = 0;
	uint32_t pos [3], ymax = 0;
	int have_pos = 0, sx, sz;

	if (fixed)
	{
		/* the shape and, for a valid one, the position and the box top: exactly the reads the original block makes, at the same gate */
		shape = (int) get_local_entity_int_value (s, INT_TYPE_OBJECT_3D_SHAPE);

		shape_state = (shape != OBJECT_3D_INVALID_OBJECT_INDEX) ? SHO_SHAPE_VALID : SHO_SHAPE_INVALID;

		if (shape_state == SHO_SHAPE_VALID)
		{
			memcpy (pos, get_local_entity_vec3d_ptr (s, VEC3D_TYPE_POSITION), sizeof (pos));

			have_pos = 1;

			ymax = bits (&get_object_3d_bounding_box ((object_3d_index_numbers) shape)->ymax);
		}
	}

	sector_coordinates (sector_entity, &sx, &sz);

	eech_sho_core_link_end ((int) get_local_entity_index (s), (int) get_local_entity_type (s), family, fixed || family == SHO_FAMILY_CITY, shape_state, shape,
		have_pos ? pos : NULL, ymax, bits (word), sx, sz);
}

void eech_sho_statement_fixed (const void *en, int type, const char *key, const float *state)
{
	uint32_t words [CAMPAIGN_FIXED_STATE_WORDS];
	int i;

	for (i = 0; i < CAMPAIGN_FIXED_STATE_WORDS; i++)
	{
		words [i] = bits (&state [i]);
	}

	eech_sho_core_statement ((int) get_local_entity_index ((entity *) en), type, family_of ((entity *) en), key, words, CAMPAIGN_FIXED_STATE_WORDS);
}

void eech_sho_statement_city (const void *en, const char *key, const float *city)
{
	uint32_t words [CAMPAIGN_CITY_STATE_WORDS];
	int i;

	for (i = 0; i < CAMPAIGN_CITY_STATE_WORDS; i++)
	{
		words [i] = bits (&city [i]);
	}

	eech_sho_core_statement ((int) get_local_entity_index ((entity *) en), (int) ENTITY_TYPE_CITY, SHO_FAMILY_CITY, key, words, CAMPAIGN_CITY_STATE_WORDS);
}

void eech_sho_scan_first (int x_sector, int z_sector, const float *word)
{
	eech_sho_core_scan_first (x_sector, z_sector, bits (word));
}

void eech_sho_candidate_fact (const float *word)
{
	eech_sho_core_candidate_fact (bits (word));
}

void eech_sho_clearance_leader (void)
{
	eech_sho_core_clearance_leader ();
}

void eech_sho_clearance_follow (void)
{
	eech_sho_core_clearance_follow ();
}

void eech_sho_clearance_tallest (const void *sector_entity, const float *terrain_elevation, const float *min_height)
{
	sector *raw = (sector *) get_local_entity_data ((entity *) sector_entity);

	eech_sho_core_clearance_tallest ((int) raw->x_sector, (int) raw->z_sector, bits (&raw->tallest_structure_height), bits (terrain_elevation), bits (min_height));
}

void eech_sho_clearance_final (const float *min_height)
{
	eech_sho_core_clearance_final (bits (min_height));
}

void eech_sho_pack (const float *word, int x_sector, int z_sector)
{
	eech_sho_core_session (1, bits (word), x_sector, z_sector);
}

void eech_sho_unpack (const float *word, int x_sector, int z_sector)
{
	eech_sho_core_session (0, bits (word), x_sector, z_sector);
}

void eech_sho_entity_release (const void *en)
{
	eech_sho_core_release ((int) get_local_entity_index ((entity *) en));
}

/* ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- */

struct out
{
	char *text;
	int capacity;
	int used;
	int overflow;
};

static void put (struct out *o, const char *format, ...)
{
	va_list args;
	int n;

	if (o->overflow)
	{
		return;
	}

	va_start (args, format);

	n = vsnprintf (o->text + o->used, (size_t) (o->capacity - o->used), format, args);

	va_end (args);

	if (n < 0 || n >= o->capacity - o->used)
	{
		o->overflow = 1;

		return;
	}

	o->used += n;
}

static int whole (void)
{
	return (event_overflow == 0) && (clearance_overflow == 0) && (scan_out_of_range == 0) && (mismatches == 0) && (pack_total == pack_stored) && (unpack_total == unpack_stored) &&
		!pending_active && (pending_clearance < 0);
}

static const char *kind_name (int kind)
{
	switch (kind)
	{
		case SHO_INSERT: return "insert";
		case SHO_LINK: return "link";
		case SHO_STMT_FIXED: return "stmt_fixed";
		case SHO_STMT_CITY: return "stmt_city";
		case SHO_PACK: return "pack";
		case SHO_UNPACK: return "unpack";
		case SHO_REMOVE: return "remove";
		case SHO_RELEASE: return "release";
	}

	return "unknown";
}

static const char *family_name (int family)
{
	switch (family)
	{
		case SHO_FAMILY_FIXED: return "fixed";
		case SHO_FAMILY_CITY: return "city";
		case SHO_FAMILY_CITY_BUILDING: return "city_building";
	}

	return "unknown";
}

static const char *shape_state_name (int state)
{
	switch (state)
	{
		case SHO_SHAPE_UNOBSERVED: return "unobserved";
		case SHO_SHAPE_VALID: return "valid";
		case SHO_SHAPE_INVALID: return "invalid";
	}

	return "unknown";
}

#define U(X) ((unsigned long long) (X))

int eech_sho_summary_run (char *text, int capacity)
{
	struct out o;

	o.text = text;
	o.capacity = capacity;
	o.used = 0;
	o.overflow = 0;
	text [0] = '\0';

	put (&o, "sho-summary schema=2 note=read-only-observer-one-boot-per-process\n");
	put (&o, "sho-counts seq=%llu inserts_total=%llu inserts_fixed=%llu inserts_city=%llu inserts_mobile=%llu removes_total=%llu removes_recorded=%llu removes_mobile=%llu released_open=%llu\n",
		U (seq), U (inserts_total), U (inserts_fixed), U (inserts_city), U (inserts_mobile), U (removes_total), U (removes_recorded), U (removes_mobile), U (released_open));
	put (&o, "sho-join records=%u link_total=%llu link_fixed=%llu link_city=%llu link_mobile=%llu links_joined=%llu links_unjoined=%llu stmt_joined=%llu stmt_unjoined=%llu stmt_type_mismatch=%llu stmt_child=%llu double_insert=%llu remove_unjoined=%llu index_out_of_range=%llu\n",
		(unsigned) record_count, U (link_total), U (link_fixed), U (link_city), U (link_mobile), U (links_joined), U (links_unjoined), U (stmt_joined), U (stmt_unjoined), U (stmt_type_mismatch),
		U (stmt_child), U (double_insert), U (remove_unjoined), U (index_out_of_range));
	put (&o, "sho-shape fixed_valid=%llu fixed_invalid=%llu fixed_unobserved=%llu city_valid=%llu city_invalid=%llu city_unobserved=%llu raised=%llu\n",
		U (shape_count [0][SHO_SHAPE_VALID]), U (shape_count [0][SHO_SHAPE_INVALID]), U (shape_count [0][SHO_SHAPE_UNOBSERVED]),
		U (shape_count [1][SHO_SHAPE_VALID]), U (shape_count [1][SHO_SHAPE_INVALID]), U (shape_count [1][SHO_SHAPE_UNOBSERVED]), U (raised));
	put (&o, "sho-statements fixed=%llu city=%llu\n", U (statements_fixed), U (statements_city));
	put (&o, "sho-scan reads=%llu firsts=%llu sectors=%d fact_offers=%llu fact_nonzero=%llu fact_equal_first=%llu fact_differ_first=%llu fact_without_first=%llu\n",
		U (scan_reads), U (scan_firsts), scan_touched, U (fact_offers), U (fact_nonzero), U (fact_equal), U (fact_differ), U (fact_without_first));
	put (&o, "sho-clearance leader_reached=%llu follow_reached=%llu reads=%llu stored=%d\n", U (leader_reached), U (follow_reached), U (clearance_total), clearance_stored);
	put (&o, "sho-session pack_total=%llu pack_stored=%llu pack_omitted=%llu unpack_total=%llu unpack_stored=%llu unpack_omitted=%llu\n",
		U (pack_total), U (pack_stored), U (pack_total - pack_stored), U (unpack_total), U (unpack_stored), U (unpack_total - unpack_stored));
	put (&o, "sho-tables events_used=%d events_capacity=%d events_overflow=%llu clearance_capacity=%d clearance_overflow=%llu scan_out_of_range=%llu mismatches=%llu pending=%d\n",
		event_count, SHO_EVENTS, U (event_overflow), SHO_CLEARANCES, U (clearance_overflow), U (scan_out_of_range), U (mismatches), pending_active || (pending_clearance >= 0));
	put (&o, "sho-complete value=%d\n", whole ());

	return o.overflow ? -1 : 0;
}

int eech_sho_event_run (int index, char *text, int capacity)
{
	struct out o;
	const struct sho_event *e;
	int i;

	if (index < 0 || index >= event_count)
	{
		return -1;
	}

	e = &events [index];

	o.text = text;
	o.capacity = capacity;
	o.used = 0;
	o.overflow = 0;
	text [0] = '\0';

	put (&o, "sho-event i=%d seq=%llu kind=%s rec=%u ent=%d sx=%d sz=%d etype=%d family=%s shape_state=%s shape=%d before=0x%08x after=0x%08x pos=0x%08x,0x%08x,0x%08x ymax=0x%08x key=%s words=",
		index, U (e->seq), kind_name (e->kind), (unsigned) e->rec, e->ent, e->sx, e->sz, e->etype, family_name (e->family), shape_state_name (e->shape_state), e->shape,
		(unsigned) e->before, (unsigned) e->after, (unsigned) e->pos [0], (unsigned) e->pos [1], (unsigned) e->pos [2], (unsigned) e->ymax, e->key [0] ? e->key : "-");

	for (i = 0; i < e->word_count; i++)
	{
		put (&o, "%s0x%08x", i ? "," : "", (unsigned) e->words [i]);
	}

	if (e->word_count == 0)
	{
		put (&o, "-");
	}

	put (&o, "\n");

	return o.overflow ? -1 : 0;
}

int eech_sho_clearance_run (int index, char *text, int capacity)
{
	struct out o;
	const struct sho_clearance *c;

	if (index < 0 || index >= clearance_stored)
	{
		return -1;
	}

	c = &clearances [index];

	o.text = text;
	o.capacity = capacity;
	o.used = 0;
	o.overflow = 0;
	text [0] = '\0';

	put (&o, "sho-clearance i=%d seq=%llu sx=%d sz=%d tallest=0x%08x terrain=0x%08x after_tallest=0x%08x final=0x%08x final_seen=%d\n",
		index, U (c->seq), c->sx, c->sz, (unsigned) c->tallest, (unsigned) c->terrain, (unsigned) c->after_tallest, (unsigned) c->final_word, c->final_seen);

	return o.overflow ? -1 : 0;
}

int eech_sho_scan_run (int index, char *text, int capacity)
{
	struct out o;
	const struct sho_scan_cell *c;

	if (index < 0 || index >= scan_touched)
	{
		return -1;
	}

	c = &scan_cells [scan_order [index]];

	o.text = text;
	o.capacity = capacity;
	o.used = 0;
	o.overflow = 0;
	text [0] = '\0';

	put (&o, "sho-scan i=%d sx=%d sz=%d reads=%llu first_seq=%llu last_seq=%llu first_word=0x%08x last_word=0x%08x changes=%llu fact_offers=%llu fact_equal_first=%llu fact_differ_first=%llu fact_first_word=0x%08x fact_last_word=0x%08x fact_changes=%llu\n",
		index, c->sx, c->sz, U (c->reads), U (c->first_seq), U (c->last_seq), (unsigned) c->first_word, (unsigned) c->last_word, U (c->changes),
		U (c->fact_offers), U (c->fact_equal), U (c->fact_differ), (unsigned) c->fact_first_word, (unsigned) c->fact_last_word, U (c->fact_changes));

	return o.overflow ? -1 : 0;
}

#endif
