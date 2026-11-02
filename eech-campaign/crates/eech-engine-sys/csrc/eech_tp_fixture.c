/*
 * TEST-ONLY original target-point characterisation (EEDCS-52, D3). Compiled only with the `target-point-fixture` feature; never part of the shipping module.
 *
 * WHAT IT OBSERVES. Two things the ORIGINAL code does, with the hooks of eech_tp_hooks.h placed (build/patches.rs, TARGET_POINT_FIXTURE_PATCHES) between original statements of
 * the STAGED copies of the sources; the sources in the tree are never edited, so the shipping build contains none of this.
 *
 *   1. THE QUERY. Every call of get_local_entity_target_point (en_maths.c) that the campaign's own update loop makes: the caller that reached it (weapon.c fill_aim_target with its
 *      aim mode, or the weapon step's target-vector lookup; any other caller, such as a camera or the HUD, is "unattributed" and counted), the branch it took (1 an instance with a
 *      TARGET_POINT sub-object, 2 an instance without one, 3 no instance and a fixed object of a valid shape, 4 the position fallback: no instance and either not a fixed object or
 *      an invalid shape), the entity's type, the host identity the host ALREADY issued for it (campaign_object_host_id only reads; it never asks the host, so the observer allocates
 *      no identity), the semantic model (the scene name of the instance, or of the shape), the offset it read, for a fixed shape whether a horizontal offset exists, the heading it read and
 *      whether the rotated x/z offsets were added (a heading of zero adds none: the legacy quirk, recorded, not corrected), the float bits it returned, and the instance viewpoint
 *      (position and attitude, twelve words) before and after the call, because branch 1 overwrites it from the entity pose.
 *   2. THE CENSUS. initialise_target_point_offsets (wn_dbase.c) fills target_point_offsets[]. It is called once per initialisation of the entity system, and one public boot runs that
 *      twice (full_initialise_game, then reinitialise_entity_system: both reach overload_weapon_functions), so a completed boot holds TWO original passes, each of which first runs
 *      calculate_seeker_limits_from_launch_angle once (campaign weapon policy, unchanged and not deduplicated). The census keeps EVERY pass as its own bounded GENERATION with an
 *      ordinal: per model index, whether the offset came from the model's TARGET_POINT sub-object, from the bounding-box fallback (with the six bounds it used) or is the invalid
 *      index's zero, and the float bits. A duplicate inside one pass, an index outside the table, a pass that never ends, a pass beyond the bounded history and a hook outside a pass
 *      are each counted. A pass is a historical snapshot: it can be complete without equalling the CURRENT table (a later pass may have written other words), so each pass reports how
 *      many of its rows the live table still holds, and only the latest pass is required to equal it. Each query records the latest ended generation at the time it ran. The census
 *      records facts the original already computed; it constructs and measures no model of its own.
 *
 *   3. THE CONTROLLED PROBE (eech_tp_probe_run, only when the driver asks, once, after the boot and before any frame). The natural witnesses reach only branch 1. The probe finds, among
 *      the live entities of the families a target can be (anti-aircraft, fixed wing, helicopter, person, routed vehicle, ship, and the structures), those for which the original takes
 *      each unreached case, using only reads the original itself makes (the instance and the TARGET_POINT search, the fixed flag, the shape, whether the shape's offset has a horizontal
 *      part, the heading), and then calls the ORIGINAL get_local_entity_target_point once per pick (at most TP_PROBE_PICKS per case). The call is observed by the same hooks as any
 *      other, as caller class "controlled_probe" with the case as its detail, so its answer, its identity and its (absent) viewpoint write are the original's own. It is NOT a campaign
 *      target selection and NOT aim or weapon coverage: the entity is chosen by the fixture, and the driver keeps these queries apart from the natural ones. Branches 2, 3 and 4 write
 *      nothing (only branch 1 writes the instance viewpoint, and it is not probed), so the probe changes no campaign state; the picks, the counts of candidates and the cases with no
 *      candidate are reported, and a case with none is UNAVAILABLE, never succeeded. Nothing is constructed, set or restored.
 *
 *   4. THE SETUP PROBE (eech_tp_setup_run, a separate opt-in mode, once, after the boot and before any frame). The read-only probe found NO live candidate for three cases: an instance
 *      without a TARGET_POINT, a mobile with no instance, a fixed object with the invalid shape. This mode gives each a real, explicitly declared physical PRECONDITION on a real live
 *      entity, calls the ORIGINAL once, and restores the precondition before anything else runs: (1) a real aircraft or vehicle with an instance has that instance pointer replaced by a
 *      temporarily constructed instance of a real census bounding-box scene that carries no random skin (construct_3d_object draws rand() only for a skin, so the choice needs none), then
 *      restored and the temporary instance destructed; (2) the instance pointer of a real mobile (possibly another one than case 1's) set to NULL, then restored; (3) a real fixed structure's shape set to the invalid index through its typed
 *      setter, then restored. There is no typed setter for the instance pointer (the aircraft and vehicle pointer setters handle only the sound effect), so the field the typed getter
 *      reads is written directly: a fixture-only use of the private layout that never reaches the wire. Each case verifies the change took (through the typed getters), verifies the
 *      restoration (the pointer, the shape, the real instance's viewpoint words and, for the constructed instance, the engine's three construction counters), and a restoration that fails
 *      stops the run before any frame. The queries are caller class "setup_probe", not natural, not aim/weapon coverage and not host authority; the temporary instance is an environment
 *      precondition. The allocator's internal state is not claimed identical, only the counters named above.
 *
 *   5. THE ORIGINAL INPUTS OF THE PHYSICAL QUERY (the completeness step before any host extraction). Branches 2, 3 and 4 record the entity BASE position: the three words the original's own
 *      get_local_entity_vec3d has just copied into the result, read by a hook placed right after that statement and before any physical addition (branch 1 keeps its vp_after position and
 *      attitude, and its original write of the instance viewpoint). Branch 1 records, immediately before the original calls get_3d_sub_object_world_viewpoint, the instance's relative scale
 *      and the chain of sub-object placements that function is about to walk: it follows the same `parent` pointers from the sub-object the original's search already found up to the main
 *      object, copies each node's relative position, heading, pitch, roll and scale as bit patterns (topmost sub-object first, the order the original applies them), and says whether the walk
 *      ended at this instance. It is bounded (the original's own 256-slot array; eight nodes are kept), makes no second search and no write, and exposes no pointer. A chain deeper than eight
 *      nodes, a walk that does not end, a wrong root or a null result is reported as that state with its depth and NO nodes: never a truncated chain presented as whole.
 *   6. THE CRT RANDOM STREAM. Every rand () of the build is renamed (csrc/eech_tp_rand.h, force-included only with this feature) to eech_tp_counted_rand, which counts the draw and calls the
 *      real rand () once, in the same place, with the same arguments and result; the single srand () of the build (random.c set_random_number_seed, called by the campaign's own seeding)
 *      is announced by a hook placed before it. The observer keeps, per initialiser pass, the draws made inside it and the number of scenes whose `skin.animation` field predicts a draw,
 *      and a bounded ordered list of marks (seed, pass begin, pass end, and the first draw OUTSIDE any pass that follows a pass end with no reseed in between: the point where a draw count
 *      of the pass would be visible to a later consumer). It draws nothing, seeds nothing and reorders nothing. The campaign's own custom generator (random_number_seed, rand16 and
 *      friends) is a different generator and is not counted. It is single-threaded bookkeeping: the engine runs on one thread behind its entry lock.
 *
 * WHAT IT CANNOT SHOW. A call that no boot or frame reached is simply absent: an empty query report says "no calls", which is NOT a measured original, and a branch with no
 * call is not covered. A table that fills, a hook outside an open call, a call that never ends and a call that begins inside another are each counted and make the capture not whole.
 * The camera, HUD and player callers are not hooked; their calls are counted as unattributed when they happen. The AI attack guidance (gd_attck.c) reads target_point_offsets[]
 * directly and is not observed. Nothing here constructs a model, calls the function from outside the game's order, draws a random number or reads a clock.
 *
 * It runs inside the engine entry (so it sees EECH's round-toward-zero mode) and changes nothing it observes.
 */

#ifdef EECH_TARGET_POINT_FIXTURE

#include <ctype.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "project.h"

#include "entity/mobile/mb_hostaim.h"
#include "entity/mobile/mb_hostobj.h"
#include "eech_tp_hooks.h"

/* eech_tp_rand.h (force-included) renamed every rand () to eech_tp_counted_rand; the one real call is inside that function */
#undef rand
extern int rand (void);

#define TP_EVENTS 4096
#define TP_HOST_ID 128
/* the most original initialiser passes kept; one boot has been seen to run two */
#define TP_GENERATIONS 4
/* sub-object nodes kept per branch-1 chain (a deeper chain is reported as such, without nodes), and the original's own limit for the walk (objects[256] in get_3d_sub_object_world_viewpoint) */
#define TP_CHAIN 8
#define TP_WALK_MAX 256
/* ordered CRT-stream marks kept */
#define TP_RNG_MARKS 32

enum
{
	TP_SITE_UNATTRIBUTED,
	TP_SITE_AIM_VECTORS,
	TP_SITE_AIM_LEAD,
	TP_SITE_AIM_INTERCEPT,
	TP_SITE_WEAPON_STEP,
	TP_SITE_OTHER,
	/* the controlled probe (eech_tp_probe_run): never a natural caller */
	TP_SITE_CONTROLLED,
	/* the setup probe (eech_tp_setup_run): never a natural caller either */
	TP_SITE_SETUP,
	TP_SITES
};

enum
{
	TP_CASE_TARGET_POINT,
	TP_CASE_INSTANCE_POSITION,
	TP_CASE_NO_HORIZONTAL,
	TP_CASE_HEADING_ZERO,
	TP_CASE_ROTATED,
	TP_CASE_NOT_FIXED,
	TP_CASE_INVALID_SHAPE,
	TP_CASES
};

enum
{
	TP_CHAIN_NONE,
	TP_CHAIN_COMPLETE,
	TP_CHAIN_EMPTY,
	TP_CHAIN_TOO_DEEP,
	TP_CHAIN_UNTERMINATED,
	TP_CHAIN_WRONG_ROOT,
	TP_CHAIN_NULL_RESULT
};

enum
{
	TP_MARK_SEED,
	TP_MARK_PASS_BEGIN,
	TP_MARK_PASS_END,
	TP_MARK_EXPOSED
};

enum
{
	TP_HOST_ID_ABSENT,
	TP_HOST_ID_PRESENT,
	TP_HOST_ID_OVERLONG
};

struct tp_event
{
	int seq, site, detail, branch, entity_type;
	/* the latest ended census generation when the query began (-1: none yet) and whether an initialiser pass was running */
	int gen_last, gen_open;
	int host_id_state, host_id_len;
	char host_id[TP_HOST_ID + 1];
	int instance, model, fixed, shape, shape_out_of_range, horizontal, heading_read, xz_applied;
	uint32_t heading;
	int offset_known;
	uint32_t offset[3], result[3];
	int vp_known, vp_changed;
	uint32_t vp_before[12], vp_after[12];
	/* the entity base position the original's get_local_entity_vec3d copied (branches 2, 3 and 4) */
	int base_known;
	uint32_t base[3];
	/* branch 1: the main object's relative scale, and the chain of placements get_3d_sub_object_world_viewpoint is about to apply, topmost first: position (3), heading, pitch, roll, scale (3) */
	int scale_known;
	uint32_t main_scale[3];
	int chain_state, chain_depth;
	uint32_t chain[TP_CHAIN][9];
};

struct tp_census_row
{
	int state; /* 0 not recorded, 1 sub-object, 2 bounding box, 3 the invalid index's zero */
	uint32_t offset[3], bounds[6];
};

static struct tp_event tp_events[TP_EVENTS];
static struct tp_event tp_current;
static const object_3d_instance *tp_current_instance;
static int tp_open;
static int tp_event_count, tp_calls, tp_dropped, tp_stale, tp_interrupted, tp_orphans, tp_unclassified, tp_out_of_range, tp_overlong;
static int tp_pending, tp_pending_site, tp_pending_detail;
static int tp_matrix[TP_SITES][5];
static int tp_case_count[TP_CASES];
static int tp_vp_changed_count, tp_host_present, tp_host_absent;

struct tp_generation
{
	int ended, interrupted;
	int duplicates, out_of_range;
	/* CRT draws made while this pass ran, and the scenes whose skin field predicts one (read when the pass ended) */
	unsigned rng_draws;
	int skinned;
	struct tp_census_row rows[OBJECT_3D_LAST];
};

static struct tp_generation tp_generations[TP_GENERATIONS];
/* passes kept, passes begun beyond the history, the pass running (kept or not), the kept pass it fills (-1: none or not kept), the latest kept pass that ended, passes abandoned by the next begin, hooks outside a pass */
static int tp_gen_count, tp_gen_dropped, tp_gen_open, tp_gen_current = -1, tp_gen_last = -1, tp_gen_interrupted, tp_census_stray;

/* the CRT random stream (see the header comment, item 6) */
struct tp_rng_mark
{
	int kind, gen;
	uint32_t value;
	unsigned draws, since_seed;
};

static struct tp_rng_mark tp_rng_marks[TP_RNG_MARKS];
static int tp_rng_mark_count;
static unsigned tp_rng_marks_dropped, tp_rng_draws, tp_rng_since_seed, tp_rng_seeds, tp_rng_pass_draws, tp_rng_outside, tp_rng_exposed, tp_rng_pass_begin_draws;
/* the pass whose end no seed has followed yet (-1: a pass beyond the history), or -2 for none */
static int tp_rng_after_pass = -2;

static void rng_mark (int kind, int gen, uint32_t value)
{
	if (tp_rng_mark_count < TP_RNG_MARKS)
	{
		struct tp_rng_mark *mark = &tp_rng_marks[tp_rng_mark_count];

		mark->kind = kind;
		mark->gen = gen;
		mark->value = value;
		mark->draws = tp_rng_draws;
		mark->since_seed = tp_rng_since_seed;
		tp_rng_mark_count++;
	}
	else
	{
		tp_rng_marks_dropped++;
	}
}

/* every rand () of the build: counts, then makes the original's one call, unchanged */
int eech_tp_counted_rand (void)
{
	tp_rng_draws++;
	tp_rng_since_seed++;
	if (tp_gen_open)
	{
		tp_rng_pass_draws++;
	}
	else
	{
		tp_rng_outside++;
		if (tp_rng_after_pass != -2)
		{
			/* the first draw after a pass end with no reseed between: the stream position the pass left is now observable */
			rng_mark (TP_MARK_EXPOSED, tp_rng_after_pass, 0);
			tp_rng_exposed++;
			tp_rng_after_pass = -2;
		}
	}
	return rand ();
}

/* announced right before the original's srand (seed) in set_random_number_seed */
void eech_tp_seed (int seed)
{
	rng_mark (TP_MARK_SEED, -1, (uint32_t) seed);
	tp_rng_seeds++;
	tp_rng_since_seed = 0;
	tp_rng_after_pass = -2;
}

/* the scenes (1 .. OBJECT_3D_LAST - 1) whose skin field says skin_random draws once for them: skin_init sets `animation` for every scene from its name, -1 for none */
static int skinned_scenes (void)
{
	int i, count = 0;

	for (i = OBJECT_3D_INVALID_OBJECT_INDEX + 1; i < OBJECT_3D_LAST; i++)
	{
		if (objects_3d_scene_database[i].skin.animation >= 0)
		{
			count++;
		}
	}
	return count;
}

static uint32_t bits_of (const float *value)
{
	uint32_t word;
	memcpy (&word, value, sizeof (word));
	return word;
}

static void copy_vec (uint32_t *out, const float *x, const float *y, const float *z)
{
	out[0] = bits_of (x);
	out[1] = bits_of (y);
	out[2] = bits_of (z);
}

static void copy_vp (uint32_t *out, const object_3d_instance *instance)
{
	int i, j;

	copy_vec (out, &instance->vp.position.x, &instance->vp.position.y, &instance->vp.position.z);
	for (i = 0; i < 3; i++)
	{
		for (j = 0; j < 3; j++)
		{
			out[3 + (i * 3) + j] = bits_of (&instance->vp.attitude[i][j]);
		}
	}
}

static int site_class (int site, int detail)
{
	if (site == 1)
	{
		switch (detail)
		{
			case CAMPAIGN_AIM_VECTORS:
			{
				return TP_SITE_AIM_VECTORS;
			}
			case CAMPAIGN_AIM_LEAD:
			{
				return TP_SITE_AIM_LEAD;
			}
			case CAMPAIGN_AIM_INTERCEPT:
			{
				return TP_SITE_AIM_INTERCEPT;
			}
			default:
			{
				return TP_SITE_OTHER;
			}
		}
	}
	if (site == 2)
	{
		return detail == 1 ? TP_SITE_WEAPON_STEP : TP_SITE_OTHER;
	}
	if (site == 0)
	{
		return TP_SITE_UNATTRIBUTED;
	}
	if (site == 3)
	{
		return TP_SITE_CONTROLLED;
	}
	if (site == 4)
	{
		return TP_SITE_SETUP;
	}
	return TP_SITE_OTHER;
}

static const char *site_name (int site_class_value)
{
	switch (site_class_value)
	{
		case TP_SITE_UNATTRIBUTED:
		{
			return "unattributed";
		}
		case TP_SITE_AIM_VECTORS:
		{
			return "aim_vectors";
		}
		case TP_SITE_AIM_LEAD:
		{
			return "aim_lead";
		}
		case TP_SITE_AIM_INTERCEPT:
		{
			return "aim_intercept";
		}
		case TP_SITE_WEAPON_STEP:
		{
			return "weapon_step_target_vec";
		}
		case TP_SITE_CONTROLLED:
		{
			return "controlled_probe";
		}
		case TP_SITE_SETUP:
		{
			return "setup_probe";
		}
		default:
		{
			return "other";
		}
	}
}

static const char *branch_name (int branch)
{
	switch (branch)
	{
		case 1:
		{
			return "instance_target_point";
		}
		case 2:
		{
			return "instance_position";
		}
		case 3:
		{
			return "fixed_shape_offset";
		}
		case 4:
		{
			return "position_fallback";
		}
		default:
		{
			return "-";
		}
	}
}

static const char *chain_state_name (int state)
{
	switch (state)
	{
		case TP_CHAIN_COMPLETE:
		{
			return "complete";
		}
		case TP_CHAIN_EMPTY:
		{
			return "empty";
		}
		case TP_CHAIN_TOO_DEEP:
		{
			return "too_deep";
		}
		case TP_CHAIN_UNTERMINATED:
		{
			return "unterminated";
		}
		case TP_CHAIN_WRONG_ROOT:
		{
			return "wrong_root";
		}
		case TP_CHAIN_NULL_RESULT:
		{
			return "null_result";
		}
		default:
		{
			return "-";
		}
	}
}

static const char *mark_name (int kind)
{
	switch (kind)
	{
		case TP_MARK_SEED:
		{
			return "seed";
		}
		case TP_MARK_PASS_BEGIN:
		{
			return "pass_begin";
		}
		case TP_MARK_PASS_END:
		{
			return "pass_end";
		}
		default:
		{
			return "exposed";
		}
	}
}

/* the case within a branch: integer logic over the recorded flags only; -1 when the call reached no branch */
static int case_of (const struct tp_event *e)
{
	switch (e->branch)
	{
		case 1:
		{
			return TP_CASE_TARGET_POINT;
		}
		case 2:
		{
			return TP_CASE_INSTANCE_POSITION;
		}
		case 3:
		{
			if (!e->horizontal)
			{
				return TP_CASE_NO_HORIZONTAL;
			}
			return e->xz_applied ? TP_CASE_ROTATED : TP_CASE_HEADING_ZERO;
		}
		case 4:
		{
			return e->fixed ? TP_CASE_INVALID_SHAPE : TP_CASE_NOT_FIXED;
		}
		default:
		{
			return -1;
		}
	}
}

static const char *case_name (int value)
{
	switch (value)
	{
		case TP_CASE_TARGET_POINT:
		{
			return "target_point_subobject";
		}
		case TP_CASE_INSTANCE_POSITION:
		{
			return "instance_without_target_point";
		}
		case TP_CASE_NO_HORIZONTAL:
		{
			return "no_horizontal_offset";
		}
		case TP_CASE_HEADING_ZERO:
		{
			return "heading_zero_quirk";
		}
		case TP_CASE_ROTATED:
		{
			return "rotated_horizontal_offset";
		}
		case TP_CASE_NOT_FIXED:
		{
			return "not_fixed";
		}
		case TP_CASE_INVALID_SHAPE:
		{
			return "invalid_shape";
		}
		default:
		{
			return "-";
		}
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the query hooks */

void eech_tp_caller (int site, int detail)
{
	if (tp_pending)
	{
		/* the previous caller context was never consumed by a call */
		tp_stale++;
	}
	tp_pending = 1;
	tp_pending_site = site;
	tp_pending_detail = detail;
}

void eech_tp_begin (const void *en_pointer, const void *instance)
{
	struct ENTITY *en = (struct ENTITY *) en_pointer;
	const char *id;
	size_t length;

	if (tp_open)
	{
		/* the previous call never ended */
		tp_interrupted++;
	}
	memset (&tp_current, 0, sizeof (tp_current));
	tp_open = 1;
	tp_current.seq = tp_calls;
	tp_calls++;
	if (tp_pending)
	{
		tp_current.site = tp_pending_site;
		tp_current.detail = tp_pending_detail;
		tp_pending = 0;
	}
	tp_current.shape = -1;
	tp_current.model = -1;
	tp_current.gen_last = tp_gen_last;
	tp_current.gen_open = tp_gen_open;
	tp_current_instance = (const object_3d_instance *) instance;
	if (en)
	{
		tp_current.entity_type = (int) get_local_entity_type (en);
		/* only reads an identity the host already issued: it never asks the host and issues none */
		id = campaign_object_host_id (en);
		if (id && id[0])
		{
			length = strlen (id);
			if (length <= TP_HOST_ID)
			{
				memcpy (tp_current.host_id, id, length + 1);
				tp_current.host_id_len = (int) length;
				tp_current.host_id_state = TP_HOST_ID_PRESENT;
			}
			else
			{
				tp_current.host_id_state = TP_HOST_ID_OVERLONG;
			}
		}
	}
	if (tp_current_instance)
	{
		tp_current.instance = 1;
		tp_current.model = tp_current_instance->object_number;
		tp_current.vp_known = 1;
		copy_vp (tp_current.vp_before, tp_current_instance);
	}
	else
	{
		/* no instance: the position fallback unless the fixed-object path below finds a valid shape */
		tp_current.branch = 4;
	}
}

#define OPEN_OR_RETURN \
	if (!tp_open) \
	{ \
		tp_orphans++; \
		return; \
	}

void eech_tp_instance_target_point (void)
{
	OPEN_OR_RETURN
	tp_current.branch = 1;
}

void eech_tp_instance_position (void)
{
	OPEN_OR_RETURN
	tp_current.branch = 2;
}

/* branches 2, 3 and 4: the original's get_local_entity_vec3d has just copied the entity position into the result; nothing physical has been added yet */
void eech_tp_base_position (const void *target_point)
{
	const vec3d *point = (const vec3d *) target_point;

	OPEN_OR_RETURN
	copy_vec (tp_current.base, &point->x, &point->y, &point->z);
	tp_current.base_known = 1;
}

/* branch 1, right before the original calls get_3d_sub_object_world_viewpoint: the placements it is about to walk, read through the same parent pointers from the sub-object the search found */
void eech_tp_instance_chain (const void *result_sub_object, const void *instance)
{
	const object_3d_sub_instance *node = (const object_3d_sub_instance *) result_sub_object;
	const object_3d_instance *main_object = (const object_3d_instance *) instance;
	const object_3d_sub_instance *leaf_first[TP_CHAIN];
	int depth = 0, i;

	OPEN_OR_RETURN
	if (main_object)
	{
		copy_vec (tp_current.main_scale, &main_object->relative_scale.x, &main_object->relative_scale.y, &main_object->relative_scale.z);
		tp_current.scale_known = 1;
	}
	if (!node)
	{
		tp_current.chain_state = TP_CHAIN_NULL_RESULT;
		return;
	}
	while (node->parent)
	{
		if (depth == TP_WALK_MAX)
		{
			tp_current.chain_state = TP_CHAIN_UNTERMINATED;
			tp_current.chain_depth = depth;
			return;
		}
		if (depth < TP_CHAIN)
		{
			leaf_first[depth] = node;
		}
		depth++;
		node = node->parent;
	}
	tp_current.chain_depth = depth;
	if ((const void *) node != instance)
	{
		tp_current.chain_state = TP_CHAIN_WRONG_ROOT;
		return;
	}
	if (depth == 0)
	{
		tp_current.chain_state = TP_CHAIN_EMPTY;
		return;
	}
	if (depth > TP_CHAIN)
	{
		tp_current.chain_state = TP_CHAIN_TOO_DEEP;
		return;
	}
	for (i = 0; i < depth; i++)
	{
		const object_3d_sub_instance *step = leaf_first[depth - 1 - i];
		uint32_t *out = tp_current.chain[i];

		copy_vec (out, &step->relative_position.x, &step->relative_position.y, &step->relative_position.z);
		copy_vec (out + 3, &step->relative_heading, &step->relative_pitch, &step->relative_roll);
		copy_vec (out + 6, &step->relative_scale.x, &step->relative_scale.y, &step->relative_scale.z);
	}
	tp_current.chain_state = TP_CHAIN_COMPLETE;
}

void eech_tp_fixed (void)
{
	OPEN_OR_RETURN
	tp_current.fixed = 1;
}

void eech_tp_shape (int index)
{
	OPEN_OR_RETURN
	tp_current.shape = index;
	if (index == OBJECT_3D_INVALID_OBJECT_INDEX)
	{
		return;
	}
	tp_current.branch = 3;
	if (index > OBJECT_3D_INVALID_OBJECT_INDEX && index < OBJECT_3D_LAST)
	{
		tp_current.model = index;
		tp_current.offset_known = 1;
		copy_vec (tp_current.offset, &target_point_offsets[index].x, &target_point_offsets[index].y, &target_point_offsets[index].z);
	}
	else
	{
		/* the original indexes its table with this value; nothing is read here */
		tp_current.shape_out_of_range = 1;
	}
}

void eech_tp_horizontal (void)
{
	OPEN_OR_RETURN
	tp_current.horizontal = 1;
}

void eech_tp_heading (const float *heading)
{
	OPEN_OR_RETURN
	tp_current.heading_read = 1;
	tp_current.heading = bits_of (heading);
}

void eech_tp_xz_applied (void)
{
	OPEN_OR_RETURN
	tp_current.xz_applied = 1;
}

void eech_tp_end (const void *target_point)
{
	const vec3d *point = (const vec3d *) target_point;
	int site, kind;

	OPEN_OR_RETURN
	copy_vec (tp_current.result, &point->x, &point->y, &point->z);
	if (tp_current_instance)
	{
		copy_vp (tp_current.vp_after, tp_current_instance);
		tp_current.vp_changed = memcmp (tp_current.vp_before, tp_current.vp_after, sizeof (tp_current.vp_before)) != 0;
	}
	tp_open = 0;
	site = site_class (tp_current.site, tp_current.detail);
	kind = case_of (&tp_current);
	if (kind < 0)
	{
		tp_unclassified++;
	}
	else
	{
		tp_matrix[site][tp_current.branch]++;
		tp_case_count[kind]++;
	}
	if (tp_current.vp_changed)
	{
		tp_vp_changed_count++;
	}
	if (tp_current.shape_out_of_range)
	{
		tp_out_of_range++;
	}
	if (tp_current.host_id_state == TP_HOST_ID_PRESENT)
	{
		tp_host_present++;
	}
	else if (tp_current.host_id_state == TP_HOST_ID_ABSENT)
	{
		tp_host_absent++;
	}
	else
	{
		tp_overlong++;
	}
	if (tp_event_count < TP_EVENTS)
	{
		tp_events[tp_event_count] = tp_current;
		tp_event_count++;
	}
	else
	{
		tp_dropped++;
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the census hooks */

static void census_record (int index, int state, const void *offset, const object_3d_bounds *bounds)
{
	const vec3d *point = (const vec3d *) offset;
	struct tp_generation *generation;
	struct tp_census_row *row;

	if (!tp_gen_open)
	{
		tp_census_stray++;
		return;
	}
	if (tp_gen_current < 0)
	{
		/* a pass beyond the bounded history: counted once by its begin, recorded nowhere */
		return;
	}
	generation = &tp_generations[tp_gen_current];
	if (index < OBJECT_3D_INVALID_OBJECT_INDEX || index >= OBJECT_3D_LAST)
	{
		generation->out_of_range++;
		return;
	}
	row = &generation->rows[index];
	if (row->state != 0)
	{
		/* a second entry for one index WITHIN this pass; the last is kept, the error is counted */
		generation->duplicates++;
	}
	memset (row, 0, sizeof (*row));
	row->state = state;
	copy_vec (row->offset, &point->x, &point->y, &point->z);
	if (bounds)
	{
		row->bounds[0] = bits_of (&bounds->xmin);
		row->bounds[1] = bits_of (&bounds->xmax);
		row->bounds[2] = bits_of (&bounds->ymin);
		row->bounds[3] = bits_of (&bounds->ymax);
		row->bounds[4] = bits_of (&bounds->zmin);
		row->bounds[5] = bits_of (&bounds->zmax);
	}
}

void eech_tp_census_begin (void)
{
	if (tp_gen_open)
	{
		/* the previous pass never ended */
		tp_gen_interrupted++;
		if (tp_gen_current >= 0)
		{
			tp_generations[tp_gen_current].interrupted = 1;
		}
	}
	tp_gen_open = 1;
	if (tp_gen_count < TP_GENERATIONS)
	{
		tp_gen_current = tp_gen_count;
		tp_gen_count++;
	}
	else
	{
		tp_gen_current = -1;
		tp_gen_dropped++;
	}
	tp_rng_pass_begin_draws = tp_rng_draws;
	rng_mark (TP_MARK_PASS_BEGIN, tp_gen_current, 0);
}

void eech_tp_census_invalid (const void *offset)
{
	census_record (OBJECT_3D_INVALID_OBJECT_INDEX, 3, offset, NULL);
}

void eech_tp_census_sub_object (int index, const void *offset)
{
	census_record (index, 1, offset, NULL);
}

void eech_tp_census_bbox (int index, const void *offset, const void *bounds)
{
	census_record (index, 2, offset, (const object_3d_bounds *) bounds);
}

void eech_tp_census_end (void)
{
	if (!tp_gen_open)
	{
		tp_census_stray++;
		return;
	}
	if (tp_gen_current >= 0)
	{
		tp_generations[tp_gen_current].rng_draws = tp_rng_draws - tp_rng_pass_begin_draws;
		tp_generations[tp_gen_current].skinned = skinned_scenes ();
	}
	rng_mark (TP_MARK_PASS_END, tp_gen_current, 0);
	tp_rng_after_pass = tp_gen_current;
	tp_gen_open = 0;
	if (tp_gen_current >= 0)
	{
		tp_generations[tp_gen_current].ended = 1;
		tp_gen_last = tp_gen_current;
	}
	tp_gen_current = -1;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* reports: `key=value` text lines; a report that does not fit is refused, never cut short */

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

static void out_start (struct out *out, char *text, int capacity)
{
	out->text = text;
	out->capacity = capacity;
	out->used = 0;
	out->overflow = 0;
}

/* a table name as one token of [A-Za-z0-9_.-]; "-" when there is none */
static void clean_name (char *dst, size_t capacity, const char *name)
{
	size_t i = 0;

	if (name)
	{
		for (; name[i] && i + 1 < capacity; i++)
		{
			unsigned char c = (unsigned char) name[i];
			dst[i] = (isalnum (c) || c == '_' || c == '.' || c == '-') ? (char) c : '_';
		}
	}
	if (i == 0)
	{
		dst[i++] = '-';
	}
	dst[i] = '\0';
}

static const char *scene_name (int scene)
{
	if (scene > OBJECT_3D_INVALID_OBJECT_INDEX && scene < OBJECT_3D_LAST && object_3d_scene_names[scene])
	{
		return object_3d_scene_names[scene];
	}
	return NULL;
}

static const char *entity_type_name (int type)
{
	if (type >= 0 && type < NUM_ENTITY_TYPES)
	{
		return entity_type_database[type].name;
	}
	return NULL;
}

/* `count` words as lower-case 8-digit hex separated by commas; the buffer holds at most 12 */
static void words_text (char *dst, size_t capacity, const uint32_t *words, int count)
{
	int i;
	size_t used = 0;

	dst[0] = '\0';
	for (i = 0; i < count; i++)
	{
		int n = snprintf (dst + used, capacity - used, "%s%08x", i ? "," : "", (unsigned int) words[i]);
		if (n < 0 || (size_t) n >= capacity - used)
		{
			return;
		}
		used += (size_t) n;
	}
}

/* does the CURRENT table still hold the words this pass recorded for the index? (a historical pass may legitimately say no) */
static int census_live (const struct tp_generation *generation, int index)
{
	const struct tp_census_row *row = &generation->rows[index];

	return row->offset[0] == bits_of (&target_point_offsets[index].x) && row->offset[1] == bits_of (&target_point_offsets[index].y) &&
		row->offset[2] == bits_of (&target_point_offsets[index].z);
}

static const char *census_state_name (int state)
{
	switch (state)
	{
		case 1:
		{
			return "sub_object";
		}
		case 2:
		{
			return "bbox";
		}
		case 3:
		{
			return "invalid_zero";
		}
		default:
		{
			return "none";
		}
	}
}

struct census_totals
{
	int recorded, sub_object, bbox, invalid_zero, live_mismatch, complete;
};

static void census_totals_of (const struct tp_generation *generation, struct census_totals *totals)
{
	int i;

	memset (totals, 0, sizeof (*totals));
	for (i = 0; i < OBJECT_3D_LAST; i++)
	{
		int state = generation->rows[i].state;

		if (state == 0)
		{
			continue;
		}
		totals->recorded++;
		if (state == 1)
		{
			totals->sub_object++;
		}
		else if (state == 2)
		{
			totals->bbox++;
		}
		else
		{
			totals->invalid_zero++;
		}
		if (!census_live (generation, i))
		{
			totals->live_mismatch++;
		}
	}
	/* a pass is a COMPLETE snapshot if it ended, was not abandoned, recorded every index exactly once, inside the table, with the invalid index's zero once; whether it still equals the
	   current table is a separate fact (live_mismatch) */
	totals->complete = generation->ended && !generation->interrupted && totals->recorded == (int) OBJECT_3D_LAST && generation->duplicates == 0 && generation->out_of_range == 0 &&
		totals->invalid_zero == 1;
}

int eech_tp_summary_run (char *text, int capacity)
{
	struct out out;
	struct census_totals census;
	int i, b, stale, census_whole, queries_whole, all_complete;

	out_start (&out, text, capacity);
	stale = tp_stale + (tp_pending ? 1 : 0);
	emit (&out, "tp-summary calls=%d events=%d event_capacity=%d dropped=%d stale=%d interrupted=%d orphans=%d unclassified=%d open=%d out_of_range=%d host_id_overlong=%d\n",
		tp_calls, tp_event_count, TP_EVENTS, tp_dropped, stale, tp_interrupted, tp_orphans, tp_unclassified, tp_open ? 1 : 0, tp_out_of_range, tp_overlong);
	for (b = 1; b <= 4; b++)
	{
		int total = 0;

		for (i = 0; i < TP_SITES; i++)
		{
			total += tp_matrix[i][b];
		}
		emit (&out, "tp-branch branch=%d name=%s calls=%d\n", b, branch_name (b), total);
	}
	for (i = 0; i < TP_SITES; i++)
	{
		emit (&out, "tp-site site=%d name=%s b1=%d b2=%d b3=%d b4=%d\n", i, site_name (i), tp_matrix[i][1], tp_matrix[i][2], tp_matrix[i][3], tp_matrix[i][4]);
	}
	emit (&out, "tp-case target_point=%d instance_position=%d no_horizontal=%d heading_zero=%d rotated=%d not_fixed=%d invalid_shape=%d vp_changed=%d host_id_present=%d host_id_absent=%d\n",
		tp_case_count[TP_CASE_TARGET_POINT], tp_case_count[TP_CASE_INSTANCE_POSITION], tp_case_count[TP_CASE_NO_HORIZONTAL], tp_case_count[TP_CASE_HEADING_ZERO],
		tp_case_count[TP_CASE_ROTATED], tp_case_count[TP_CASE_NOT_FIXED], tp_case_count[TP_CASE_INVALID_SHAPE], tp_vp_changed_count, tp_host_present, tp_host_absent);
	emit (&out, "tp-census expected=%d generations=%d capacity=%d dropped=%d open=%d interrupted=%d stray=%d last=%d\n", (int) OBJECT_3D_LAST, tp_gen_count, TP_GENERATIONS, tp_gen_dropped,
		tp_gen_open ? 1 : 0, tp_gen_interrupted, tp_census_stray, tp_gen_last);
	all_complete = 1;
	for (i = 0; i < tp_gen_count; i++)
	{
		census_totals_of (&tp_generations[i], &census);
		emit (&out, "tp-pass ordinal=%d ended=%d interrupted=%d recorded=%d sub_object=%d bbox=%d invalid_zero=%d duplicates=%d out_of_range=%d live_mismatch=%d complete=%d rng_draws=%u skinned=%d\n", i,
			tp_generations[i].ended, tp_generations[i].interrupted, census.recorded, census.sub_object, census.bbox, census.invalid_zero, tp_generations[i].duplicates,
			tp_generations[i].out_of_range, census.live_mismatch, census.complete, tp_generations[i].rng_draws, tp_generations[i].skinned);
		if (!census.complete)
		{
			all_complete = 0;
		}
	}
	emit (&out, "tp-rng draws=%u seeds=%u pass_draws=%u outside_draws=%u exposed=%u marks=%d mark_capacity=%d marks_dropped=%u\n", tp_rng_draws, tp_rng_seeds, tp_rng_pass_draws, tp_rng_outside,
		tp_rng_exposed, tp_rng_mark_count, TP_RNG_MARKS, tp_rng_marks_dropped);
	for (i = 0; i < tp_rng_mark_count; i++)
	{
		char value[12];

		if (tp_rng_marks[i].kind == TP_MARK_SEED)
		{
			snprintf (value, sizeof (value), "%08x", (unsigned int) tp_rng_marks[i].value);
		}
		else
		{
			strcpy (value, "-");
		}
		emit (&out, "tp-rng-mark seq=%d kind=%s gen=%d value=%s draws=%u since_seed=%u\n", i, mark_name (tp_rng_marks[i].kind), tp_rng_marks[i].gen, value, tp_rng_marks[i].draws,
			tp_rng_marks[i].since_seed);
	}
	/* the census is whole only if at least one pass ran, no pass was lost, abandoned or left open, no hook strayed, every kept pass is a complete snapshot, and the LATEST pass is the one the
	   table holds now (an older pass may differ: it is a fact about that pass, not a lost capture). A query capture is whole only if no call was lost, interrupted, unclassified or out of
	   range. A capture with NO call is whole but measured nothing: the caller reads `calls` */
	census_whole = tp_gen_count >= 1 && tp_gen_dropped == 0 && !tp_gen_open && tp_gen_interrupted == 0 && tp_census_stray == 0 && all_complete && tp_gen_last == tp_gen_count - 1;
	if (census_whole)
	{
		census_totals_of (&tp_generations[tp_gen_last], &census);
		census_whole = census.live_mismatch == 0;
	}
	queries_whole = tp_dropped == 0 && stale == 0 && tp_interrupted == 0 && tp_orphans == 0 && tp_unclassified == 0 && !tp_open && tp_out_of_range == 0 && tp_overlong == 0;
	emit (&out, "tp-complete census=%d queries_whole=%d\n", census_whole ? 1 : 0, queries_whole ? 1 : 0);
	return out.overflow ? 1 : 0;
}

int eech_tp_offsets_run (int generation_index, char *text, int capacity)
{
	struct out out;
	const struct tp_generation *generation;
	int i;

	out_start (&out, text, capacity);
	if (generation_index < 0 || generation_index >= tp_gen_count)
	{
		return 1;
	}
	generation = &tp_generations[generation_index];
	for (i = 0; i < OBJECT_3D_LAST; i++)
	{
		const struct tp_census_row *row = &generation->rows[i];
		char name[96], offset[40], bounds[64];

		clean_name (name, sizeof (name), scene_name (i));
		if (row->state == 0)
		{
			emit (&out, "tp-offset generation=%d index=%d name=%s state=none offset=- bounds=- live=-\n", generation_index, i, name);
			continue;
		}
		words_text (offset, sizeof (offset), row->offset, 3);
		if (row->state == 2)
		{
			words_text (bounds, sizeof (bounds), row->bounds, 6);
		}
		else
		{
			strcpy (bounds, "-");
		}
		emit (&out, "tp-offset generation=%d index=%d name=%s state=%s offset=%s bounds=%s live=%d\n", generation_index, i, name, census_state_name (row->state), offset, bounds,
			census_live (generation, i) ? 1 : 0);
	}
	return out.overflow ? 1 : 0;
}

int eech_tp_event_run (int index, char *text, int capacity)
{
	struct out out;
	const struct tp_event *e;
	char type_name[96], model_name[96], heading[12], offset[40], before[128], after[128], base[40], scale[40], chain[TP_CHAIN * 9 * 9 + 8];
	int kind, i;

	out_start (&out, text, capacity);
	if (index < 0 || index >= tp_event_count)
	{
		return 1;
	}
	e = &tp_events[index];
	kind = case_of (e);
	clean_name (type_name, sizeof (type_name), entity_type_name (e->entity_type));
	clean_name (model_name, sizeof (model_name), e->model >= 0 ? scene_name (e->model) : NULL);
	if (e->heading_read)
	{
		snprintf (heading, sizeof (heading), "%08x", (unsigned int) e->heading);
	}
	else
	{
		strcpy (heading, "-");
	}
	if (e->offset_known)
	{
		words_text (offset, sizeof (offset), e->offset, 3);
	}
	else
	{
		strcpy (offset, "-");
	}
	if (e->vp_known)
	{
		words_text (before, sizeof (before), e->vp_before, 12);
		words_text (after, sizeof (after), e->vp_after, 12);
	}
	else
	{
		strcpy (before, "-");
		strcpy (after, "-");
	}
	if (e->base_known)
	{
		words_text (base, sizeof (base), e->base, 3);
	}
	else
	{
		strcpy (base, "-");
	}
	if (e->scale_known)
	{
		words_text (scale, sizeof (scale), e->main_scale, 3);
	}
	else
	{
		strcpy (scale, "-");
	}
	/* the chain: nodes of nine words, topmost first, joined by `;`; only a complete chain carries them */
	strcpy (chain, "-");
	if (e->chain_state == TP_CHAIN_COMPLETE)
	{
		size_t used = 0;

		chain[0] = '\0';
		for (i = 0; i < e->chain_depth && i < TP_CHAIN; i++)
		{
			char node[96];

			words_text (node, sizeof (node), e->chain[i], 9);
			used += (size_t) snprintf (chain + used, sizeof (chain) - used, "%s%s", i ? ";" : "", node);
		}
	}
	emit (&out, "tp-event index=%d seq=%d gen=%d gen_open=%d site=%d site_name=%s detail=%d branch=%d branch_name=%s case=%s entity_type=%d entity_type_name=%s instance=%d model=%d model_name=%s "
		"fixed=%d shape=%d horizontal=%d heading=%s xz_applied=%d offset=%s result=%08x,%08x,%08x vp_before=%s vp_after=%s base=%s main_scale=%s chain_state=%s chain_depth=%d chain=%s vp_changed=%d host_id_state=%s host_id_len=%d",
		index, e->seq, e->gen_last, e->gen_open, site_class (e->site, e->detail), site_name (site_class (e->site, e->detail)), e->detail, e->branch, branch_name (e->branch), case_name (kind), e->entity_type,
		type_name, e->instance, e->model, model_name, e->fixed, e->shape, e->horizontal, heading, e->xz_applied, offset, (unsigned int) e->result[0], (unsigned int) e->result[1],
		(unsigned int) e->result[2], before, after, base, scale, chain_state_name (e->chain_state), e->chain_depth, chain, e->vp_changed, e->host_id_state == TP_HOST_ID_PRESENT ? "present" : (e->host_id_state == TP_HOST_ID_ABSENT ? "absent" : "overlong"),
		e->host_id_len);
	if (e->host_id_state == TP_HOST_ID_PRESENT)
	{
		/* the identity is the LAST field and runs to the end of the line (its length is declared above): the host's text is printable but may hold spaces */
		emit (&out, " host_id=%s", e->host_id);
	}
	emit (&out, "\n");
	return out.overflow ? 1 : 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the controlled probe */

#define TP_PROBE_BUCKETS 6
#define TP_PROBE_PICKS 2

static int tp_probe_done;

static const char *probe_bucket_name (int bucket)
{
	switch (bucket)
	{
		case 1:
		{
			return "instance_without_target_point";
		}
		case 2:
		{
			return "fixed_no_horizontal";
		}
		case 3:
		{
			return "fixed_rotated";
		}
		case 4:
		{
			return "fixed_heading_zero";
		}
		case 5:
		{
			return "not_fixed_no_instance";
		}
		case 6:
		{
			return "fixed_invalid_shape";
		}
		default:
		{
			return "-";
		}
	}
}

/* the families a target can be: the mobiles that answer the instance pointer, and the structures */
static int probe_type_considered (int type)
{
	switch (type)
	{
		case ENTITY_TYPE_ANTI_AIRCRAFT:
		case ENTITY_TYPE_FIXED_WING:
		case ENTITY_TYPE_HELICOPTER:
		case ENTITY_TYPE_PERSON:
		case ENTITY_TYPE_ROUTED_VEHICLE:
		case ENTITY_TYPE_SHIP_VEHICLE:
		{
			return 1;
		}
		default:
		{
			return campaign_object_is_structure (type) ? 1 : 0;
		}
	}
}

/* Which unreached case the original takes for this entity: 0 none to probe (an instance WITH a TARGET_POINT sub-object is branch 1, already measured, and the only branch that writes), 1 an
   instance without one, 2 a fixed object whose shape has no horizontal offset, 3 one with a horizontal offset and a non-zero heading, 4 one with a horizontal offset and a zero heading (the
   legacy quirk), 5 not a fixed object and no instance, 6 a fixed object with the invalid shape; -1 a fixed object whose shape is outside the table (the original would index outside it). Only
   reads the original itself makes, and no floating-point arithmetic: a float is zero if its magnitude bits are. */
static int probe_bucket (entity *en)
{
	object_3d_instance *instance;
	object_3d_sub_object_search_data search;
	object_3d_index_numbers index;

	instance = (object_3d_instance *) get_local_entity_ptr_value (en, PTR_TYPE_INSTANCE_3D_OBJECT);
	if (instance)
	{
		search.search_depth = 0;
		search.search_object = instance;
		search.sub_object_index = OBJECT_3D_SUB_OBJECT_TARGET_POINT;
		return find_object_3d_sub_object (&search) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND ? 0 : 1;
	}
	if (!get_local_entity_int_value (en, INT_TYPE_IDENTIFY_FIXED))
	{
		return 5;
	}
	index = get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);
	if (index == OBJECT_3D_INVALID_OBJECT_INDEX)
	{
		return 6;
	}
	if ((int) index < OBJECT_3D_INVALID_OBJECT_INDEX || (int) index >= OBJECT_3D_LAST)
	{
		return -1;
	}
	if ((bits_of (&target_point_offsets[index].x) & 0x7fffffffu) == 0 && (bits_of (&target_point_offsets[index].z) & 0x7fffffffu) == 0)
	{
		return 2;
	}
	{
		float heading = get_local_entity_float_value (en, FLOAT_TYPE_HEADING);

		return (bits_of (&heading) & 0x7fffffffu) != 0 ? 3 : 4;
	}
}

/* A used-entity list node is trusted only if it is an aligned slot of the entity heap this run allocated; compared as integers, never as unrelated pointers. */
static int probe_heap_slot (const entity *en)
{
	uintptr_t base = (uintptr_t) entities;
	uintptr_t at = (uintptr_t) en;

	if (!entities || number_of_entities <= 0)
	{
		return 0;
	}
	if (at < base || at >= base + ((uintptr_t) number_of_entities * sizeof (entity)))
	{
		return 0;
	}
	return ((at - base) % sizeof (entity)) == 0;
}

/* A refusal: nothing was called, the once flag is untouched, and the reason is the text (the engine entry turns the non-zero return into a bad argument that keeps it). */
static int probe_refused (char *text, int capacity, const char *reason, int visited)
{
	snprintf (text, (size_t) capacity, "tp-probe-refused reason=%s visited=%d\n", reason, visited);
	return 1;
}

int eech_tp_probe_run (char *text, int capacity)
{
	struct out out;
	entity *picks[TP_PROBE_BUCKETS + 1][TP_PROBE_PICKS];
	int candidates[TP_PROBE_BUCKETS + 1], picked[TP_PROBE_BUCKETS + 1];
	int visited = 0, examined = 0, out_of_range = 0, b, k;
	entity *en;

	if (capacity <= 0)
	{
		return 1;
	}
	text[0] = '\0';
	/* once, and only between queries and passes */
	if (tp_probe_done)
	{
		return probe_refused (text, capacity, "already_run", 0);
	}
	if (tp_open || tp_gen_open || tp_pending)
	{
		return probe_refused (text, capacity, "a_query_or_pass_is_running", 0);
	}
	memset (candidates, 0, sizeof (candidates));
	memset (picked, 0, sizeof (picked));
	/* THE SCAN. It visits at most as many nodes as the heap has slots, validates each node as a heap slot BEFORE it reads the node, and touches no original function. A list that is still
	   going at the bound (a cycle), or leaves the heap, makes the whole scan INCOMPLETE: the probe is refused, nothing is called and no case is ever called unavailable from a partial scan. */
	for (en = get_local_entity_list (); en;)
	{
		int bucket;

		if (visited >= number_of_entities)
		{
			return probe_refused (text, capacity, "list_longer_than_heap", visited);
		}
		if (!probe_heap_slot (en))
		{
			return probe_refused (text, capacity, "list_node_not_a_heap_slot", visited);
		}
		visited++;
		if (probe_type_considered ((int) en->type))
		{
			/* an eligible node must carry live entity data BEFORE any typed getter dereferences it (a valid heap slot and a type do not establish that): refused, never skipped or called unavailable */
			if (!get_local_entity_data (en))
			{
				return probe_refused (text, capacity, "eligible_node_without_entity_data", visited);
			}
			examined++;
			bucket = probe_bucket (en);
			if (bucket < 0)
			{
				out_of_range++;
			}
			else if (bucket > 0)
			{
				candidates[bucket]++;
				if (picked[bucket] < TP_PROBE_PICKS)
				{
					picks[bucket][picked[bucket]] = en;
					picked[bucket]++;
				}
			}
		}
		en = get_local_entity_succ (en);
	}
	/* THE REPORT, staged BEFORE any original is called: its size depends on the scan only, so a buffer that cannot hold it refuses here with nothing consumed */
	out_start (&out, text, capacity);
	emit (&out, "tp-probe status=complete visited=%d entities=%d shape_out_of_range=%d\n", visited, examined, out_of_range);
	for (b = 1; b <= TP_PROBE_BUCKETS; b++)
	{
		emit (&out, "tp-probe-bucket bucket=%d name=%s candidates=%d picked=%d available=%d\n", b, probe_bucket_name (b), candidates[b], picked[b], picked[b] > 0 ? 1 : 0);
	}
	if (out.overflow)
	{
		return probe_refused (text, capacity, "report_capacity", visited);
	}
	/* the scan is complete and the report fits: only now is the once flag consumed and the original called, at most TP_PROBE_PICKS times per case */
	tp_probe_done = 1;
	for (b = 1; b <= TP_PROBE_BUCKETS; b++)
	{
		for (k = 0; k < picked[b]; k++)
		{
			vec3d point;

			/* the ORIGINAL, once, as the controlled caller with the case as its detail; the hooks record it like any other query */
			eech_tp_caller (3, b);
			get_local_entity_target_point (picks[b][k], &point);
		}
	}
	return 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the setup probe */

#define TP_SETUP_CASES 3
/* the report holds at most three bounded rows: a buffer below this refuses BEFORE anything is changed */
#define TP_SETUP_MIN_CAPACITY 4096

extern int object_3d_scenes_creation_count[];

static int tp_setup_done;

static const char *setup_case_name (int which)
{
	switch (which)
	{
		case 1:
		{
			return "instance_without_target_point";
		}
		case 2:
		{
			return "not_fixed_no_instance";
		}
		case 3:
		{
			return "fixed_invalid_shape";
		}
		default:
		{
			return "-";
		}
	}
}

/* the field the typed instance getter reads, for the families that have one and that the setup touches (there is no typed setter for it) */
static object_3d_instance **setup_instance_field (entity *en)
{
	void *data = get_local_entity_data (en);

	switch ((int) en->type)
	{
		case ENTITY_TYPE_FIXED_WING:
		case ENTITY_TYPE_HELICOPTER:
		{
			return &((aircraft *) data)->inst3d;
		}
		case ENTITY_TYPE_ANTI_AIRCRAFT:
		case ENTITY_TYPE_ROUTED_VEHICLE:
		case ENTITY_TYPE_SHIP_VEHICLE:
		{
			return &((vehicle *) data)->inst3d;
		}
		default:
		{
			return NULL;
		}
	}
}

struct tp_setup_row
{
	int available, applied, called, restored;
	int entity_type, host_id_state, from, to, vp_unchanged, counters_restored;
	const char *reason;
};

static void setup_identity (struct tp_setup_row *row, entity *en)
{
	const char *id = campaign_object_host_id (en);

	row->entity_type = (int) en->type;
	row->host_id_state = (id && id[0]) ? 1 : 0;
}

/* case 1: the instance pointer of a real mobile is a temporarily constructed instance of a bounding-box scene (no TARGET_POINT) */
static void setup_instance_case (struct tp_setup_row *row, entity *en, int scene)
{
	object_3d_instance **field = setup_instance_field (en);
	object_3d_instance *original = *field;
	object_3d_instance *temporary;
	object_3d_sub_object_search_data search;
	uint32_t before[12], after[12];
	vec3d point;
	int constructed = current_number_of_3d_objects_constructed;
	int memory = current_amount_of_3d_objects_memory;
	int created = object_3d_scenes_creation_count[scene];

	setup_identity (row, en);
	row->from = original->object_number;
	row->to = scene;
	copy_vp (before, original);
	temporary = construct_3d_object ((object_3d_index_numbers) scene);
	search.search_depth = 0;
	search.search_object = temporary;
	search.sub_object_index = OBJECT_3D_SUB_OBJECT_TARGET_POINT;
	if (find_object_3d_sub_object (&search) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND)
	{
		/* the census said bounding box and a fresh instance says otherwise: nothing was changed on the entity */
		destruct_3d_object (temporary);
		row->reason = "scene_has_target_point";
		row->restored = 1;
		row->vp_unchanged = 1;
		row->counters_restored = constructed == current_number_of_3d_objects_constructed && memory == current_amount_of_3d_objects_memory && created == object_3d_scenes_creation_count[scene];
		return;
	}
	*field = temporary;
	row->applied = get_local_entity_ptr_value (en, PTR_TYPE_INSTANCE_3D_OBJECT) == (void *) temporary;
	if (row->applied)
	{
		eech_tp_caller (4, 1);
		get_local_entity_target_point (en, &point);
		row->called = 1;
	}
	else
	{
		row->reason = "setup_not_applied";
	}
	*field = original;
	destruct_3d_object (temporary);
	copy_vp (after, original);
	row->restored = get_local_entity_ptr_value (en, PTR_TYPE_INSTANCE_3D_OBJECT) == (void *) original;
	row->vp_unchanged = memcmp (before, after, sizeof (before)) == 0;
	row->counters_restored = constructed == current_number_of_3d_objects_constructed && memory == current_amount_of_3d_objects_memory &&
		created == object_3d_scenes_creation_count[scene];
}

/* case 2: the instance pointer of a real mobile is NULL */
static void setup_no_instance_case (struct tp_setup_row *row, entity *en)
{
	object_3d_instance **field = setup_instance_field (en);
	object_3d_instance *original = *field;
	uint32_t before[12], after[12];
	vec3d point;

	setup_identity (row, en);
	row->from = original->object_number;
	row->to = -1;
	row->counters_restored = -1;
	copy_vp (before, original);
	*field = NULL;
	row->applied = get_local_entity_ptr_value (en, PTR_TYPE_INSTANCE_3D_OBJECT) == NULL;
	if (row->applied)
	{
		eech_tp_caller (4, 2);
		get_local_entity_target_point (en, &point);
		row->called = 1;
	}
	else
	{
		row->reason = "setup_not_applied";
	}
	*field = original;
	copy_vp (after, original);
	row->restored = get_local_entity_ptr_value (en, PTR_TYPE_INSTANCE_3D_OBJECT) == (void *) original;
	row->vp_unchanged = memcmp (before, after, sizeof (before)) == 0;
}

/* case 3: the shape of a real fixed structure is the invalid index, through its typed setter */
static void setup_invalid_shape_case (struct tp_setup_row *row, entity *en)
{
	int original = (int) get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);
	vec3d point;

	setup_identity (row, en);
	row->from = original;
	row->to = OBJECT_3D_INVALID_OBJECT_INDEX;
	row->vp_unchanged = -1;
	row->counters_restored = -1;
	set_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE, OBJECT_3D_INVALID_OBJECT_INDEX);
	row->applied = get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE) == OBJECT_3D_INVALID_OBJECT_INDEX;
	if (row->applied)
	{
		eech_tp_caller (4, 3);
		get_local_entity_target_point (en, &point);
		row->called = 1;
	}
	else
	{
		row->reason = "setup_not_applied";
	}
	set_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE, original);
	row->restored = (int) get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE) == original;
}

static void setup_emit_row (struct out *out, int which, const struct tp_setup_row *row)
{
	char type[96], from_name[96], to_name[96];

	clean_name (type, sizeof (type), row->available ? entity_type_name (row->entity_type) : NULL);
	clean_name (from_name, sizeof (from_name), row->available && row->from > 0 ? scene_name (row->from) : NULL);
	clean_name (to_name, sizeof (to_name), row->available && row->to > 0 ? scene_name (row->to) : NULL);
	emit (out, "tp-setup-case case=%d name=%s available=%d reason=%s applied=%d called=%d restored=%d entity_type=%s host_id_state=%s from=%d from_name=%s to=%d to_name=%s "
		"vp_unchanged=%d counters_restored=%d\n", which, setup_case_name (which), row->available, row->reason, row->applied, row->called, row->restored, type,
		row->available ? (row->host_id_state ? "present" : "absent") : "-", row->available ? row->from : -1, from_name, row->available ? row->to : -1, to_name,
		row->available ? row->vp_unchanged : -1, row->available ? row->counters_restored : -1);
}

int eech_tp_setup_run (char *text, int capacity)
{
	struct out out;
	struct tp_setup_row rows[TP_SETUP_CASES + 1];
	entity *mobile[2];
	entity *structure = NULL;
	int mobiles = 0, visited = 0, examined = 0, scene = -1, i, which;
	entity *en;

	if (capacity <= 0)
	{
		return 1;
	}
	text[0] = '\0';
	if (tp_setup_done)
	{
		return probe_refused (text, capacity, "already_run", 0);
	}
	if (tp_open || tp_gen_open || tp_pending)
	{
		return probe_refused (text, capacity, "a_query_or_pass_is_running", 0);
	}
	/* the report is bounded, so its capacity is checked before anything is changed */
	if (capacity < TP_SETUP_MIN_CAPACITY)
	{
		return probe_refused (text, capacity, "report_capacity", 0);
	}
	memset (rows, 0, sizeof (rows));
	for (which = 1; which <= TP_SETUP_CASES; which++)
	{
		rows[which].reason = "-";
		rows[which].from = -1;
		rows[which].to = -1;
		rows[which].vp_unchanged = -1;
		rows[which].counters_restored = -1;
	}
	/* THE SCAN, as the read-only probe's: bounded by the heap's slot count, every node validated as an aligned heap slot and every eligible node required to carry live data BEFORE any typed
	   getter; an incomplete scan refuses the whole setup and changes nothing. Only reads here. */
	for (en = get_local_entity_list (); en;)
	{
		if (visited >= number_of_entities)
		{
			return probe_refused (text, capacity, "list_longer_than_heap", visited);
		}
		if (!probe_heap_slot (en))
		{
			return probe_refused (text, capacity, "list_node_not_a_heap_slot", visited);
		}
		visited++;
		if (probe_type_considered ((int) en->type))
		{
			if (!get_local_entity_data (en))
			{
				return probe_refused (text, capacity, "eligible_node_without_entity_data", visited);
			}
			examined++;
			if (setup_instance_field (en))
			{
				if (mobiles < 2 && get_local_entity_ptr_value (en, PTR_TYPE_INSTANCE_3D_OBJECT))
				{
					mobile[mobiles] = en;
					mobiles++;
				}
			}
			else if (!structure && campaign_object_is_structure ((int) en->type) && get_local_entity_int_value (en, INT_TYPE_IDENTIFY_FIXED))
			{
				int shape = (int) get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);

				if (shape > OBJECT_3D_INVALID_OBJECT_INDEX && shape < OBJECT_3D_LAST)
				{
					structure = en;
				}
			}
		}
		en = get_local_entity_succ (en);
	}
	/* a real census bounding-box scene (the latest finished pass) that needs no random skin, so constructing it draws nothing */
	if (tp_gen_last >= 0)
	{
		for (i = OBJECT_3D_INVALID_OBJECT_INDEX + 1; i < OBJECT_3D_LAST; i++)
		{
			if (tp_generations[tp_gen_last].rows[i].state == 2 && objects_3d_scene_database[i].skin.animation < 0)
			{
				scene = i;
				break;
			}
		}
	}
	/* the three cases, each applied, observed, restored and checked before the next; a restoration that fails stops everything (the state is then not trusted, and no frame may run) */
	tp_setup_done = 1;
	rows[1].available = mobiles > 0 && scene > 0;
	rows[1].reason = mobiles == 0 ? "no_mobile_with_instance" : (scene < 0 ? (tp_gen_last < 0 ? "no_census_pass" : "no_bbox_scene_without_skin") : "-");
	rows[2].available = mobiles > 0;
	rows[2].reason = mobiles == 0 ? "no_mobile_with_instance" : "-";
	rows[3].available = structure != NULL;
	rows[3].reason = structure ? "-" : "no_fixed_structure_with_valid_shape";
	if (rows[1].available)
	{
		setup_instance_case (&rows[1], mobile[0], scene);
	}
	if (rows[1].available && (!rows[1].restored || !rows[1].counters_restored || !rows[1].vp_unchanged))
	{
		return probe_refused (text, capacity, "restoration_mismatch_case_1", visited);
	}
	if (rows[2].available)
	{
		setup_no_instance_case (&rows[2], mobile[mobiles - 1]);
	}
	if (rows[2].available && (!rows[2].restored || !rows[2].vp_unchanged))
	{
		return probe_refused (text, capacity, "restoration_mismatch_case_2", visited);
	}
	if (rows[3].available)
	{
		setup_invalid_shape_case (&rows[3], structure);
	}
	if (rows[3].available && !rows[3].restored)
	{
		return probe_refused (text, capacity, "restoration_mismatch_case_3", visited);
	}
	out_start (&out, text, capacity);
	emit (&out, "tp-setup status=complete visited=%d entities=%d\n", visited, examined);
	for (which = 1; which <= TP_SETUP_CASES; which++)
	{
		setup_emit_row (&out, which, &rows[which]);
	}
	if (out.overflow)
	{
		return probe_refused (text, capacity, "report_capacity_after_restoration", visited);
	}
	return 0;
}

#endif
