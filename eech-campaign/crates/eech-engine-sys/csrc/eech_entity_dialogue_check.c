/*
 * TEST-ONLY (EEDCS-49, feature `entity-dialogue-check`): the isolated control of the campaign's half of the host-owned entity collision narrow phase (aphavoc/.../mb_hostentity.h).
 *
 * It includes the REAL dialogue (the header the shipping mb_hostmodel.c includes) and binds it to a scripted host, so what is checked is the shipped code's behaviour at the
 * boundary: what it sends (the segment and the pose, never the caller's outputs), what it writes into the caller's buffers (only the components the host reports the original
 * assigned, and nothing at all on a refusal) and when it stops the engine.  It calls no original code, loads no model and says nothing about the host's kernel (eech-world).
 *
 * THE PRECONDITION THIS EXISTS FOR.  A real caller (answer_weapon_step_lookup in wn_move.c) passes an automatic face normal it never initialised, so the intercept point and normal
 * are outputs, not inputs.  Where the platform has memory protection the buffers are placed on pages that cannot be read: a dialogue that read, loaded, copied or forwarded a word
 * of a buffer the original had not assigned would fault.  Elsewhere the same cases run on ordinary buffers and the request is compared across two different buffer contents.
 *
 * The same control covers the campaign's half of the host's per-candidate decisions (ENTITY_CANDIDATE): the exchange (mb_hostentity.h: the 44 facts sent bit for bit in the order of
 * mb_hostmodel.h, each of the seven states, the narrow phase pose of 12 or 16 floats, scene 0 for a candidate whose shape was not read, and what stops the engine: a state the facts do not allow,
 * a pose of another size or with a non-finite float, a quiet or signalling NaN included, another scene's name under its own entity_candidate.identity label, a refusal) and the campaign's
 * OFFER (mb_hostoffer.h, the very code en_cllsn.c runs) over scripted entities that log every getter in order: a candidate the host skips is read for nothing the original did not read, a direct
 * city building needs no scene, the destructive UPDATED getter and the terrain question are performed once and only when named.
 *
 * eech_entity_dialogue_check_run returns the number of failed expectations (0: all held) and writes one line per failure; *checks is the number of expectations made.
 */

#ifdef EECH_ENTITY_DIALOGUE_CHECK

#include <setjmp.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#if defined(__linux__)
#include <sys/mman.h>
#define HAVE_GUARD_PAGES 1
#else
#define HAVE_GUARD_PAGES 0
#endif

#include "entity/mobile/mb_hostmodel.h"

#define FALSE 0
#define TRUE 1

/* ---- the scripted host ---- */

#define SCENES 100
#define SCENE_NAME "CANDIDATE"

struct scripted_answer
{
	int result;
	int write_mask;
	uint32_t words[6];
	const char *name;
};

struct scripted_candidate
{
	int state;
	int count;
	uint32_t pose[16];
	const char *name;
};

static struct scripted_candidate decision;
static struct scripted_candidate queue[8];
static int queue_length, queue_next;
static struct scripted_answer answer;
static campaign_model_request seen;
static int asks;
static int refuse_the_exchange;

static jmp_buf fatal_jump;
static char fatal_text[512];

static void debug_fatal (const char *format, ...)
{
	va_list arguments;

	va_start (arguments, format);
	vsnprintf (fatal_text, sizeof (fatal_text), format, arguments);
	va_end (arguments);

	longjmp (fatal_jump, 1);
}

static void scripted_ask (campaign_model_request *request, campaign_model_reply *reply)
{
	memset (reply, 0, sizeof (*reply));

	seen = *request;
	asks++;

	if (refuse_the_exchange)
	{
		debug_fatal ("host model: [model-provider:callback] the scripted host refused");
	}

	if (request->op == CAMPAIGN_MODEL_ENTITY_CANDIDATE)
	{
		const struct scripted_candidate *given = (queue_next < queue_length) ? &queue[queue_next++] : &decision;

		reply->candidate_state = given->state;
		reply->candidate_count = given->count;
		memcpy (reply->candidate, given->pose, sizeof (given->pose));
		/* the scene's name, as the host echoes it: empty for a candidate that has no scene, unless the script names another */
		strncpy (reply->name, given->name ? given->name : ((request->scene == 0) ? "" : SCENE_NAME), sizeof (reply->name) - 1);

		return;
	}

	reply->result = answer.result;
	reply->write_mask = answer.write_mask;
	memcpy (reply->points, answer.words, sizeof (answer.words));
	strncpy (reply->name, answer.name, sizeof (reply->name) - 1);
}

static void scripted_scene (int scene)
{
	if ((scene < 1) || (scene >= SCENES))
	{
		debug_fatal ("host model: [model-provider:scene.index] scene %d is outside 1 to %d", scene, SCENES - 1);
	}
}

static void scripted_identity (const campaign_model_reply *reply, int scene)
{
	if (strcmp (reply->name, SCENE_NAME) != 0)
	{
		debug_fatal ("host model: [model-provider:entity_collision.identity] the host named scene %d \"%.60s\"", scene, reply->name);
	}
}

static void scripted_candidate_identity (const campaign_model_reply *reply, int scene)
{
	if (strcmp (reply->name, SCENE_NAME) != 0)
	{
		debug_fatal ("host model: [model-provider:entity_candidate.identity] the host named scene %d \"%.60s\"", scene, reply->name);
	}
}

#define CAMPAIGN_ENTITY_REQUIRE_SCENE(SCENE) scripted_scene (SCENE)
#define CAMPAIGN_ENTITY_ASK(REQUEST,REPLY) scripted_ask ((REQUEST), (REPLY))
#define CAMPAIGN_ENTITY_IDENTITY(REPLY,SCENE) scripted_identity ((REPLY), (SCENE))
#define CAMPAIGN_ENTITY_CANDIDATE_ASK(REQUEST,REPLY) scripted_ask ((REQUEST), (REPLY))
#define CAMPAIGN_ENTITY_CANDIDATE_IDENTITY(REPLY,SCENE) scripted_candidate_identity ((REPLY), (SCENE))

#include "entity/mobile/mb_hostentity.h"

/* ---- the control ---- */

struct tc
{
	char *text;
	int capacity, used, failures, checks;
};

static void tc_expect (struct tc *t, int condition, const char *label, int line)
{
	t->checks++;

	if (!condition)
	{
		int n;

		t->failures++;
		n = snprintf (t->text + t->used, (size_t) (t->capacity - t->used), "FAIL line %d: %s\n", line, label);
		if ((n > 0) && (n < t->capacity - t->used))
		{
			t->used += n;
		}
	}
}

#define EXPECT(T,C) tc_expect ((T), (C), #C, __LINE__)

/* words that must stay exactly, or arrive exactly: a signed zero, a subnormal, a signalling NaN, an infinity, a quiet NaN with a payload */
static const uint32_t SENTINEL_POINT[3] = { 0x80000000u, 0x00000001u, 0x7fa00001u };
static const uint32_t SENTINEL_NORMAL[3] = { 0x7f800000u, 0xffc00123u, 0x80000001u };
/* computed words an answer carries: they include a signed zero and a NaN payload, which are legitimate results of arithmetic */
static const uint32_t COMPUTED[6] = { 0x40490fdbu, 0x80000000u, 0x7fa00002u, 0x00000000u, 0x3f800000u, 0xffc00124u };

/* a buffer of three floats on a page of its own, optionally unreadable and unwritable */
struct buffer
{
	float *floats;
	void *page;
	size_t length;
	int guarded;
};

static struct buffer buffer_make (const uint32_t *initial, int guarded)
{
	struct buffer b;

	memset (&b, 0, sizeof (b));

#if HAVE_GUARD_PAGES
	b.length = 4096;
	b.page = mmap (NULL, b.length, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
	if (b.page == MAP_FAILED)
	{
		b.page = NULL;
	}
	else
	{
		b.floats = (float *) b.page;
		if (initial)
		{
			memcpy (b.floats, initial, 3 * sizeof (float));
		}
		if (guarded)
		{
			if (mprotect (b.page, b.length, PROT_NONE) == 0)
			{
				b.guarded = TRUE;
			}
		}
	}
#else
	(void) guarded;
	b.page = NULL;
#endif

	if (!b.page)
	{
		static float fallback[8][3];
		static int next;

		b.floats = fallback[next++ % 8];
		if (initial)
		{
			memcpy (b.floats, initial, 3 * sizeof (float));
		}
		else
		{
			memset (b.floats, 0x5a, 3 * sizeof (float));
		}
	}

	return (b);
}

static void buffer_free (struct buffer *b)
{
#if HAVE_GUARD_PAGES
	if (b->page)
	{
		munmap (b->page, b->length);
	}
#else
	(void) b;
#endif
}

/* a guarded buffer is made readable again only for the check's own comparison, after the dialogue has finished */
static int buffer_is (struct buffer *b, const uint32_t *words)
{
#if HAVE_GUARD_PAGES
	if (b->page && b->guarded)
	{
		mprotect (b->page, b->length, PROT_READ);
	}
#endif

	return (memcmp (b->floats, words, 3 * sizeof (float)) == 0);
}

static const float SEGMENT_OLD[3] = { 1.0f, 2.0f, 3.0f };
static const float SEGMENT_NEW[3] = { 4.0f, 5.0f, 6.0f };

static void candidate_words (float *candidate, int count)
{
	static const uint32_t odd[4] = { 0x80000000u, 0x00000001u, 0x7fa00003u, 0x7f800000u };
	int i;

	for (i = 0; i < count; i++)
	{
		memcpy (&candidate[i], &odd[i % 4], sizeof (float));
	}
}

struct outcome
{
	int stopped, hit;
};

/* one exchange with the scripted answer; a stop is caught and its text kept */
static struct outcome ask_once (int scene, int candidate_count, struct buffer *point, struct buffer *normal)
{
	struct outcome o;
	float candidate[16];

	memset (&o, 0, sizeof (o));
	candidate_words (candidate, 16);
	fatal_text[0] = '\0';

	if (setjmp (fatal_jump) == 0)
	{
		o.hit = campaign_entity_collision_exchange (scene, SEGMENT_OLD, SEGMENT_NEW, candidate, candidate_count, point->floats, normal->floats);
	}
	else
	{
		o.stopped = TRUE;
	}

	return (o);
}

static void script (int result, int mask, const uint32_t *words, const char *name)
{
	memset (&answer, 0, sizeof (answer));
	answer.result = result;
	answer.write_mask = mask;
	if (words)
	{
		memcpy (answer.words, words, sizeof (answer.words));
	}
	answer.name = name;
}

/* an answer whose six words are COMPUTED for the components the mask names and zero for the others, as the host gives it */
static void script_masked (int result, int mask)
{
	uint32_t words[6];
	int i;

	for (i = 0; i < 6; i++)
	{
		words[i] = (mask & (1 << i)) ? COMPUTED[i] : 0;
	}

	script (result, mask, words, SCENE_NAME);
}

/* a refusal: the dialogue stopped with `marker`, asked the host once or not at all, and left BOTH buffers byte for byte as they were */
static void expect_refusal (struct tc *t, const char *label, const char *marker, int scene, int candidate_count, int line)
{
	struct buffer point = buffer_make (SENTINEL_POINT, FALSE), normal = buffer_make (SENTINEL_NORMAL, FALSE);
	struct outcome o;
	char what[160];

	o = ask_once (scene, candidate_count, &point, &normal);

	snprintf (what, sizeof (what), "%s: stopped", label);
	tc_expect (t, o.stopped, what, line);
	snprintf (what, sizeof (what), "%s: stopped at %s (said: %.80s)", label, marker, fatal_text);
	tc_expect (t, strstr (fatal_text, marker) != NULL, what, line);
	snprintf (what, sizeof (what), "%s: the intercept point is untouched", label);
	tc_expect (t, buffer_is (&point, SENTINEL_POINT), what, line);
	snprintf (what, sizeof (what), "%s: the face normal is untouched", label);
	tc_expect (t, buffer_is (&normal, SENTINEL_NORMAL), what, line);

	buffer_free (&point);
	buffer_free (&normal);
}


/* ---- ENTITY_CANDIDATE: the exchange, and the campaign's offer (mb_hostoffer.h) over scripted entities ---- */

static void script_candidate (int state, int count, const char *name)
{
	int i;

	memset (&decision, 0, sizeof (decision));
	decision.state = state;
	decision.count = count;
	decision.name = name;

	for (i = 0; i < count; i++)
	{
		/* a signed zero, a subnormal and ordinary numbers: all finite, all exact */
		static const uint32_t finite[4] = { 0x80000000u, 0x00000001u, 0x3f800000u, 0xc2c80000u };

		decision.pose[i] = finite[i % 4];
	}
}

/* the facts of a candidate of one kind: odd but finite words everywhere, then the flags that make the kind. The stage flags say what was read: the fixed facts of a fixed or city candidate,
   the placement of a candidate that has one */
static void make_facts (float *facts, int mobile, int fixed, int city, int has_buildings, int of_city, int align)
{
	int i;

	for (i = 0; i < CAMPAIGN_MODEL_CANDIDATE_FACTS; i++)
	{
		facts[i] = (float) i + 0.25f;
	}

	facts[CAMPAIGN_CANDIDATE_MOBILE] = (float) mobile;
	facts[CAMPAIGN_CANDIDATE_FIXED] = (float) fixed;
	facts[CAMPAIGN_CANDIDATE_CITY] = (float) city;
	facts[CAMPAIGN_CANDIDATE_CITY_BUILDING] = 0.0f;
	facts[CAMPAIGN_CANDIDATE_OF_CITY] = (float) of_city;
	facts[CAMPAIGN_CANDIDATE_HAS_BUILDINGS] = (float) has_buildings;
	facts[CAMPAIGN_CANDIDATE_UPDATED] = 0.0f;
	facts[CAMPAIGN_CANDIDATE_ALIGN] = (float) align;
	facts[CAMPAIGN_CANDIDATE_NORMAL_GIVEN] = 0.0f;
	facts[CAMPAIGN_CANDIDATE_FIXED_GIVEN] = (float) (fixed || city);
	facts[CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] = (float) (align || mobile);
}

struct candidate_outcome
{
	int stopped, state, pose_count;
	float pose[16];
};

static struct candidate_outcome ask_candidate (int scene, const float *facts)
{
	struct candidate_outcome o;

	memset (&o, 0, sizeof (o));
	fatal_text[0] = '\0';
	asks = 0;

	if (setjmp (fatal_jump) == 0)
	{
		o.state = campaign_entity_candidate_exchange (scene, facts, o.pose, &o.pose_count);
	}
	else
	{
		o.stopped = TRUE;
	}

	return (o);
}

static void expect_candidate_refusal (struct tc *t, const char *label, const char *marker, int scene, const float *facts, int line)
{
	struct candidate_outcome o = ask_candidate (scene, facts);
	char what[160];

	snprintf (what, sizeof (what), "%s: stopped", label);
	tc_expect (t, o.stopped, what, line);
	snprintf (what, sizeof (what), "%s: stopped at %s (said: %.80s)", label, marker, fatal_text);
	tc_expect (t, strstr (fatal_text, marker) != NULL, what, line);
	snprintf (what, sizeof (what), "%s: no pose was handed over", label);
	tc_expect (t, o.pose_count == 0, what, line);
}

/* ---- the offer: scripted entities that log every getter the campaign calls, in order ---- */

struct sentity
{
	int mobile, fixed, city, city_building, shape, align, updated;
	float position[3], motion[3], attitude[9], heading, box_min[3], box_max[3];
	struct sentity *first_building, *next_building;
};

static char reads[128];
static int collision_calls, collision_scene, collision_count, collision_hit;
static const float *collision_old, *collision_new;

static void read_log (char letter)
{
	size_t n = strlen (reads);

	if (n + 1 < sizeof (reads))
	{
		reads[n] = letter;
		reads[n + 1] = '\0';
	}
}

static int so_mobile (struct sentity *e) { read_log ('M'); return (e->mobile); }
static int so_fixed (struct sentity *e) { read_log ('F'); return (e->fixed); }
static int so_is_city (struct sentity *e) { read_log ('C'); return (e->city); }
static int so_is_city_building (struct sentity *e) { read_log ('G'); return (e->city_building); }
static int so_shape (struct sentity *e) { read_log ('S'); return (e->shape); }
static void so_position (struct sentity *e, float *out) { read_log ('P'); memcpy (out, e->position, sizeof (e->position)); }
static void so_motion (struct sentity *e, float *out) { read_log ('V'); memcpy (out, e->motion, sizeof (e->motion)); }
static void so_attitude (struct sentity *e, float *out) { read_log ('A'); memcpy (out, e->attitude, sizeof (e->attitude)); }
static float so_heading (struct sentity *e) { read_log ('H'); return (e->heading); }
static int so_align (struct sentity *e) { read_log ('L'); return (e->align); }
static struct sentity *so_first_building (struct sentity *e) { read_log ('B'); return (e->first_building); }
static struct sentity *so_next_building (struct sentity *e) { return (e->next_building); }
static void so_box (struct sentity *e, float *min, float *max) { read_log ('X'); memcpy (min, e->box_min, sizeof (e->box_min)); memcpy (max, e->box_max, sizeof (e->box_max)); }
static int so_updated (struct sentity *e) { read_log ('U'); return (e->updated); }
static void so_terrain_normal (float x, float z, float *out) { read_log ('N'); out[0] = x * 0.0f; out[1] = 1.0f; out[2] = z * 0.0f; }

#define CAMPAIGN_OFFER_ENTITY struct sentity
#define CAMPAIGN_OFFER_MOBILE_FLAG(E) so_mobile (E)
#define CAMPAIGN_OFFER_FIXED_FLAG(E) so_fixed (E)
#define CAMPAIGN_OFFER_IS_CITY(E) so_is_city (E)
#define CAMPAIGN_OFFER_IS_CITY_BUILDING(E) so_is_city_building (E)
#define CAMPAIGN_OFFER_SHAPE(E) so_shape (E)
#define CAMPAIGN_OFFER_POSITION(E,OUT) so_position ((E), (OUT))
#define CAMPAIGN_OFFER_MOTION(E,OUT) so_motion ((E), (OUT))
#define CAMPAIGN_OFFER_ATTITUDE(E,OUT) so_attitude ((E), (OUT))
#define CAMPAIGN_OFFER_HEADING(E) so_heading (E)
#define CAMPAIGN_OFFER_ALIGN(E) so_align (E)
#define CAMPAIGN_OFFER_FIRST_BUILDING(E) so_first_building (E)
#define CAMPAIGN_OFFER_NEXT_BUILDING(B) so_next_building (B)
#define CAMPAIGN_OFFER_BOX(E,MIN,MAX) so_box ((E), (MIN), (MAX))
#define CAMPAIGN_OFFER_UPDATED(E) so_updated (E)
#define CAMPAIGN_OFFER_TERRAIN_NORMAL(X,Z,OUT) so_terrain_normal ((X), (Z), (OUT))
#define CAMPAIGN_OFFER_DELTA_TIME() 0.0625f

/* the campaign's two host calls, bound to the REAL candidate exchange (with its complete reply check) and to a recording narrow phase */
#define campaign_model_entity_candidate check_model_entity_candidate
#define campaign_model_entity_collision check_model_entity_collision

static int check_model_entity_candidate (int scene, const float *facts, float *pose, int *pose_count)
{
	return (campaign_entity_candidate_exchange (scene, facts, pose, pose_count));
}

static int check_model_entity_collision (int scene, const float *segment_old, const float *segment_new, const float *candidate, int candidate_count, float *point, float *normal)
{
	(void) candidate;
	(void) point;
	(void) normal;

	collision_calls++;
	collision_scene = scene;
	collision_count = candidate_count;
	collision_old = segment_old;
	collision_new = segment_new;

	return (collision_hit);
}

#include "entity/mobile/mb_hostoffer.h"

#undef campaign_model_entity_candidate
#undef campaign_model_entity_collision

/* the decisions the scripted host gives, in order, one per question; the name is the scene's (empty for scene 0) unless one is given */
static void queue_reset (void)
{
	memset (&decision, 0, sizeof (decision));
	queue_length = 0;
	queue_next = 0;
}

static void queue_state (int state, int count)
{
	static const uint32_t finite[4] = { 0x80000000u, 0x00000001u, 0x3f800000u, 0xc2c80000u };
	int i;

	memset (&queue[queue_length], 0, sizeof (queue[0]));
	queue[queue_length].state = state;
	queue[queue_length].count = count;

	for (i = 0; i < count; i++)
	{
		queue[queue_length].pose[i] = finite[i % 4];
	}

	queue_length++;
}

struct offer_outcome
{
	int stopped;
	struct sentity *hit;
};

static struct offer_outcome offer (struct sentity *target, int of_city, float tallest)
{
	struct offer_outcome o;
	float point[3], normal[3];

	memset (&o, 0, sizeof (o));
	memset (point, 0, sizeof (point));
	memset (normal, 0, sizeof (normal));
	reads[0] = '\0';
	collision_calls = 0;
	asks = 0;
	fatal_text[0] = '\0';

	if (setjmp (fatal_jump) == 0)
	{
		o.hit = campaign_offer_candidate (target, of_city, tallest, SEGMENT_OLD, SEGMENT_NEW, point, normal);
	}
	else
	{
		o.stopped = TRUE;
	}

	return (o);
}

static struct sentity make_entity (int mobile, int fixed, int shape)
{
	struct sentity e;
	static const float identity[9] = { 1.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 1.0f };

	memset (&e, 0, sizeof (e));
	e.mobile = mobile;
	e.fixed = fixed;
	e.shape = shape;
	e.position[0] = 10.0f;
	e.motion[0] = 5.0f;
	e.heading = 0.5f;
	memcpy (e.attitude, identity, sizeof (identity));

	return (e);
}

static void offer_cases (struct tc *t)
{
	struct sentity e, b1, b2;
	struct offer_outcome o;

	/* A building listed directly in a sector has no shape. Skipped by its sector's height it is read for nothing but its mobile flag, and the question carries no scene. */
	e = make_entity (FALSE, TRUE, 0);
	e.city_building = TRUE;
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
	o = offer (&e, FALSE, -500.0f);
	EXPECT (t, !o.stopped && o.hit == NULL && strcmp (reads, "M") == 0 && asks == 1 && seen.scene == 0);

	/* The same building in a sector that does not skip it: the host asks for the fixed facts, the campaign reads the flag and the type and still has no shape to give; the host's skip is accepted with scene 0. */
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, !o.stopped && o.hit == NULL && strcmp (reads, "MFCG") == 0 && asks == 2 && seen.scene == 0);
	EXPECT (t, seen.pose[CAMPAIGN_CANDIDATE_CITY_BUILDING] == 1.0f && seen.pose[CAMPAIGN_CANDIDATE_FIXED_GIVEN] == 1.0f && seen.pose[CAMPAIGN_CANDIDATE_FIXED] == 1.0f);

	/* A fixed candidate in a sector below the segment whose geometry is unusable (shape 0) is read for its mobile flag only: no fixed flag, shape, position, heading or align flag */
	e = make_entity (FALSE, TRUE, 0);
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
	o = offer (&e, FALSE, -500.0f);
	EXPECT (t, !o.stopped && o.hit == NULL && strcmp (reads, "M") == 0 && collision_calls == 0 && seen.scene == 0);

	/* A mobile candidate the host skips was read for its shape, position and motion vector, and not for its attitude or its UPDATED flag */
	e = make_entity (TRUE, FALSE, 7);
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
	o = offer (&e, FALSE, -500.0f);
	EXPECT (t, !o.stopped && o.hit == NULL && strcmp (reads, "MSPV") == 0 && seen.scene == 7);

	/* a moving mobile candidate: the destructive getter once, when named, then the placement, then the narrow phase at the host's pose; a hit returns the candidate */
	e = make_entity (TRUE, FALSE, 7);
	e.updated = TRUE;
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_UPDATED, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_TEST, 16);
	collision_hit = TRUE;
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, !o.stopped && o.hit == &e && strcmp (reads, "MSPVUA") == 0 && asks == 3);
	EXPECT (t, collision_calls == 1 && collision_scene == 7 && collision_count == 16 && collision_old == SEGMENT_OLD && collision_new == SEGMENT_NEW);
	EXPECT (t, seen.pose[CAMPAIGN_CANDIDATE_UPDATED] == 2.0f && seen.pose[CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] == 1.0f);
	collision_hit = FALSE;

	/* a fixed candidate aligned with the terrain, in the original's order: the fixed facts, the placement, the terrain question, then the narrow phase; a miss returns nothing */
	e = make_entity (FALSE, TRUE, 9);
	e.align = TRUE;
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_NORMAL, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_TEST, 12);
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, !o.stopped && o.hit == NULL && strcmp (reads, "MFCGSPHLN") == 0 && asks == 4);
	EXPECT (t, collision_calls == 1 && collision_scene == 9 && collision_count == 12);
	EXPECT (t, seen.pose[CAMPAIGN_CANDIDATE_NORMAL_GIVEN] == 1.0f && seen.pose[CAMPAIGN_CANDIDATE_NORMAL + 1] == 1.0f);

	/* a candidate whose branch needs geometry it cannot give (shape 0): the host's narrow phase for it is refused, nothing is tested */
	e = make_entity (FALSE, TRUE, 0);
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_TEST, 12);
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, o.stopped && strstr (fatal_text, "entity_candidate.state") != NULL && collision_calls == 0);

	/* a city: its facts (first building, position, box), then its buildings in list order, each read for its shape and position; the second is hit and returned */
	e = make_entity (FALSE, TRUE, 0);
	e.city = TRUE;
	b1 = make_entity (FALSE, TRUE, 3);
	b2 = make_entity (FALSE, TRUE, 4);
	e.first_building = &b1;
	b1.next_building = &b2;
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_BUILDINGS, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_TEST, 12);
	collision_hit = TRUE;
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, !o.stopped && o.hit == &b2 && strcmp (reads, "MFCBPXSPSPMFHL") == 0 && asks == 5);
	EXPECT (t, collision_calls == 1 && collision_scene == 4 && collision_count == 12);
	collision_hit = FALSE;

	/* the host's NEED answers are bounded by the facts: fixed facts asked of a mobile candidate stop the engine before anything else is read */
	e = make_entity (TRUE, FALSE, 7);
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED, 0);
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, o.stopped && strstr (fatal_text, "entity_candidate.state") != NULL && strcmp (reads, "MSPV") == 0);
}

static void candidate_cases (struct tc *t)
{
	float facts[CAMPAIGN_MODEL_CANDIDATE_FACTS], more[CAMPAIGN_MODEL_CANDIDATE_FACTS];
	struct candidate_outcome o;
	int i, same;

	/* the question: exactly the 44 facts, bit for bit, in the layout the header names, for the scene, and nothing else */
	make_facts (facts, TRUE, FALSE, FALSE, FALSE, FALSE, FALSE);
	queue_reset ();
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0, SCENE_NAME);
	memset (&seen, 0x7e, sizeof (seen));
	o = ask_candidate (7, facts);
	EXPECT (t, !o.stopped && o.state == CAMPAIGN_MODEL_CANDIDATE_SKIP && o.pose_count == 0 && asks == 1);
	EXPECT (t, seen.op == CAMPAIGN_MODEL_ENTITY_CANDIDATE && seen.scene == 7 && seen.pose_count == CAMPAIGN_MODEL_CANDIDATE_FACTS);
	same = TRUE;
	for (i = 0; i < CAMPAIGN_MODEL_CANDIDATE_FACTS; i++)
	{
		same = same && (memcmp (&seen.pose[i], &facts[i], sizeof (float)) == 0);
	}
	EXPECT (t, same);

	/* a candidate whose shape has not been read has scene 0, and the host's skip of it is accepted with an empty name */
	make_facts (more, FALSE, FALSE, FALSE, FALSE, FALSE, FALSE);
	more[CAMPAIGN_CANDIDATE_FIXED_GIVEN] = 0.0f;
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0, "");
	o = ask_candidate (0, more);
	EXPECT (t, !o.stopped && o.state == CAMPAIGN_MODEL_CANDIDATE_SKIP && seen.scene == 0);

	/* each state the facts allow is accepted, and only TEST carries a pose */
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_NEED_UPDATED, 0, SCENE_NAME);
	o = ask_candidate (7, facts);
	EXPECT (t, !o.stopped && o.state == CAMPAIGN_MODEL_CANDIDATE_NEED_UPDATED && o.pose_count == 0);

	script_candidate (CAMPAIGN_MODEL_CANDIDATE_TEST, 16, SCENE_NAME);
	o = ask_candidate (7, facts);
	EXPECT (t, !o.stopped && o.state == CAMPAIGN_MODEL_CANDIDATE_TEST && o.pose_count == 16);
	same = TRUE;
	for (i = 0; i < 16; i++)
	{
		same = same && (memcmp (&o.pose[i], &decision.pose[i], sizeof (float)) == 0);
	}
	EXPECT (t, same);

	script_candidate (CAMPAIGN_MODEL_CANDIDATE_TEST, 12, SCENE_NAME);
	o = ask_candidate (7, facts);
	EXPECT (t, !o.stopped && o.state == CAMPAIGN_MODEL_CANDIDATE_TEST && o.pose_count == 12);

	/* the next stages of a non-mobile candidate the sector skip did not remove */
	make_facts (more, FALSE, FALSE, FALSE, FALSE, FALSE, FALSE);
	more[CAMPAIGN_CANDIDATE_FIXED_GIVEN] = 0.0f;
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED, 0, "");
	o = ask_candidate (0, more);
	EXPECT (t, !o.stopped && o.state == CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED);

	make_facts (more, FALSE, TRUE, FALSE, FALSE, FALSE, FALSE);
	more[CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] = 0.0f;
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0, SCENE_NAME);
	o = ask_candidate (7, more);
	EXPECT (t, !o.stopped && o.state == CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT);

	/* a fixed candidate aligned with the terrain, its placement read, may ask for the face normal, once */
	make_facts (more, FALSE, TRUE, FALSE, FALSE, FALSE, TRUE);
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_NEED_NORMAL, 0, SCENE_NAME);
	o = ask_candidate (7, more);
	EXPECT (t, !o.stopped && o.state == CAMPAIGN_MODEL_CANDIDATE_NEED_NORMAL);

	/* a city has no scene of its own: scene 0 and an empty name; its buildings are named only when it has one */
	make_facts (more, FALSE, TRUE, TRUE, TRUE, FALSE, FALSE);
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_BUILDINGS, 0, "");
	o = ask_candidate (0, more);
	EXPECT (t, !o.stopped && o.state == CAMPAIGN_MODEL_CANDIDATE_BUILDINGS && seen.scene == 0);

	/* a building listed under a city: a stationary pose of 12 floats */
	make_facts (more, FALSE, TRUE, FALSE, FALSE, TRUE, FALSE);
	more[CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] = 1.0f;
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_TEST, 12, SCENE_NAME);
	o = ask_candidate (9, more);
	EXPECT (t, !o.stopped && o.state == CAMPAIGN_MODEL_CANDIDATE_TEST && o.pose_count == 12);

	/* refusals: each stops the engine with its marker and hands nothing over */
	make_facts (facts, TRUE, FALSE, FALSE, FALSE, FALSE, FALSE);

	script_candidate (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0, SCENE_NAME);
	refuse_the_exchange = TRUE;
	expect_candidate_refusal (t, "the host refuses the exchange", "model-provider:callback", 7, facts, __LINE__);
	refuse_the_exchange = FALSE;

	expect_candidate_refusal (t, "a scene beyond the catalogue", "scene.index", SCENES, facts, __LINE__);
	expect_candidate_refusal (t, "a negative scene", "scene.index", -1, facts, __LINE__);

	script_candidate (0, 0, SCENE_NAME);
	expect_candidate_refusal (t, "state 0", "entity_candidate.state", 7, facts, __LINE__);
	script_candidate (8, 0, SCENE_NAME);
	expect_candidate_refusal (t, "state 8", "entity_candidate.state", 7, facts, __LINE__);
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_SKIP, 12, SCENE_NAME);
	expect_candidate_refusal (t, "a skip that carries a pose", "entity_candidate.pose", 7, facts, __LINE__);

	/* the scene's name is the candidate's own refusal, not the narrow phase's */
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0, "OTHER");
	expect_candidate_refusal (t, "another scene's name", "entity_candidate.identity", 7, facts, __LINE__);
	EXPECT (t, strstr (fatal_text, "entity_collision.identity") == NULL);
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0, "OTHER");
	expect_candidate_refusal (t, "a name for a candidate that has no scene", "entity_candidate.identity", 0, facts, __LINE__);

	/* the UPDATED flag is asked of a mobile candidate once: not of a fixed one, not of a building under a city, not again */
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_NEED_UPDATED, 0, SCENE_NAME);
	make_facts (more, FALSE, TRUE, FALSE, FALSE, FALSE, FALSE);
	expect_candidate_refusal (t, "UPDATED asked of a fixed candidate", "entity_candidate.state", 7, more, __LINE__);
	make_facts (more, TRUE, FALSE, FALSE, FALSE, TRUE, FALSE);
	expect_candidate_refusal (t, "UPDATED asked of a building under a city", "entity_candidate.state", 7, more, __LINE__);
	make_facts (more, TRUE, FALSE, FALSE, FALSE, FALSE, FALSE);
	more[CAMPAIGN_CANDIDATE_UPDATED] = 2.0f;
	expect_candidate_refusal (t, "UPDATED asked again", "entity_candidate.state", 7, more, __LINE__);

	/* the fixed facts are asked of a candidate that is not mobile, once */
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED, 0, SCENE_NAME);
	expect_candidate_refusal (t, "fixed facts asked of a mobile candidate", "entity_candidate.state", 7, facts, __LINE__);
	make_facts (more, FALSE, TRUE, FALSE, FALSE, FALSE, FALSE);
	expect_candidate_refusal (t, "fixed facts asked again", "entity_candidate.state", 7, more, __LINE__);
	make_facts (more, FALSE, FALSE, FALSE, FALSE, TRUE, FALSE);
	expect_candidate_refusal (t, "fixed facts asked of a building under a city", "entity_candidate.state", 7, more, __LINE__);

	/* the placement is asked once, and only by a branch that has reached one */
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0, SCENE_NAME);
	make_facts (more, FALSE, TRUE, FALSE, FALSE, FALSE, FALSE);
	more[CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] = 1.0f;
	expect_candidate_refusal (t, "placement asked again", "entity_candidate.state", 7, more, __LINE__);
	make_facts (more, FALSE, FALSE, FALSE, FALSE, FALSE, FALSE);
	more[CAMPAIGN_CANDIDATE_FIXED_GIVEN] = 0.0f;
	expect_candidate_refusal (t, "placement asked before the fixed facts were read", "entity_candidate.state", 7, more, __LINE__);

	/* the terrain's face normal is asked of a fixed candidate that aligns with the terrain and has its placement, once */
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_NEED_NORMAL, 0, SCENE_NAME);
	make_facts (more, FALSE, TRUE, FALSE, FALSE, FALSE, FALSE);
	expect_candidate_refusal (t, "a normal asked of a candidate that does not align", "entity_candidate.state", 7, more, __LINE__);
	make_facts (more, FALSE, TRUE, FALSE, FALSE, FALSE, TRUE);
	more[CAMPAIGN_CANDIDATE_NORMAL_GIVEN] = 1.0f;
	expect_candidate_refusal (t, "a normal asked again", "entity_candidate.state", 7, more, __LINE__);
	make_facts (more, TRUE, FALSE, FALSE, FALSE, FALSE, TRUE);
	expect_candidate_refusal (t, "a normal asked of a mobile candidate", "entity_candidate.state", 7, more, __LINE__);
	make_facts (more, FALSE, TRUE, FALSE, FALSE, FALSE, TRUE);
	more[CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] = 0.0f;
	expect_candidate_refusal (t, "a normal asked before the placement was read", "entity_candidate.state", 7, more, __LINE__);

	/* buildings are named only for a city that has one */
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_BUILDINGS, 0, SCENE_NAME);
	make_facts (more, FALSE, TRUE, FALSE, FALSE, FALSE, FALSE);
	expect_candidate_refusal (t, "buildings named for a candidate that is not a city", "entity_candidate.state", 7, more, __LINE__);
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_BUILDINGS, 0, "");
	make_facts (more, FALSE, TRUE, TRUE, FALSE, FALSE, FALSE);
	expect_candidate_refusal (t, "buildings named for a city without one", "entity_candidate.state", 0, more, __LINE__);

	/* the pose of a test: 12 or 16 finite floats, 16 only for a mobile candidate, and a candidate that has a scene */
	make_facts (facts, TRUE, FALSE, FALSE, FALSE, FALSE, FALSE);
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_TEST, 13, SCENE_NAME);
	expect_candidate_refusal (t, "a pose of 13 floats", "entity_candidate.pose", 7, facts, __LINE__);
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_TEST, 0, SCENE_NAME);
	expect_candidate_refusal (t, "a test without a pose", "entity_candidate.pose", 7, facts, __LINE__);
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_TEST, 16, SCENE_NAME);
	decision.pose[9] = 0x7f800000u;
	expect_candidate_refusal (t, "an infinite float in the pose", "entity_candidate.pose", 7, facts, __LINE__);
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_TEST, 12, SCENE_NAME);
	decision.pose[0] = 0x7fc00000u;
	expect_candidate_refusal (t, "a quiet NaN in the pose", "entity_candidate.pose", 7, facts, __LINE__);
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_TEST, 12, SCENE_NAME);
	decision.pose[5] = 0x7fa00001u;
	expect_candidate_refusal (t, "a signalling NaN in the pose", "entity_candidate.pose", 7, facts, __LINE__);
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_TEST, 16, SCENE_NAME);
	decision.pose[15] = 0xffa00001u;
	expect_candidate_refusal (t, "a negative signalling NaN in the last float of a moving pose", "entity_candidate.pose", 7, facts, __LINE__);
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_TEST, 16, SCENE_NAME);
	make_facts (more, FALSE, TRUE, FALSE, FALSE, FALSE, FALSE);
	expect_candidate_refusal (t, "a moving pose for a candidate that is not mobile", "entity_candidate.pose", 7, more, __LINE__);
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_TEST, 12, "");
	expect_candidate_refusal (t, "a narrow phase for a candidate that has no scene", "entity_candidate.state", 0, facts, __LINE__);

	/* a city with a scene */
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0, "");
	make_facts (more, FALSE, TRUE, TRUE, TRUE, FALSE, FALSE);
	expect_candidate_refusal (t, "a city with a scene of its own", "entity_candidate.request", 7, more, __LINE__);

	offer_cases (t);
}

int eech_entity_dialogue_check_run (char *text, int capacity, int *checks)
{
	struct tc t;
	struct buffer point, normal, other_point, other_normal;
	struct outcome o;
	float expected[22];

	t.text = text;
	t.capacity = capacity;
	t.used = 0;
	t.failures = 0;
	t.checks = 0;
	text[0] = '\0';

	/* ---- the request: the segment then the candidate, bit for bit, and nothing of the caller's outputs ---- */
	candidate_words (expected, 16);

	/* a MISS that assigned nothing (not a state the original reaches, but the dialogue's domain allows it): both buffers are on pages that cannot be read or written, and the
	   exchange completes: no word of a buffer the original did not assign is read, loaded, copied or forwarded */
	point = buffer_make (NULL, TRUE);
	normal = buffer_make (NULL, TRUE);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, 0);
	asks = 0;
	o = ask_once (7, 12, &point, &normal);
	EXPECT (&t, !o.stopped && !o.hit && asks == 1);
	EXPECT (&t, seen.op == CAMPAIGN_MODEL_ENTITY_COLLISION && seen.scene == 7 && seen.pose_count == CAMPAIGN_MODEL_ENTITY_STATIONARY_POSE);
	EXPECT (&t, memcmp (&seen.pose[0], SEGMENT_OLD, sizeof (SEGMENT_OLD)) == 0 && memcmp (&seen.pose[3], SEGMENT_NEW, sizeof (SEGMENT_NEW)) == 0);
	EXPECT (&t, memcmp (&seen.pose[6], expected, 12 * sizeof (float)) == 0);
	buffer_free (&point);
	buffer_free (&normal);

	/* a moving candidate: 16 candidate floats and a pose of 22 */
	point = buffer_make (SENTINEL_POINT, FALSE);
	normal = buffer_make (SENTINEL_NORMAL, FALSE);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	o = ask_once (7, 16, &point, &normal);
	EXPECT (&t, !o.stopped && !o.hit && seen.pose_count == CAMPAIGN_MODEL_ENTITY_MOVING_POSE);
	EXPECT (&t, memcmp (&seen.pose[6], expected, 16 * sizeof (float)) == 0);
	buffer_free (&point);
	buffer_free (&normal);

	/* the request does not depend on what the caller's buffers hold: two different contents give the same words */
	{
		static const uint32_t other_a[3] = { 0x11111111u, 0x22222222u, 0x33333333u };
		static const uint32_t other_b[3] = { 0xffc00001u, 0x80000000u, 0x00000000u };
		campaign_model_request first;

		point = buffer_make (other_a, FALSE);
		normal = buffer_make (other_a, FALSE);
		other_point = buffer_make (other_b, FALSE);
		other_normal = buffer_make (other_b, FALSE);
		script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
		(void) ask_once (7, 12, &point, &normal);
		first = seen;
		(void) ask_once (7, 12, &other_point, &other_normal);
		EXPECT (&t, memcmp (&first, &seen, sizeof (first)) == 0);
		/* the unassigned point of both stays exactly as it was */
		EXPECT (&t, buffer_is (&point, other_a) && buffer_is (&other_point, other_b));
		buffer_free (&point);
		buffer_free (&normal);
		buffer_free (&other_point);
		buffer_free (&other_normal);
	}

	/* ---- a HIT: both vectors assigned, written with the computed words (a signed zero and NaN payloads are results like any other) ---- */
	point = buffer_make (SENTINEL_POINT, FALSE);
	normal = buffer_make (SENTINEL_NORMAL, FALSE);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_FOUND, CAMPAIGN_ENTITY_EVERYTHING_WRITTEN);
	o = ask_once (7, 12, &point, &normal);
	EXPECT (&t, !o.stopped && o.hit);
	EXPECT (&t, buffer_is (&point, &COMPUTED[0]) && buffer_is (&normal, &COMPUTED[3]));
	buffer_free (&point);
	buffer_free (&normal);

	/* ---- a MISS that crossed a plane outside the face: both vectors assigned and written, and it is still a miss ---- */
	point = buffer_make (SENTINEL_POINT, FALSE);
	normal = buffer_make (SENTINEL_NORMAL, FALSE);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_EVERYTHING_WRITTEN);
	o = ask_once (7, 12, &point, &normal);
	EXPECT (&t, !o.stopped && !o.hit);
	EXPECT (&t, buffer_is (&point, &COMPUTED[0]) && buffer_is (&normal, &COMPUTED[3]));
	buffer_free (&point);
	buffer_free (&normal);

	/* ---- a MISS that never crossed a plane: only the normal was assigned. The point's page cannot be read or written, and its initialised signed-zero, subnormal and NaN words
	   stay exactly (checked on a second, readable buffer) ---- */
	point = buffer_make (NULL, TRUE);
	normal = buffer_make (SENTINEL_NORMAL, FALSE);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	o = ask_once (7, 12, &point, &normal);
	EXPECT (&t, !o.stopped && !o.hit);
	EXPECT (&t, buffer_is (&normal, &COMPUTED[3]));
	buffer_free (&point);
	buffer_free (&normal);

	point = buffer_make (SENTINEL_POINT, FALSE);
	normal = buffer_make (SENTINEL_NORMAL, FALSE);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	o = ask_once (7, 12, &point, &normal);
	EXPECT (&t, !o.stopped && !o.hit);
	EXPECT (&t, buffer_is (&point, SENTINEL_POINT));
	EXPECT (&t, buffer_is (&normal, &COMPUTED[3]));
	buffer_free (&point);
	buffer_free (&normal);

	/* a computed zero is a written word, not an unassigned one: a degenerate face assigns a zero normal */
	{
		static const uint32_t zeros[6] = { 0, 0, 0, 0, 0, 0 };

		point = buffer_make (SENTINEL_POINT, FALSE);
		normal = buffer_make (SENTINEL_NORMAL, FALSE);
		script (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_NORMAL_WRITTEN, zeros, SCENE_NAME);
		o = ask_once (7, 12, &point, &normal);
		EXPECT (&t, !o.stopped && !o.hit && buffer_is (&point, SENTINEL_POINT) && buffer_is (&normal, &zeros[0]));
		buffer_free (&point);
		buffer_free (&normal);
	}

	/* ---- refusals before any caller commit: each leaves both initialised buffers byte for byte as they were ---- */
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	refuse_the_exchange = TRUE;
	expect_refusal (&t, "the host refuses the exchange", "model-provider:callback", 7, 12, __LINE__);
	refuse_the_exchange = FALSE;

	expect_refusal (&t, "scene 0", "scene.index", 0, 12, __LINE__);
	expect_refusal (&t, "a scene beyond the catalogue", "scene.index", SCENES, 12, __LINE__);
	expect_refusal (&t, "a candidate of 13 floats", "entity_collision.pose", 7, 13, __LINE__);
	expect_refusal (&t, "a candidate of no floats", "entity_collision.pose", 7, 0, __LINE__);

	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	answer.name = "OTHER";
	expect_refusal (&t, "another scene's name", "entity_collision.identity", 7, 12, __LINE__);

	script_masked (0, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	expect_refusal (&t, "a result that is neither a hit nor a miss (0)", "entity_collision.result", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_ABSENT, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	expect_refusal (&t, "a result of the other kind (absent)", "entity_collision.result", 7, 12, __LINE__);

	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, 64);
	expect_refusal (&t, "write mask 64", "entity_collision.write_mask", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, -1);
	expect_refusal (&t, "write mask -1", "entity_collision.write_mask", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, 0x01);
	expect_refusal (&t, "a point written in part", "entity_collision.write_mask", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, 0x0b);
	expect_refusal (&t, "a point component and a normal component", "entity_collision.write_mask", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, 0x08);
	expect_refusal (&t, "a normal written in part", "entity_collision.write_mask", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_POINT_WRITTEN);
	expect_refusal (&t, "a point without the normal", "entity_collision.write_mask", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_FOUND, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	expect_refusal (&t, "a hit that assigned only the normal", "entity_collision.write_mask", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_FOUND, 0);
	expect_refusal (&t, "a hit that assigned nothing", "entity_collision.write_mask", 7, 12, __LINE__);

	/* a word for a component the mask says was not assigned: even a signed zero is a word */
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, CAMPAIGN_ENTITY_NORMAL_WRITTEN);
	answer.words[1] = 0x80000000u;
	expect_refusal (&t, "a negative zero for an unassigned point component", "entity_collision.unwritten", 7, 12, __LINE__);
	script_masked (CAMPAIGN_MODEL_TRIANGLE_END, 0);
	answer.words[5] = 0x00000001u;
	expect_refusal (&t, "a word for an unassigned normal component", "entity_collision.unwritten", 7, 12, __LINE__);

	/* every refusal above stopped before a commit even where the first words were fine: the normal (written first) is untouched when the POINT part is the defect */
	script_masked (CAMPAIGN_MODEL_TRIANGLE_FOUND, CAMPAIGN_ENTITY_EVERYTHING_WRITTEN);
	answer.write_mask = 0x3b;
	expect_refusal (&t, "a point with one component missing", "entity_collision.write_mask", 7, 12, __LINE__);

	candidate_cases (&t);

	if (checks)
	{
		*checks = t.checks;
	}

	return (t.failures);
}

#endif
