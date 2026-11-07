/*
 * TEST-ONLY (EEDCS-49, D3, sector-height public baseline; feature `sector-height-observer`): the READ-ONLY observer of the sector tallest-structure high-water value on the
 * real public path (Lua host -> public native module -> the real en_list funnel, the real sector response, the host roster statements, the hosted scan and candidate questions,
 * the fixed-wing clearance and the session pack). The plan is docs/d3-sector-height-public-baseline-plan.md. This file holds the hook definitions (eech_sho_hooks.h) and the
 * report formatting; the engine entries that expose the report are in eech_engine.c.
 *
 * Observation only. Nothing here changes a word the campaign holds, the order of a statement, a clock or the random stream; floats arrive as pointers and leave as 32-bit
 * patterns (no floating-point arithmetic, so the engine's rounding mode and any signalling NaN are untouched). The tables are bounded and private; a table that fills latches an
 * overflow counter and the report says `complete=0`. Single-threaded bookkeeping: the engine runs behind one entry lock. ONE boot per process: the tables are never reset.
 *
 * The report is `key=value` text. A recorded word is a hex bit pattern; a sequence number orders every record across all kinds (the aggregate cells stamp their first and last
 * read, so the order of a boot-time raise against the first scan read is visible).
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
#define SHO_PACK_EVENTS 64

enum sho_kind { SHO_INSERT = 1, SHO_LINK, SHO_STMT_FIXED, SHO_STMT_CITY, SHO_PACK, SHO_UNPACK };

struct sho_event
{
	uint64_t seq;
	int kind;
	int sx, sz;
	int etype, fixed, shape, valid;
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
};

static struct sho_event events [SHO_EVENTS];
static struct sho_clearance clearances [SHO_CLEARANCES];
static struct sho_scan_cell scan_cells [SHO_SCAN_SIDE * SHO_SCAN_SIDE];
static int scan_order [SHO_SCAN_SIDE * SHO_SCAN_SIDE];
static int scan_touched;

static uint64_t seq;
static int event_count, clearance_stored;
static uint64_t event_overflow, clearance_overflow, scan_out_of_range, mismatches;
static uint64_t inserts_total, inserts_fixed, inserts_city, inserts_mobile;
static uint64_t link_total, link_fixed, link_city, link_mobile, link_fixed_valid, link_fixed_invalid, link_city_valid, link_city_invalid, raised;
static uint64_t statements_fixed, statements_city, pack_count, unpack_count, scan_reads;
static uint64_t candidate_offers, candidate_nonzero, leader_reached, follow_reached, clearance_total;
static int pending_active;
static uint32_t pending_before;
static int pending_clearance = -1;

static uint32_t bits (const float *word)
{
	uint32_t value;

	memcpy (&value, word, sizeof (value));

	return value;
}

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

static void sector_coordinates (const void *sector_entity, int *x, int *z)
{
	sector *raw = (sector *) get_local_entity_data ((entity *) sector_entity);

	*x = (int) raw->x_sector;

	*z = (int) raw->z_sector;
}

void eech_sho_sector_insert (const void *en, const void *sector_entity)
{
	entity *e = (entity *) en;
	struct sho_event *ev;
	int city = (get_local_entity_type (e) == ENTITY_TYPE_CITY);
	int fixed = get_local_entity_int_value (e, INT_TYPE_IDENTIFY_FIXED) ? 1 : 0;

	inserts_total++;

	if (city)
	{
		inserts_city++;
	}
	else if (fixed)
	{
		inserts_fixed++;
	}
	else
	{
		inserts_mobile++;

		return;
	}

	ev = begin_event (SHO_INSERT);

	if (ev)
	{
		ev->etype = (int) get_local_entity_type (e);

		ev->fixed = fixed;

		sector_coordinates (sector_entity, &ev->sx, &ev->sz);
	}
}

void eech_sho_link_begin (const void *sector_entity, const void *sender, const float *word)
{
	(void) sector_entity;
	(void) sender;

	if (pending_active)
	{
		mismatches++;
	}

	pending_active = 1;

	pending_before = bits (word);
}

void eech_sho_link_end (const void *sector_entity, const void *sender, const float *word)
{
	entity *s = (entity *) sender;
	struct sho_event *ev;
	int city = (get_local_entity_type (s) == ENTITY_TYPE_CITY);
	int fixed = get_local_entity_int_value (s, INT_TYPE_IDENTIFY_FIXED) ? 1 : 0;
	int shape = 0, valid = 0;
	uint32_t after = bits (word);

	if (!pending_active)
	{
		mismatches++;
	}

	pending_active = 0;

	link_total++;

	if (!fixed && !city)
	{
		link_mobile++;

		return;
	}

	if (after != pending_before)
	{
		raised++;
	}

	ev = begin_event (SHO_LINK);

	if (fixed)
	{
		/* the shape and, for a valid one, the position and the box top: exactly the reads the original block makes */
		shape = (int) get_local_entity_int_value (s, INT_TYPE_OBJECT_3D_SHAPE);

		valid = (shape != OBJECT_3D_INVALID_OBJECT_INDEX);
	}

	if (city)
	{
		link_city++;

		if (valid)
		{
			link_city_valid++;
		}
		else
		{
			link_city_invalid++;
		}
	}
	else
	{
		link_fixed++;

		if (valid)
		{
			link_fixed_valid++;
		}
		else
		{
			link_fixed_invalid++;
		}
	}

	if (ev)
	{
		ev->etype = (int) get_local_entity_type (s);

		ev->fixed = fixed;

		ev->shape = shape;

		ev->valid = valid;

		ev->before = pending_before;

		ev->after = after;

		sector_coordinates (sector_entity, &ev->sx, &ev->sz);

		if (valid)
		{
			memcpy (ev->pos, get_local_entity_vec3d_ptr (s, VEC3D_TYPE_POSITION), sizeof (ev->pos));

			{
				const float *top = &get_object_3d_bounding_box ((object_3d_index_numbers) shape)->ymax;

				ev->ymax = bits (top);
			}
		}
	}
}

void eech_sho_statement_fixed (const void *en, int type, const char *key, const float *state)
{
	struct sho_event *ev;
	int i;

	(void) en;

	statements_fixed++;

	ev = begin_event (SHO_STMT_FIXED);

	if (ev)
	{
		ev->etype = type;

		copy_key (ev->key, key);

		ev->word_count = CAMPAIGN_FIXED_STATE_WORDS;

		for (i = 0; i < CAMPAIGN_FIXED_STATE_WORDS; i++)
		{
			ev->words [i] = bits (&state [i]);
		}
	}
}

void eech_sho_statement_city (const void *en, const char *key, const float *city)
{
	struct sho_event *ev;
	int i;

	(void) en;

	statements_city++;

	ev = begin_event (SHO_STMT_CITY);

	if (ev)
	{
		ev->etype = (int) ENTITY_TYPE_CITY;

		copy_key (ev->key, key);

		ev->word_count = CAMPAIGN_CITY_STATE_WORDS;

		for (i = 0; i < CAMPAIGN_CITY_STATE_WORDS; i++)
		{
			ev->words [i] = bits (&city [i]);
		}
	}
}

void eech_sho_scan_first (int x_sector, int z_sector, const float *word)
{
	struct sho_scan_cell *c;
	uint32_t value = bits (word);

	seq++;

	scan_reads++;

	if (x_sector < 0 || z_sector < 0 || x_sector >= SHO_SCAN_SIDE || z_sector >= SHO_SCAN_SIDE)
	{
		scan_out_of_range++;

		return;
	}

	c = &scan_cells [x_sector + z_sector * SHO_SCAN_SIDE];

	if (!c->touched)
	{
		c->touched = 1;

		c->sx = x_sector;

		c->sz = z_sector;

		c->first_seq = seq;

		c->first_word = value;

		scan_order [scan_touched++] = x_sector + z_sector * SHO_SCAN_SIDE;
	}
	else if (value != c->last_word)
	{
		c->changes++;
	}

	c->reads++;

	c->last_seq = seq;

	c->last_word = value;
}

void eech_sho_candidate_fact (const float *word)
{
	seq++;

	candidate_offers++;

	if (bits (word) != 0)
	{
		candidate_nonzero++;
	}
}

void eech_sho_clearance_leader (void)
{
	seq++;

	leader_reached++;
}

void eech_sho_clearance_follow (void)
{
	seq++;

	follow_reached++;
}

void eech_sho_clearance_tallest (const void *sector_entity, const float *terrain_elevation, const float *min_height)
{
	sector *raw = (sector *) get_local_entity_data ((entity *) sector_entity);

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

	clearances [pending_clearance].sx = (int) raw->x_sector;

	clearances [pending_clearance].sz = (int) raw->z_sector;

	clearances [pending_clearance].tallest = bits (&raw->tallest_structure_height);

	clearances [pending_clearance].terrain = bits (terrain_elevation);

	clearances [pending_clearance].after_tallest = bits (min_height);
}

void eech_sho_clearance_final (const float *min_height)
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

	clearances [pending_clearance].final_word = bits (min_height);

	clearances [pending_clearance].final_seen = 1;

	pending_clearance = -1;
}

static void pack_event (int kind, const float *word, int x_sector, int z_sector)
{
	struct sho_event *ev = begin_event (kind);

	if (ev)
	{
		ev->sx = x_sector;

		ev->sz = z_sector;

		ev->after = bits (word);
	}
}

void eech_sho_pack (const float *word, int x_sector, int z_sector)
{
	pack_count++;

	if (pack_count <= SHO_PACK_EVENTS)
	{
		pack_event (SHO_PACK, word, x_sector, z_sector);
	}
}

void eech_sho_unpack (const float *word, int x_sector, int z_sector)
{
	unpack_count++;

	if (unpack_count <= SHO_PACK_EVENTS)
	{
		pack_event (SHO_UNPACK, word, x_sector, z_sector);
	}
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
	return (event_overflow == 0) && (clearance_overflow == 0) && (scan_out_of_range == 0) && (mismatches == 0) && !pending_active && (pending_clearance < 0);
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
	}

	return "unknown";
}

int eech_sho_summary_run (char *text, int capacity)
{
	struct out o;

	o.text = text;
	o.capacity = capacity;
	o.used = 0;
	o.overflow = 0;
	text [0] = '\0';

	put (&o, "sho-summary schema=1 note=read-only-observer-one-boot-per-process\n");
	put (&o, "sho-counts seq=%llu inserts_total=%llu inserts_fixed=%llu inserts_city=%llu inserts_mobile=%llu link_total=%llu link_fixed=%llu link_city=%llu link_mobile=%llu\n",
		(unsigned long long) seq, (unsigned long long) inserts_total, (unsigned long long) inserts_fixed, (unsigned long long) inserts_city, (unsigned long long) inserts_mobile,
		(unsigned long long) link_total, (unsigned long long) link_fixed, (unsigned long long) link_city, (unsigned long long) link_mobile);
	put (&o, "sho-link-shape fixed_valid=%llu fixed_invalid=%llu city_valid=%llu city_invalid=%llu raised=%llu\n",
		(unsigned long long) link_fixed_valid, (unsigned long long) link_fixed_invalid, (unsigned long long) link_city_valid, (unsigned long long) link_city_invalid, (unsigned long long) raised);
	put (&o, "sho-statements fixed=%llu city=%llu\n", (unsigned long long) statements_fixed, (unsigned long long) statements_city);
	put (&o, "sho-scan reads=%llu sectors=%d candidate_offers=%llu candidate_nonzero=%llu\n",
		(unsigned long long) scan_reads, scan_touched, (unsigned long long) candidate_offers, (unsigned long long) candidate_nonzero);
	put (&o, "sho-clearance leader_reached=%llu follow_reached=%llu reads=%llu stored=%d\n",
		(unsigned long long) leader_reached, (unsigned long long) follow_reached, (unsigned long long) clearance_total, clearance_stored);
	put (&o, "sho-session pack=%llu unpack=%llu\n", (unsigned long long) pack_count, (unsigned long long) unpack_count);
	put (&o, "sho-tables events_used=%d events_capacity=%d events_overflow=%llu clearance_capacity=%d clearance_overflow=%llu scan_out_of_range=%llu mismatches=%llu pending=%d\n",
		event_count, SHO_EVENTS, (unsigned long long) event_overflow, SHO_CLEARANCES, (unsigned long long) clearance_overflow, (unsigned long long) scan_out_of_range,
		(unsigned long long) mismatches, pending_active || (pending_clearance >= 0));
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

	put (&o, "sho-event i=%d seq=%llu kind=%s sx=%d sz=%d etype=%d fixed=%d shape=%d valid=%d before=0x%08x after=0x%08x pos=0x%08x,0x%08x,0x%08x ymax=0x%08x key=%s words=",
		index, (unsigned long long) e->seq, kind_name (e->kind), e->sx, e->sz, e->etype, e->fixed, e->shape, e->valid, (unsigned) e->before, (unsigned) e->after,
		(unsigned) e->pos [0], (unsigned) e->pos [1], (unsigned) e->pos [2], (unsigned) e->ymax, e->key [0] ? e->key : "-");

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
		index, (unsigned long long) c->seq, c->sx, c->sz, (unsigned) c->tallest, (unsigned) c->terrain, (unsigned) c->after_tallest, (unsigned) c->final_word, c->final_seen);

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

	put (&o, "sho-scan i=%d sx=%d sz=%d reads=%llu first_seq=%llu last_seq=%llu first_word=0x%08x last_word=0x%08x changes=%llu\n",
		index, c->sx, c->sz, (unsigned long long) c->reads, (unsigned long long) c->first_seq, (unsigned long long) c->last_seq, (unsigned) c->first_word, (unsigned) c->last_word,
		(unsigned long long) c->changes);

	return o.overflow ? -1 : 0;
}

#endif
