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
 * It also covers the ENTITY_POSE OBSERVATION (mb_hostmodel.h, fifth slice): the offer asks it beside the original's reads (the position and motion of a mobile candidate; its attitude when the broad phase
 * passed), with the candidate's type, its alive flag and its semantic host identity, and NO decision, read, count or order of the candidate dialogue depends on the answer (the scripted pose host is
 * kept apart from the candidate queue, so every earlier expectation is checked unchanged). The shape of the answer is checked completely and a malformed one stops the engine by marker.
 *
 * eech_entity_dialogue_check_run returns the number of failed expectations (0: all held) and writes one line per failure; *checks is the number of expectations made.
 */

#ifdef EECH_ENTITY_DIALOGUE_CHECK

#include <setjmp.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
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

/* the scripted host's decisions about the ENUMERATION, in order, one per question; and every scan question the campaign asked (phase, cursor, excluded, hit) */
struct scripted_scan
{
	int state;
	uint32_t cursor[2];
};

static struct scripted_scan scan_queue[48];
static int scan_queue_length, scan_queue_next, scan_cycle;
static float scan_seen[64][4];
static int scan_seen_count;
/* the key (the identity place of the question) the campaign named with each scan question; the key the scripted host serves INSTEAD of the coherent one when `scan_key_forced` is set, and a name it
   puts on a state that serves no entry when `scan_stray_name` is set; and how a coherent host picks the key it serves (set once the scripted sector lists exist) */
static char scan_identity_seen[64][24];
static const char *scan_key_forced, *scan_stray_name;
static const char *(*scripted_served_key) (int state, const campaign_model_request *request, const campaign_model_reply *reply);
/* the roster's events (the same question, phases 4 to 6): the question the campaign asked last and how many, and the scripted host's answer (a coherent one unless overridden) */
static campaign_model_request roster_seen;
static int roster_asks, roster_override, roster_state, roster_issue = 1;
static const char *roster_name;
/* the CITY events (phases 7 to 12): the scripted host's lists (every parent's buildings, head first, in one array), the question the campaign asked last and how many, the last identity of a LINK and a NAME,
   and an answer forced instead of the coherent one when `city_override` is set */
struct city_entry
{
	char parent[24];
	char key[24];
};

static struct city_entry city_list[64];
static int city_count, city_asks, city_override, city_state;
static const char *city_name;
static campaign_model_request city_seen;
static char city_last_link[160], city_last_name[160];

/* the index of the entry with `key` in the scripted lists, or -1 */
static int city_find (const char *key)
{
	int i;

	for (i = 0; i < city_count; i++)
	{
		if (strcmp (city_list[i].key, key) == 0)
		{
			return (i);
		}
	}

	return (-1);
}

/* the first and the second key of an identity place (the second empty when there is none) */
static void city_keys (const char *identity, char *first, char *second)
{
	const char *space = strchr (identity, ' ');

	if (space)
	{
		memcpy (first, identity, (size_t) (space - identity));
		first[space - identity] = 0;
		strcpy (second, space + 1);
	}
	else
	{
		strcpy (first, identity);
		second[0] = 0;
	}
}

static void scripted_city (const campaign_model_request *request, campaign_model_reply *reply)
{
	char first[160], second[160];
	int phase = (int) request->pose[CAMPAIGN_SCAN_PHASE], i, at;

	city_seen = *request;
	city_asks++;
	city_keys (request->host_id, first, second);

	switch (phase)
	{
		case CAMPAIGN_SCAN_PHASE_CHILD_INSERT:
		{
			/* at the head of the parent's buildings, or right after the named predecessor */
			at = city_count;

			for (i = 0; i < city_count; i++)
			{
				if (strcmp (city_list[i].parent, first) == 0)
				{
					at = i;
					break;
				}
			}

			if (second[0])
			{
				at = city_find (second) + 1;
			}

			memmove (&city_list[at + 1], &city_list[at], (size_t) (city_count - at) * sizeof (city_list[0]));
			strcpy (city_list[at].parent, first);
			snprintf (city_list[at].key, sizeof (city_list[at].key), "%d", roster_issue++);
			city_count++;
			reply->candidate_state = CAMPAIGN_MODEL_SCAN_PLACED;
			strcpy (reply->name, city_list[at].key);
			break;
		}
		case CAMPAIGN_SCAN_PHASE_CHILD_REMOVE:
		{
			at = city_find (second);

			if (at >= 0)
			{
				memmove (&city_list[at], &city_list[at + 1], (size_t) (city_count - at - 1) * sizeof (city_list[0]));
				city_count--;
			}

			reply->candidate_state = CAMPAIGN_MODEL_SCAN_REMOVED;
			break;
		}
		case CAMPAIGN_SCAN_PHASE_CHILD_FIRST:
		case CAMPAIGN_SCAN_PHASE_CHILD_NEXT:
		{
			reply->candidate_state = CAMPAIGN_MODEL_SCAN_NEXT;
			at = (phase == CAMPAIGN_SCAN_PHASE_CHILD_NEXT) ? city_find (second) + 1 : 0;

			for (i = at; i < city_count; i++)
			{
				if (strcmp (city_list[i].parent, first) == 0)
				{
					strcpy (reply->name, city_list[i].key);
					break;
				}
			}

			break;
		}
		case CAMPAIGN_SCAN_PHASE_LINK:
		{
			strcpy (city_last_link, request->host_id);
			reply->candidate_state = CAMPAIGN_MODEL_SCAN_PLACED;
			strcpy (reply->name, second);
			break;
		}
		default:
		{
			strcpy (city_last_name, request->host_id);
			reply->candidate_state = CAMPAIGN_MODEL_SCAN_PLACED;
			strcpy (reply->name, first);
			break;
		}
	}

	if (city_override)
	{
		reply->candidate_state = city_state;
		memset (reply->name, 0, sizeof (reply->name));
		strncpy (reply->name, city_name, sizeof (reply->name) - 1);
	}
}
static struct scripted_answer answer;
static campaign_model_request seen;
/* the ENTITY_POSE observations the campaign asked (kept apart from `seen` and `asks`, which describe the candidate dialogue), and the scripted host's answer to them: by default a recorded identity
   (record kind 1, no differences) for a named candidate and "no identity" for an unnamed one, unless a script overrides the answer */
static campaign_model_request pose_seen[8];
static int pose_asks;
static int pose_override, pose_state, pose_kind, pose_mask;
/* a separate answer for the placement stage (depth 1), when pose_split is set */
static int pose_split, pose_state1, pose_kind1, pose_mask1;
/* the observation's three added read-only metadata reads (type, alive, roster identity), counted apart from `reads`, which is the transcript of the original's PHYSICAL getters only */
static int meta_type_reads, meta_alive_reads, meta_id_reads;
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

	if (request->op == CAMPAIGN_MODEL_ENTITY_POSE)
	{
		if (pose_asks < 8)
		{
			pose_seen[pose_asks] = *request;
		}

		pose_asks++;

		if (refuse_the_exchange)
		{
			debug_fatal ("host model: [model-provider:callback] the scripted host refused");
		}

		if (pose_override && pose_split && request->depth == CAMPAIGN_POSE_STAGE_PLACEMENT)
		{
			reply->candidate_state = pose_state1;
			reply->candidate_count = pose_kind1;
			reply->write_mask = pose_mask1;
		}
		else if (pose_override)
		{
			reply->candidate_state = pose_state;
			reply->candidate_count = pose_kind;
			reply->write_mask = pose_mask;
		}
		else if (request->host_id[0] != 0)
		{
			reply->candidate_state = CAMPAIGN_MODEL_POSE_RECORDED;
			reply->candidate_count = 1;
		}
		else
		{
			reply->candidate_state = CAMPAIGN_MODEL_POSE_NO_ID;
		}

		return;
	}

	if ((request->op == CAMPAIGN_MODEL_ENTITY_SCAN) && ((int) request->pose[CAMPAIGN_SCAN_PHASE] >= CAMPAIGN_SCAN_PHASE_CHILD_INSERT))
	{
		if (refuse_the_exchange)
		{
			debug_fatal ("host model: [model-provider:callback] the scripted host refused");
		}

		scripted_city (request, reply);

		return;
	}

	if ((request->op == CAMPAIGN_MODEL_ENTITY_SCAN) && ((int) request->pose[CAMPAIGN_SCAN_PHASE] >= CAMPAIGN_SCAN_PHASE_INSERT))
	{
		int phase = (int) request->pose[CAMPAIGN_SCAN_PHASE];

		roster_seen = *request;
		roster_asks++;

		if (refuse_the_exchange)
		{
			debug_fatal ("host model: [model-provider:callback] the scripted host refused");
		}

		reply->candidate_state = roster_override ? roster_state : ((phase == CAMPAIGN_SCAN_PHASE_INSERT) ? CAMPAIGN_MODEL_SCAN_PLACED : ((phase == CAMPAIGN_SCAN_PHASE_REMOVE) ? CAMPAIGN_MODEL_SCAN_REMOVED : CAMPAIGN_MODEL_SCAN_CLEARED));

		if (roster_override)
		{
			strncpy (reply->name, roster_name, sizeof (reply->name) - 1);
		}
		else if (phase == CAMPAIGN_SCAN_PHASE_INSERT)
		{
			snprintf (reply->name, sizeof (reply->name), "%d", roster_issue++);
		}

		return;
	}

	seen = *request;
	asks++;

	if (refuse_the_exchange)
	{
		debug_fatal ("host model: [model-provider:callback] the scripted host refused");
	}

	if (request->op == CAMPAIGN_MODEL_ENTITY_SCAN)
	{
		if (scan_seen_count < 64)
		{
			scan_seen[scan_seen_count][0] = request->pose[CAMPAIGN_SCAN_PHASE];
			scan_seen[scan_seen_count][1] = request->pose[CAMPAIGN_SCAN_CURSOR];
			scan_seen[scan_seen_count][2] = request->pose[CAMPAIGN_SCAN_EXCLUDED];
			scan_seen[scan_seen_count][3] = request->pose[CAMPAIGN_SCAN_HIT];
			strncpy (scan_identity_seen[scan_seen_count], request->host_id, sizeof (scan_identity_seen[0]) - 1);
			scan_identity_seen[scan_seen_count][sizeof (scan_identity_seen[0]) - 1] = 0;
		}

		scan_seen_count++;

		if (scan_cycle)
		{
			/* a host that never ends the scan: the first sector again and again */
			reply->candidate_state = CAMPAIGN_MODEL_SCAN_FIRST;
		}
		else if (scan_queue_next < scan_queue_length)
		{
			const struct scripted_scan *given = &scan_queue[scan_queue_next++];

			reply->candidate_state = given->state;
			memcpy (&reply->candidate[0], &given->cursor[0], sizeof (float));
			memcpy (&reply->candidate[1], &given->cursor[1], sizeof (float));
		}
		else
		{
			reply->candidate_state = 0;
		}

		/* the entry the host serves, by its key: the coherent host serves what its roster (the scripted lists) holds at that place, unless a case forces a key */
		if ((reply->candidate_state == CAMPAIGN_MODEL_SCAN_FIRST) || (reply->candidate_state == CAMPAIGN_MODEL_SCAN_NEXT))
		{
			const char *served = scan_key_forced ? scan_key_forced : (scripted_served_key ? scripted_served_key (reply->candidate_state, request, reply) : "");

			strncpy (reply->name, served, sizeof (reply->name) - 1);
		}
		else if (scan_stray_name)
		{
			strncpy (reply->name, scan_stray_name, sizeof (reply->name) - 1);
		}

		return;
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
#define CAMPAIGN_ENTITY_SCAN_ASK(REQUEST,REPLY) scripted_ask ((REQUEST), (REPLY))
#define CAMPAIGN_ENTITY_POSE_ASK(REQUEST,REPLY) scripted_ask ((REQUEST), (REPLY))
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
		o.state = campaign_entity_candidate_exchange (scene, facts, NULL, o.pose, &o.pose_count);
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
	/* the observation's reads (not logged in `reads`: they are the additions of the fifth slice, and the original's reads are what that log describes) */
	int type, alive;
	const char *host_id;
	float position[3], motion[3], attitude[9], heading, box_min[3], box_max[3];
	struct sentity *first_building, *next_building;
	/* a sector list (the scan's cursor serves it): the sector's tallest structure and first child, an entry's successor and predecessor */
	float tallest;
	struct sentity *first_child, *succ, *pred;
	int id;
	/* the key the scripted host's roster holds for the entity in a list (assigned by sector_list; empty when it is in none) */
	char key[24];
	/* the entity's slot in the placement heap (pheap) when it is placed in the host's city lists: see city_register */
	int pslot;
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
/* a city's buildings are the HOST's list: the real placement map asks the scripted host and compares the building it serves with the scripted entity's own list (the mirror); defined after the map */
static struct sentity *so_city_first (struct sentity *city);
static struct sentity *so_city_next (struct sentity *city, struct sentity *building);
struct tc;
static int city_register (struct tc *t, struct sentity *city, struct sentity *b1, struct sentity *b2);
static void so_box (struct sentity *e, float *min, float *max) { read_log ('X'); memcpy (min, e->box_min, sizeof (e->box_min)); memcpy (max, e->box_max, sizeof (e->box_max)); }
static int so_updated (struct sentity *e) { read_log ('U'); return (e->updated); }
static int so_type (struct sentity *e) { meta_type_reads++; return (e->type); }
static int so_alive (struct sentity *e) { meta_alive_reads++; return (e->alive); }
static const char *so_host_id (struct sentity *e) { meta_id_reads++; return (e->host_id); }
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
#define CAMPAIGN_OFFER_FIRST_BUILDING(E) so_city_first (E)
#define CAMPAIGN_OFFER_NEXT_BUILDING(C,B) so_city_next ((C), (B))
#define CAMPAIGN_OFFER_BOX(E,MIN,MAX) so_box ((E), (MIN), (MAX))
#define CAMPAIGN_OFFER_UPDATED(E) so_updated (E)
#define CAMPAIGN_OFFER_TERRAIN_NORMAL(X,Z,OUT) so_terrain_normal ((X), (Z), (OUT))
#define CAMPAIGN_OFFER_DELTA_TIME() 0.0625f
#define CAMPAIGN_OFFER_TYPE(E) so_type (E)
#define CAMPAIGN_OFFER_ALIVE(E) so_alive (E)
#define CAMPAIGN_OFFER_HOST_ID(E) so_host_id (E)

/* the campaign's two host calls, bound to the REAL candidate exchange (with its complete reply check) and to a recording narrow phase */
#define campaign_model_entity_candidate check_model_entity_candidate
#define campaign_model_entity_collision check_model_entity_collision
#define campaign_model_entity_pose check_model_entity_pose

static int check_model_entity_pose (int stage, int entity_type, int alive, const char *host_id, const float *words)
{
	return (campaign_entity_pose_exchange (stage, entity_type, alive, host_id, words));
}

static int check_model_entity_candidate (int scene, const float *facts, const char *host_id, float *pose, int *pose_count)
{
	return (campaign_entity_candidate_exchange (scene, facts, host_id, pose, pose_count));
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
#undef campaign_model_entity_pose

/* ---- the host's cursor over scripted sector lists: the campaign's scan driver (mb_hostscan.h) bound to a 3 x 3 map whose lists log every getter ---- */

static struct sentity sectors[3][3];

static float ss_tallest (struct sentity *s) { read_log ('t'); return (s->tallest); }
static struct sentity *ss_first (struct sentity *s) { read_log ('h'); return (s->first_child); }
static struct sentity *ss_succ (struct sentity *e) { read_log ('s'); return (e->succ); }
static struct sentity *ss_pred (struct sentity *e) { read_log ('p'); return (e->pred); }

#define CAMPAIGN_SCAN_ENTITY struct sentity
#define CAMPAIGN_SCAN_SIDE_LENGTH() 100.0f
#define CAMPAIGN_SCAN_MAP_MIN_X() 0.0f
#define CAMPAIGN_SCAN_MAP_MIN_Z() 0.0f
#define CAMPAIGN_SCAN_MAP_MAX_X() 299.0f
#define CAMPAIGN_SCAN_MAP_MAX_Z() 299.0f
#define CAMPAIGN_SCAN_SECTOR_VALID(X,Z) (((X) >= 0) && ((X) < 3) && ((Z) >= 0) && ((Z) < 3))
#define CAMPAIGN_SCAN_SECTOR(X,Z) (&sectors[(X)][(Z)])
#define CAMPAIGN_SCAN_TALLEST(S) ss_tallest (S)
#define CAMPAIGN_SCAN_FIRST(S) ss_first (S)
#define CAMPAIGN_SCAN_SUCC(E) ss_succ (E)
#define CAMPAIGN_SCAN_PRED(E) ss_pred (E)
#define CAMPAIGN_SCAN_ENTITY_OF(KEY) scripted_entity_of ((KEY))

/* the entity the host's key names: found by the key alone, in no list order and with no logged read (the real binding resolves a key through the campaign's placement table, which reads no entity) */
static struct sentity *scripted_entity_of (const char *key)
{
	int x, z;

	for (x = 0; x < 3; x++)
	{
		for (z = 0; z < 3; z++)
		{
			struct sentity *e;

			for (e = sectors[x][z].first_child; e; e = e->succ)
			{
				if (e->key[0] && (strcmp (e->key, key) == 0))
				{
					return (e);
				}
			}
		}
	}

	return (NULL);
}

/* a coherent host serves the key of the entry its roster holds at the place it was asked about: the TAIL of the sector it names, or the predecessor of the entry the question names */
static const char *scripted_serve (int state, const campaign_model_request *request, const campaign_model_reply *reply)
{
	struct sentity *e;

	if (state == CAMPAIGN_MODEL_SCAN_FIRST)
	{
		float x, z;

		memcpy (&x, &reply->candidate[0], sizeof (float));
		memcpy (&z, &reply->candidate[1], sizeof (float));

		/* a sector outside the map has no list (the comparisons are false for a NaN too): the campaign refuses the cursor itself */
		if (!((x >= 0.0f) && (x < 3.0f) && (z >= 0.0f) && (z < 3.0f)))
		{
			return ("");
		}

		e = sectors[(int) x][(int) z].first_child;

		while (e && e->succ)
		{
			e = e->succ;
		}

		return (e ? e->key : "");
	}

	e = scripted_entity_of (request->host_id);

	return ((e && e->pred) ? e->pred->key : "");
}

#define campaign_model_entity_scan check_model_entity_scan

static int check_model_entity_scan (const float *facts, const char *entry_key, float *cursor, char *key)
{
	return (campaign_entity_scan_exchange (facts, entry_key, cursor, key));
}

#include "entity/mobile/mb_hostscan.h"

#undef campaign_model_entity_scan

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
	e.type = 3;
	e.alive = 1;

	return (e);
}

static void pose_reset (void)
{
	memset (pose_seen, 0, sizeof (pose_seen));
	pose_asks = 0;
	pose_override = 0;
	pose_state = 0;
	pose_kind = 0;
	pose_mask = 0;
	pose_split = 0;
	pose_state1 = 0;
	pose_kind1 = 0;
	pose_mask1 = 0;
	meta_type_reads = 0;
	meta_alive_reads = 0;
	meta_id_reads = 0;
}

/* the words an observation carried, as the bits they were read as */
static int pose_words_are (const campaign_model_request *q, const float *position, const float *motion, const float *attitude)
{
	float expected[CAMPAIGN_MODEL_POSE_FACTS];

	memset (expected, 0, sizeof (expected));
	memcpy (&expected[0], position, 3 * sizeof (float));
	memcpy (&expected[3], motion, 3 * sizeof (float));

	if (attitude)
	{
		memcpy (&expected[6], attitude, 9 * sizeof (float));
	}

	return ((q->pose_count == CAMPAIGN_MODEL_POSE_FACTS) && (memcmp (q->pose, expected, sizeof (expected)) == 0));
}

static void pose_cases (struct tc *t);

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
	city_register (t, &e, &b1, &b2);
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

	/* A malformed NEED_PLACEMENT is refused by the exchange BEFORE the offer reads a heading, an align flag or an attitude, and before any narrow phase. A city: read as far as its fixed facts. */
	e = make_entity (FALSE, TRUE, 0);
	e.city = TRUE;
	b1 = make_entity (FALSE, TRUE, 3);
	e.first_building = &b1;
	city_register (t, &e, &b1, NULL);
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, o.stopped && strstr (fatal_text, "entity_candidate.state") != NULL && strcmp (reads, "MFCBPX") == 0 && collision_calls == 0);

	/* a building listed directly in a sector: its type and nothing more */
	e = make_entity (FALSE, TRUE, 0);
	e.city_building = TRUE;
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, o.stopped && strstr (fatal_text, "entity_candidate.state") != NULL && strcmp (reads, "MFCG") == 0 && collision_calls == 0);

	/* an entity whose fixed flag says it is neither mobile nor fixed: no shape, no placement */
	e = make_entity (FALSE, FALSE, 5);
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, o.stopped && strstr (fatal_text, "entity_candidate.state") != NULL && strcmp (reads, "MF") == 0 && collision_calls == 0);

	/* a fixed candidate whose shape is unusable (scene 0), and a mobile one: no heading, align flag or attitude is read */
	e = make_entity (FALSE, TRUE, 0);
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, o.stopped && strstr (fatal_text, "entity_candidate.state") != NULL && strcmp (reads, "MFCGSP") == 0 && collision_calls == 0);
	e = make_entity (TRUE, FALSE, 0);
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, o.stopped && strstr (fatal_text, "entity_candidate.state") != NULL && strcmp (reads, "MSPV") == 0 && collision_calls == 0);

	/* a building under a city that has no shape: its flags are read during a placement only a building with a shape may have */
	e = make_entity (FALSE, TRUE, 0);
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	o = offer (&e, TRUE, 1000.0f);
	EXPECT (t, o.stopped && strstr (fatal_text, "entity_candidate.state") != NULL && strcmp (reads, "SP") == 0 && collision_calls == 0);

	/* the host's NEED answers are bounded by the facts: fixed facts asked of a mobile candidate stop the engine before anything else is read */
	e = make_entity (TRUE, FALSE, 7);
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED, 0);
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, o.stopped && strstr (fatal_text, "entity_candidate.state") != NULL && strcmp (reads, "MSPV") == 0);
}

/* ---- ENTITY_POSE: the observation beside the original's reads ---- */

static void pose_cases (struct tc *t)
{
	struct sentity e, b1;
	struct offer_outcome o;

	/* a named mobile candidate the host skips: ONE observation, asked at the offer with the words the original read (position, motion), nothing else, beside the original's reads */
	e = make_entity (TRUE, FALSE, 7);
	e.type = 4;
	e.alive = 1;
	e.host_id = "eech-world:12";
	e.position[1] = 2.0f;
	e.position[2] = -3.5f;
	e.motion[1] = 0.25f;
	pose_reset ();
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
	o = offer (&e, FALSE, -500.0f);
	EXPECT (t, !o.stopped && o.hit == NULL && strcmp (reads, "MSPV") == 0 && asks == 1 && pose_asks == 1);
	EXPECT (t, pose_seen[0].op == CAMPAIGN_MODEL_ENTITY_POSE && pose_seen[0].scene == 0 && pose_seen[0].depth == CAMPAIGN_POSE_STAGE_OFFER && pose_seen[0].kind == 4 && pose_seen[0].definition == 1);
	EXPECT (t, strcmp (pose_seen[0].host_id, "eech-world:12") == 0 && pose_words_are (&pose_seen[0], e.position, e.motion, NULL));
	/* the observation's reads are the three added metadata ones, once each per question, and the transcript of the original's physical getters above is exactly the original's */
	EXPECT (t, meta_type_reads == 1 && meta_alive_reads == 1 && meta_id_reads == 1);
	/* the candidate question of a mobile carries the SAME identity (the host decides its pose from its own record of it); the facts' position and motion words are still the ones the original read */
	EXPECT (t, strcmp (seen.host_id, "eech-world:12") == 0);

	/* a moving mobile candidate through the whole dialogue: the offer observation, then the placement observation with the attitude the original read; the decisions, reads, counts and the hit are exactly
	   those without the observation (see offer_cases) */
	e.updated = TRUE;
	e.alive = 0;
	e.attitude[2] = 0.5f;
	pose_reset ();
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_UPDATED, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_TEST, 16);
	collision_hit = TRUE;
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, !o.stopped && o.hit == &e && strcmp (reads, "MSPVUA") == 0 && asks == 3 && collision_calls == 1 && pose_asks == 2);
	EXPECT (t, pose_seen[0].depth == CAMPAIGN_POSE_STAGE_OFFER && pose_seen[0].definition == 0 && pose_words_are (&pose_seen[0], e.position, e.motion, NULL));
	EXPECT (t, pose_seen[1].depth == CAMPAIGN_POSE_STAGE_PLACEMENT && pose_seen[1].kind == 4 && strcmp (pose_seen[1].host_id, "eech-world:12") == 0 && pose_words_are (&pose_seen[1], e.position, e.motion, e.attitude));
	/* two observations, ONE read of the identity (once per offer) and two of the type and alive flag; every candidate question of the offer carries the identity */
	EXPECT (t, meta_type_reads == 2 && meta_alive_reads == 2 && meta_id_reads == 1 && strcmp (seen.host_id, "eech-world:12") == 0);
	collision_hit = FALSE;

	/* a candidate that is not mobile is never observed: a skipped fixed one, a fixed one tested, and a city with its buildings */
	e = make_entity (FALSE, TRUE, 9);
	e.align = TRUE;
	e.host_id = "eech-world:3";
	pose_reset ();
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_NORMAL, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_TEST, 12);
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, !o.stopped && strcmp (reads, "MFCGSPHLN") == 0 && asks == 4 && pose_asks == 0 && meta_type_reads == 0 && meta_alive_reads == 0 && meta_id_reads == 0 && seen.host_id[0] == 0);
	e = make_entity (FALSE, TRUE, 0);
	e.city = TRUE;
	b1 = make_entity (FALSE, TRUE, 3);
	b1.host_id = "eech-world:4";
	e.first_building = &b1;
	city_register (t, &e, &b1, NULL);
	pose_reset ();
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_BUILDINGS, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, !o.stopped && o.hit == NULL && pose_asks == 0);

	/* a building listed under a city is a candidate of its city, never an observed mobile: even when its collision flags read as mobile during the placement, and it has an identity, no observation is asked */
	e = make_entity (FALSE, TRUE, 0);
	e.city = TRUE;
	b1 = make_entity (TRUE, FALSE, 3);
	b1.host_id = "eech-world:5";
	e.first_building = &b1;
	city_register (t, &e, &b1, NULL);
	pose_reset ();
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_FIXED, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_BUILDINGS, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_TEST, 16);
	collision_hit = TRUE;
	o = offer (&e, FALSE, 1000.0f);
	collision_hit = FALSE;
	EXPECT (t, !o.stopped && o.hit == &b1 && collision_calls == 1 && pose_asks == 0 && meta_type_reads == 0 && meta_alive_reads == 0 && meta_id_reads == 0);

	/* a candidate the campaign has no identity for: the observation says so by an empty key, and the host's "no identity" is the only class that fits it */
	e = make_entity (TRUE, FALSE, 7);
	pose_reset ();
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
	o = offer (&e, FALSE, -500.0f);
	EXPECT (t, !o.stopped && pose_asks == 1 && pose_seen[0].host_id[0] == 0 && seen.host_id[0] == 0);
	pose_override = TRUE;
	pose_state = CAMPAIGN_MODEL_POSE_RECORDED;
	pose_kind = 1;
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
	o = offer (&e, FALSE, -500.0f);
	EXPECT (t, o.stopped && strstr (fatal_text, "entity_pose.state") != NULL && asks == 0);

	/* NOTHING DECIDES FROM THE ANSWER: whatever the host says about the identity (unknown, retired, unrecorded, a record that differs in every way), the offer's decisions, reads, counts and hit are those of the
	   dialogue without it */
	{
		static const int classes[5][3] = {
			{ CAMPAIGN_MODEL_POSE_UNKNOWN, 0, 0 }, { CAMPAIGN_MODEL_POSE_RETIRED, 0, 0 }, { CAMPAIGN_MODEL_POSE_UNRECORDED, 0, 0 },
			{ CAMPAIGN_MODEL_POSE_RECORDED, 5, 16 }, { CAMPAIGN_MODEL_POSE_RECORDED, 2, 24 } };
		int i;

		e = make_entity (TRUE, FALSE, 7);
		e.type = 4;
		e.host_id = "eech-world:12";
		e.updated = TRUE;

		for (i = 0; i < 5; i++)
		{
			pose_reset ();
			pose_override = TRUE;
			pose_state = classes[i][0];
			pose_kind = classes[i][1];
			pose_mask = classes[i][2];
			queue_reset ();
			queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_UPDATED, 0);
			queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
			queue_state (CAMPAIGN_MODEL_CANDIDATE_TEST, 16);
			collision_hit = TRUE;
			o = offer (&e, FALSE, 1000.0f);
			EXPECT (t, !o.stopped && o.hit == &e && strcmp (reads, "MSPVUA") == 0 && asks == 3 && collision_calls == 1 && pose_asks == 2);
			collision_hit = FALSE;
		}
	}

	/* the answer's shape, checked completely before anything else is asked: a class outside the five, a record kind or a difference mask outside the contract, a class that has no record carrying one, a record
	   of no kind, "no identity" for a named candidate and, with the stage's own masks, an attitude difference at the OFFER stage (which reads no attitude) and a difference from a held motion vector together with
	   "none held". Each stops the engine by marker through the REAL shared checker, and no candidate question was asked */
	{
		static const int bad[18][3] = {
			{ 0, 0, 0 }, { 6, 0, 0 }, { CAMPAIGN_MODEL_POSE_RECORDED, 6, 0 }, { CAMPAIGN_MODEL_POSE_RECORDED, -1, 0 }, { CAMPAIGN_MODEL_POSE_RECORDED, 1, 32 },
			{ CAMPAIGN_MODEL_POSE_RECORDED, 1, -1 }, { CAMPAIGN_MODEL_POSE_UNKNOWN, 2, 0 }, { CAMPAIGN_MODEL_POSE_RETIRED, 0, 4 }, { CAMPAIGN_MODEL_POSE_RECORDED, 0, 0 },
			{ CAMPAIGN_MODEL_POSE_NO_ID, 0, 0 },
			{ CAMPAIGN_MODEL_POSE_RECORDED, 1, 2 }, { CAMPAIGN_MODEL_POSE_RECORDED, 1, 3 }, { CAMPAIGN_MODEL_POSE_RECORDED, 4, 10 },
			{ CAMPAIGN_MODEL_POSE_RECORDED, 1, 12 }, { CAMPAIGN_MODEL_POSE_RECORDED, 5, 28 },
			{ CAMPAIGN_MODEL_POSE_RECORDED, 1, 1 }, { CAMPAIGN_MODEL_POSE_RECORDED, 4, 4 }, { CAMPAIGN_MODEL_POSE_RECORDED, 1, 17 } };
		static const char *marker[18] = {
			"entity_pose.state", "entity_pose.state", "entity_pose.answer", "entity_pose.answer", "entity_pose.answer",
			"entity_pose.answer", "entity_pose.answer", "entity_pose.answer", "entity_pose.answer", "entity_pose.state",
			"entity_pose.answer", "entity_pose.answer", "entity_pose.answer", "entity_pose.answer", "entity_pose.answer",
			"entity_pose.mirror", "entity_pose.mirror", "entity_pose.mirror" };
		int i;

		e = make_entity (TRUE, FALSE, 7);
		e.host_id = "eech-world:12";

		for (i = 0; i < 18; i++)
		{
			pose_reset ();
			pose_override = TRUE;
			pose_state = bad[i][0];
			pose_kind = bad[i][1];
			pose_mask = bad[i][2];
			queue_reset ();
			queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
			o = offer (&e, FALSE, -500.0f);
			EXPECT (t, o.stopped && strstr (fatal_text, marker[i]) != NULL && asks == 0 && pose_asks == 1);
		}
	}

	/* the stage's own masks at the PLACEMENT stage, through the real shared checker: the attitude IS read there, so an attitude difference is a valid answer (the offer's answer was valid too, and the dialogue, its
	   reads, its counts and its hit are those without the observation); a difference from a held vector together with "none held" is not, and stops the engine at the placement, before the narrow phase */
	e = make_entity (TRUE, FALSE, 7);
	e.host_id = "eech-world:12";
	pose_reset ();
	pose_override = TRUE;
	pose_split = TRUE;
	pose_state = CAMPAIGN_MODEL_POSE_RECORDED;
	pose_kind = 1;
	pose_mask = 24;
	pose_state1 = CAMPAIGN_MODEL_POSE_RECORDED;
	pose_kind1 = 1;
	pose_mask1 = 24;
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_TEST, 16);
	collision_hit = TRUE;
	o = offer (&e, FALSE, 1000.0f);
	collision_hit = FALSE;
	EXPECT (t, !o.stopped && o.hit == &e && strcmp (reads, "MSPVA") == 0 && asks == 2 && pose_asks == 2 && collision_calls == 1);
	/* an attitude difference at the PLACEMENT stage is a valid shape (the attitude was read there) and a MIRROR difference: refused by that name, before the narrow phase */
	pose_mask1 = 26;
	pose_asks = 0;
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_TEST, 16);
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, o.stopped && strstr (fatal_text, "entity_pose.mirror") != NULL && asks == 1 && pose_asks == 2 && collision_calls == 0);
	pose_mask1 = 12;
	pose_asks = 0;
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_TEST, 16);
	o = offer (&e, FALSE, 1000.0f);
	EXPECT (t, o.stopped && strstr (fatal_text, "entity_pose.answer") != NULL && asks == 1 && pose_asks == 2 && collision_calls == 0);

	/* the roster's WHOLE identity domain is carried, unshortened and never skipped (1 byte, 127, the longest 128 and a 128 byte multi-byte one), and each decides nothing; one beyond the domain cannot exist and is
	   refused by marker, never shortened or skipped */
	{
		static const size_t lengths[4] = { 1, 127, 128, 128 };
		char id[CAMPAIGN_MODEL_HOST_ID_MAX + 2];
		size_t at, i;

		e = make_entity (TRUE, FALSE, 7);

		for (i = 0; i < 4; i++)
		{
			memset (id, 'x', sizeof (id));

			if (i == 3)
			{
				for (at = 0; at < 128; at += 2)
				{
					id[at] = (char) 0xc3;
					id[at + 1] = (char) 0xa9;
				}
			}

			id[lengths[i]] = '\0';
			e.host_id = id;
			pose_reset ();
			queue_reset ();
			queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
			o = offer (&e, FALSE, -500.0f);
			EXPECT (t, !o.stopped && o.hit == NULL && asks == 1 && pose_asks == 1 && strlen (pose_seen[0].host_id) == lengths[i] && memcmp (pose_seen[0].host_id, id, lengths[i] + 1) == 0 && strlen (seen.host_id) == lengths[i] && memcmp (seen.host_id, id, lengths[i] + 1) == 0);
		}

		memset (id, 'x', sizeof (id));
		id[CAMPAIGN_MODEL_HOST_ID_MAX + 1] = '\0';
		e.host_id = id;
		pose_reset ();
		queue_reset ();
		queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
		o = offer (&e, FALSE, -500.0f);
		EXPECT (t, o.stopped && strstr (fatal_text, "entity_pose.request") != NULL && asks == 0 && pose_asks == 0);
	}

	/* a host that refuses stops the engine by the callback marker, before any candidate question */
	e = make_entity (TRUE, FALSE, 7);
	e.host_id = "eech-world:12";
	pose_reset ();
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
	refuse_the_exchange = TRUE;
	o = offer (&e, FALSE, -500.0f);
	refuse_the_exchange = FALSE;
	EXPECT (t, o.stopped && strstr (fatal_text, "callback") != NULL && pose_asks == 1);
}

static void scan_cases (struct tc *t);

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

	/* the placement is legitimate for exactly three branches, each with its scene and its stage facts: a mobile candidate, a building under a city (whose flags are read DURING the placement, so its
	   fixed facts were never read), and an ordinary fixed candidate whose fixed facts say it is one */
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0, SCENE_NAME);
	make_facts (more, TRUE, FALSE, FALSE, FALSE, FALSE, FALSE);
	more[CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] = 0.0f;
	o = ask_candidate (7, more);
	EXPECT (t, !o.stopped && o.state == CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT);
	make_facts (more, FALSE, FALSE, FALSE, FALSE, TRUE, FALSE);
	more[CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] = 0.0f;
	more[CAMPAIGN_CANDIDATE_FIXED_GIVEN] = 0.0f;
	o = ask_candidate (9, more);
	EXPECT (t, !o.stopped && o.state == CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT);
	make_facts (more, FALSE, TRUE, FALSE, FALSE, FALSE, FALSE);
	more[CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] = 0.0f;
	o = ask_candidate (7, more);
	EXPECT (t, !o.stopped && o.state == CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT);

	/* and for no other: a city, a building listed directly in a sector, an entity the fixed facts say is neither mobile nor fixed, and a candidate that has no scene are refused BEFORE any getter */
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0, "");
	make_facts (more, FALSE, TRUE, TRUE, TRUE, FALSE, FALSE);
	more[CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] = 0.0f;
	expect_candidate_refusal (t, "placement asked of a city", "entity_candidate.state", 0, more, __LINE__);
	make_facts (more, FALSE, TRUE, FALSE, FALSE, FALSE, FALSE);
	more[CAMPAIGN_CANDIDATE_CITY_BUILDING] = 1.0f;
	more[CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] = 0.0f;
	expect_candidate_refusal (t, "placement asked of a building listed directly in a sector", "entity_candidate.state", 0, more, __LINE__);
	/* the role alone refuses a building listed directly in a sector even when a scene is named for it (a city has none, which the request refuses first) */
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0, SCENE_NAME);
	make_facts (more, FALSE, TRUE, FALSE, FALSE, FALSE, FALSE);
	more[CAMPAIGN_CANDIDATE_CITY_BUILDING] = 1.0f;
	more[CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] = 0.0f;
	expect_candidate_refusal (t, "placement asked of a building listed directly in a sector that was given a scene", "entity_candidate.state", 7, more, __LINE__);
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0, SCENE_NAME);
	make_facts (more, FALSE, FALSE, FALSE, FALSE, FALSE, FALSE);
	more[CAMPAIGN_CANDIDATE_FIXED_GIVEN] = 1.0f;
	more[CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] = 0.0f;
	expect_candidate_refusal (t, "placement asked of an entity that is neither mobile nor fixed", "entity_candidate.state", 7, more, __LINE__);
	script_candidate (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0, "");
	make_facts (more, FALSE, TRUE, FALSE, FALSE, FALSE, FALSE);
	more[CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] = 0.0f;
	expect_candidate_refusal (t, "placement asked of an ordinary fixed candidate that has no scene", "entity_candidate.state", 0, more, __LINE__);
	make_facts (more, TRUE, FALSE, FALSE, FALSE, FALSE, FALSE);
	more[CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] = 0.0f;
	expect_candidate_refusal (t, "placement asked of a mobile candidate that has no scene", "entity_candidate.state", 0, more, __LINE__);
	make_facts (more, FALSE, FALSE, FALSE, FALSE, TRUE, FALSE);
	more[CAMPAIGN_CANDIDATE_PLACEMENT_GIVEN] = 0.0f;
	more[CAMPAIGN_CANDIDATE_FIXED_GIVEN] = 0.0f;
	expect_candidate_refusal (t, "placement asked of a building under a city that has no scene", "entity_candidate.state", 0, more, __LINE__);

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

	pose_cases (t);

	scan_cases (t);
}


/* ---- the enumeration: the campaign's scan driver (mb_hostscan.h) against scripted sector lists and a scripted host, with the real exchange and the real offer ---- */

static void scan_reset (void)
{
	memset (sectors, 0, sizeof (sectors));
	scan_queue_length = 0;
	scan_queue_next = 0;
	scan_cycle = FALSE;
	scan_seen_count = 0;
	collision_hit = FALSE;
	scan_key_forced = NULL;
	scan_stray_name = NULL;
	scripted_served_key = scripted_serve;
}

static uint32_t float_word (float value)
{
	uint32_t word;

	memcpy (&word, &value, sizeof (word));

	return (word);
}

static void scan_state (int state, float x, float z)
{
	scan_queue[scan_queue_length].state = state;
	scan_queue[scan_queue_length].cursor[0] = float_word (x);
	scan_queue[scan_queue_length].cursor[1] = float_word (z);
	scan_queue_length++;
}

static void scan_state_words (int state, uint32_t x, uint32_t z)
{
	scan_queue[scan_queue_length].state = state;
	scan_queue[scan_queue_length].cursor[0] = x;
	scan_queue[scan_queue_length].cursor[1] = z;
	scan_queue_length++;
}

/* the entities of a sector, head first; the scan reads the list from its tail */
static void sector_list (int x, int z, float tallest, struct sentity **entities, int count)
{
	int i;

	sectors[x][z].tallest = tallest;
	sectors[x][z].first_child = count ? entities[0] : NULL;

	for (i = 0; i < count; i++)
	{
		static int next_key = 1;

		entities[i]->succ = (i + 1 < count) ? entities[i + 1] : NULL;
		entities[i]->pred = (i > 0) ? entities[i - 1] : NULL;

		/* the host's roster issued a key at the insertion, never twice: an entity keeps the key it has */
		if (!entities[i]->key[0])
		{
			snprintf (entities[i]->key, sizeof (entities[i]->key), "%d", next_key++);
		}
	}
}

struct scan_outcome
{
	int stopped;
	struct sentity *hit;
};

static struct scan_outcome run_scan (struct sentity *source, struct sentity *target)
{
	struct scan_outcome o;
	float point[3], normal[3];

	memset (&o, 0, sizeof (o));
	memset (point, 0, sizeof (point));
	memset (normal, 0, sizeof (normal));
	reads[0] = '\0';
	collision_calls = 0;
	asks = 0;
	scan_seen_count = 0;
	fatal_text[0] = '\0';

	if (setjmp (fatal_jump) == 0)
	{
		o.hit = campaign_scan_entities (source, target, SEGMENT_OLD, SEGMENT_NEW, point, normal);
	}
	else
	{
		o.stopped = TRUE;
	}

	return (o);
}

static void expect_scan_refusal (struct tc *t, const char *label, const char *marker, int line)
{
	struct scan_outcome o;
	char what[160];

	scan_queue_next = 0;
	o = run_scan (NULL, NULL);

	snprintf (what, sizeof (what), "%s: stopped", label);
	tc_expect (t, o.stopped, what, line);
	snprintf (what, sizeof (what), "%s: stopped at %s (said: %.100s)", label, marker, fatal_text);
	tc_expect (t, strstr (fatal_text, marker) != NULL, what, line);
	snprintf (what, sizeof (what), "%s: nothing was offered or tested", label);
	tc_expect (t, o.hit == NULL && collision_calls == 0, what, line);
}

static void scan_cases (struct tc *t)
{
	struct sentity a, b, c, list[3];
	struct sentity *entities[3];

	/* ---- positives first: the host's order, tail first, the first hit ends the scan, an unnamed sector is never read ---- */
	scan_reset ();
	a = make_entity (TRUE, FALSE, 7);
	b = make_entity (TRUE, FALSE, 7);
	c = make_entity (TRUE, FALSE, 7);
	a.id = 1;
	b.id = 2;
	c.id = 3;
	entities[0] = &a;
	entities[1] = &b;
	sector_list (0, 0, 55.0f, entities, 2);
	entities[0] = &c;
	sector_list (0, 1, 55.0f, entities, 1);
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_OFFER, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_NEXT, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_OFFER, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_DONE_HIT, 0.0f, 0.0f);
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_TEST, 12);
	collision_hit = TRUE;
	{
		struct scan_outcome o = run_scan (NULL, NULL);

		/* b (the tail) is offered first and misses, then a, which hits; the sector (0, 1) is never read; the tallest height is read once, with the sector */
		EXPECT (t, !o.stopped && o.hit == &a && collision_calls == 1);
		EXPECT (t, strcmp (reads, "thssMSPVpMSPVA") == 0);
		EXPECT (t, scan_seen_count == 5);
		EXPECT (t, scan_seen[0][0] == 0.0f && scan_seen[1][0] == 1.0f && scan_seen[2][0] == 3.0f && scan_seen[3][0] == 1.0f && scan_seen[4][0] == 3.0f);
		EXPECT (t, scan_seen[2][3] == 0.0f && scan_seen[4][3] == 1.0f);
		/* the question that follows a served entry names it by the host's key and the others name none: the host serves what comes AFTER the entry it is told the campaign is at */
		EXPECT (t, scan_identity_seen[0][0] == 0 && strcmp (scan_identity_seen[1], b.key) == 0 && strcmp (scan_identity_seen[2], b.key) == 0);
		EXPECT (t, strcmp (scan_identity_seen[3], a.key) == 0 && strcmp (scan_identity_seen[4], a.key) == 0 && strcmp (a.key, b.key) != 0);
	}

	/* the source and the target are skipped by the host on the campaign's fact, and only the entity heap compared them: b is the source, a the target, so nothing is offered in that sector */
	scan_reset ();
	entities[0] = &a;
	entities[1] = &b;
	sector_list (0, 0, 55.0f, entities, 2);
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_NEXT, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_NEXT, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_DONE_NONE, 0.0f, 0.0f);
	queue_reset ();
	{
		struct scan_outcome o = run_scan (&b, &a);

		EXPECT (t, !o.stopped && o.hit == NULL && collision_calls == 0 && strstr (reads, "M") == NULL);
		EXPECT (t, strcmp (reads, "thsspp") == 0);
		EXPECT (t, scan_seen_count == 4 && scan_seen[1][2] == 1.0f && scan_seen[2][2] == 1.0f && scan_seen[3][0] == 2.0f);
	}

	/* an empty list is served as the end of that list, and the host goes on to the next sector it names (z inner, then x outer): the lists of sectors it does not name are not read */
	scan_reset ();
	entities[0] = &c;
	sector_list (1, 0, 10.0f, entities, 1);
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.0f, 2.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 1.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_OFFER, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_NEXT, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_DONE_NONE, 0.0f, 0.0f);
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
	{
		struct scan_outcome o = run_scan (NULL, NULL);

		EXPECT (t, !o.stopped && o.hit == NULL && collision_calls == 0);
		EXPECT (t, strcmp (reads, "thth" "sMSPVp") == 0 || strcmp (reads, "thth" "MSPVp") == 0);
		EXPECT (t, scan_seen_count == 5 && scan_seen[1][0] == 2.0f && scan_seen[2][0] == 1.0f && scan_seen[2][1] == 1.0f);
	}

	/* ---- the exact boundary refusals, each at the host's answer and before anything is offered or tested ---- */
	scan_reset ();
	queue_reset ();
	entities[0] = &a;
	sector_list (0, 0, 55.0f, entities, 1);

	scan_state (CAMPAIGN_MODEL_SCAN_OFFER, 0.0f, 0.0f);
	expect_scan_refusal (t, "an entry offered at the start of a scan", "entity_scan.state", __LINE__);
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_NEXT, 0.0f, 0.0f);
	expect_scan_refusal (t, "the next entry at the start of a scan", "entity_scan.state", __LINE__);
	scan_queue_length = 0;
	scan_state (0, 0.0f, 0.0f);
	expect_scan_refusal (t, "state 0", "entity_scan.state", __LINE__);
	scan_queue_length = 0;
	scan_state (6, 0.0f, 0.0f);
	expect_scan_refusal (t, "state 6", "entity_scan.state", __LINE__);

	/* a served entry that is not excluded must be offered, and an excluded one skipped */
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_NEXT, 0.0f, 0.0f);
	expect_scan_refusal (t, "a served entity that is not excluded, skipped", "entity_scan.state", __LINE__);
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_DONE_NONE, 0.0f, 0.0f);
	expect_scan_refusal (t, "the scan ended with an entity still served", "entity_scan.state", __LINE__);
	{
		struct scan_outcome o;

		scan_queue_length = 0;
		scan_queue_next = 0;
		scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.0f, 0.0f);
		scan_state (CAMPAIGN_MODEL_SCAN_OFFER, 0.0f, 0.0f);
		o = run_scan (&a, NULL);
		EXPECT (t, o.stopped && strstr (fatal_text, "entity_scan.state") != NULL && collision_calls == 0 && strstr (reads, "M") == NULL);
	}

	/* the end of a list goes to the next sector or ends the scan: nothing else */
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 2.0f, 2.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_NEXT, 0.0f, 0.0f);
	expect_scan_refusal (t, "the next entry at the end of a list", "entity_scan.state", __LINE__);

	/* after an offer: a hit ends the scan, a miss goes on */
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_OFFER, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_NEXT, 0.0f, 0.0f);
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_NEED_PLACEMENT, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_TEST, 12);
	collision_hit = TRUE;
	{
		struct scan_outcome o;

		scan_queue_next = 0;
		o = run_scan (NULL, NULL);
		EXPECT (t, o.stopped && strstr (fatal_text, "entity_scan.state") != NULL && collision_calls == 1);
	}
	collision_hit = FALSE;
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_OFFER, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_DONE_HIT, 0.0f, 0.0f);
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
	{
		struct scan_outcome o;

		scan_queue_next = 0;
		o = run_scan (NULL, NULL);
		EXPECT (t, o.stopped && strstr (fatal_text, "entity_scan.state") != NULL && collision_calls == 0);
	}

	/* the cursor: a sector outside the map is refused by the campaign before any list is touched; a cursor that is not whole numbers inside 0 to 16777216, not finite (a signalling NaN included), or
	   that rides on a state that carries none, is refused by the exchange from its raw words */
	queue_reset ();
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 3.0f, 0.0f);
	expect_scan_refusal (t, "a sector x outside the map", "entity_scan.cursor", __LINE__);
	EXPECT (t, strcmp (reads, "") == 0);
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.0f, 3.0f);
	expect_scan_refusal (t, "a sector z outside the map", "entity_scan.cursor", __LINE__);
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.5f, 0.0f);
	expect_scan_refusal (t, "a fractional sector", "entity_scan.cursor", __LINE__);
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, -1.0f, 0.0f);
	expect_scan_refusal (t, "a negative sector", "entity_scan.cursor", __LINE__);
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 1.0e9f, 0.0f);
	expect_scan_refusal (t, "a sector beyond 16777216", "entity_scan.cursor", __LINE__);
	scan_queue_length = 0;
	scan_state_words (CAMPAIGN_MODEL_SCAN_FIRST, 0x7f800000u, 0);
	expect_scan_refusal (t, "an infinite sector", "entity_scan.cursor", __LINE__);
	scan_queue_length = 0;
	scan_state_words (CAMPAIGN_MODEL_SCAN_FIRST, 0, 0x7fc00000u);
	expect_scan_refusal (t, "a quiet NaN sector", "entity_scan.cursor", __LINE__);
	scan_queue_length = 0;
	scan_state_words (CAMPAIGN_MODEL_SCAN_FIRST, 0x7fa00001u, 0);
	expect_scan_refusal (t, "a signalling NaN sector", "entity_scan.cursor", __LINE__);
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_DONE_NONE, 1.0f, 0.0f);
	expect_scan_refusal (t, "a sector named with a state that carries none", "entity_scan.cursor", __LINE__);

	/* a refusal of the host and a host that never ends the scan */
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.0f, 0.0f);
	refuse_the_exchange = TRUE;
	expect_scan_refusal (t, "the host refuses the exchange", "model-provider:callback", __LINE__);
	refuse_the_exchange = FALSE;
	/* an empty sector, so that the host's first-sector answer is legal at the end of its list every time */
	sector_list (0, 0, 0.0f, entities, 0);
	scan_cycle = TRUE;
	expect_scan_refusal (t, "a host that never ends the scan", "entity_scan.runaway", __LINE__);
	scan_cycle = FALSE;

	/* ---- the roster is the answer and the campaign's own list only its mirror: an entry the list does not agree with, or a key that names nothing, is refused BY NAME and is never an end, a miss or a fallback ---- */
	scan_reset ();
	queue_reset ();
	a = make_entity (TRUE, FALSE, 7);
	b = make_entity (TRUE, FALSE, 7);
	entities[0] = &a;
	entities[1] = &b;
	sector_list (0, 0, 55.0f, entities, 2);
	sector_list (0, 1, 55.0f, entities, 0);

	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.0f, 0.0f);
	scan_key_forced = a.key;
	expect_scan_refusal (t, "the host serves the head of the list where the list's tail is", "entity_scan.roster_mirror", __LINE__);
	scan_key_forced = "";
	expect_scan_refusal (t, "the host serves the end of a list that has entries", "entity_scan.roster_mirror", __LINE__);
	scan_key_forced = "999";
	expect_scan_refusal (t, "the host serves a key that names no entity", "entity_scan.roster_key", __LINE__);
	scan_key_forced = "999";
	/* the campaign's own list was read the way the original read it (the sector, its tallest, its first child and the walk to the end) and the mirror was never asked for a predecessor */
	EXPECT (t, strcmp (reads, "thss") == 0);
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.0f, 1.0f);
	scan_key_forced = a.key;
	expect_scan_refusal (t, "the host serves an entry where the list is empty", "entity_scan.roster_mirror", __LINE__);

	/* a key that is not a roster key is refused by the exchange before any entity is resolved */
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.0f, 0.0f);
	scan_key_forced = "007";
	expect_scan_refusal (t, "a key with a leading zero", "entity_scan.key", __LINE__);
	scan_key_forced = "12x";
	expect_scan_refusal (t, "a key that is not a number", "entity_scan.key", __LINE__);
	scan_key_forced = "123456789012345678901";
	expect_scan_refusal (t, "a key of 21 digits", "entity_scan.key", __LINE__);
	scan_key_forced = "-5";
	expect_scan_refusal (t, "a negative key", "entity_scan.key", __LINE__);
	scan_key_forced = NULL;
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_OFFER, 0.0f, 0.0f);
	scan_stray_name = a.key;
	expect_scan_refusal (t, "a key on an answer that serves no entry", "entity_scan.key", __LINE__);
	scan_stray_name = NULL;

	/* the coherent host: the same roster, served in its order through the real driver, with the mirror agreeing at every step */
	scan_queue_length = 0;
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_OFFER, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_NEXT, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_OFFER, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_NEXT, 0.0f, 0.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_FIRST, 0.0f, 1.0f);
	scan_state (CAMPAIGN_MODEL_SCAN_DONE_NONE, 0.0f, 0.0f);
	queue_reset ();
	queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
	queue_state (CAMPAIGN_MODEL_CANDIDATE_SKIP, 0);
	{
		struct scan_outcome o;

		scan_queue_next = 0;
		o = run_scan (NULL, NULL);
		EXPECT (t, !o.stopped && o.hit == NULL && scan_seen_count == 7);
		EXPECT (t, strcmp (scan_identity_seen[2], b.key) == 0 && strcmp (scan_identity_seen[4], a.key) == 0 && scan_identity_seen[5][0] == 0 && scan_identity_seen[6][0] == 0);
	}
}

/* the roster's events: one insertion, removal or reset of the sector lists, with the real exchange and a scripted host */
static int roster_event (int phase, int entity_type, const float *position, const char *identity, char *key)
{
	roster_asks = 0;
	key[0] = 0;
	fatal_text[0] = '\0';

	if (setjmp (fatal_jump) == 0)
	{
		campaign_entity_roster_exchange (phase, entity_type, 100.0f, position, identity, key);

		return (FALSE);
	}

	return (TRUE);
}

static void roster_cases (struct tc *t)
{
	static const float position[3] = { 120.5f, -0.0f, 270.25f };
	char key[CAMPAIGN_ROSTER_KEY_MAX + 1], identity[CAMPAIGN_MODEL_HOST_ID_MAX + 2];
	int stopped;

	roster_override = FALSE;
	roster_issue = 1;
	refuse_the_exchange = FALSE;

	/* an insertion states the entity's type, the sector side length and the position the campaign holds, bit for bit (a negative zero included), with a hosted mobile's identity; the host's key comes back */
	stopped = roster_event (CAMPAIGN_SCAN_PHASE_INSERT, 27, position, "eech-world:7", key);
	EXPECT (t, !stopped && roster_asks == 1 && strcmp (key, "1") == 0);
	EXPECT (t, roster_seen.op == CAMPAIGN_MODEL_ENTITY_SCAN && roster_seen.scene == 0 && roster_seen.pose_count == CAMPAIGN_MODEL_SCAN_FACTS);
	EXPECT (t, memcmp (&roster_seen.pose[CAMPAIGN_ROSTER_POSITION], position, 3 * sizeof (float)) == 0);
	EXPECT (t, roster_seen.pose[CAMPAIGN_ROSTER_TYPE] == 27.0f && roster_seen.pose[CAMPAIGN_SCAN_SIDE] == 100.0f && (int) roster_seen.pose[CAMPAIGN_SCAN_PHASE] == CAMPAIGN_SCAN_PHASE_INSERT);
	EXPECT (t, strcmp (roster_seen.host_id, "eech-world:7") == 0);

	/* every other entity names no identity, and a hosted mobile's whole domain of 128 bytes crosses unshortened */
	stopped = roster_event (CAMPAIGN_SCAN_PHASE_INSERT, 4, position, NULL, key);
	EXPECT (t, !stopped && roster_seen.host_id[0] == 0 && strcmp (key, "2") == 0);
	memset (identity, 'z', CAMPAIGN_MODEL_HOST_ID_MAX);
	identity[CAMPAIGN_MODEL_HOST_ID_MAX] = 0;
	stopped = roster_event (CAMPAIGN_SCAN_PHASE_INSERT, 4, position, identity, key);
	EXPECT (t, !stopped && strcmp (roster_seen.host_id, identity) == 0 && strlen (roster_seen.host_id) == CAMPAIGN_MODEL_HOST_ID_MAX);
	identity[CAMPAIGN_MODEL_HOST_ID_MAX] = 'z';
	identity[CAMPAIGN_MODEL_HOST_ID_MAX + 1] = 0;
	stopped = roster_event (CAMPAIGN_SCAN_PHASE_INSERT, 4, position, identity, key);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.request") != NULL && roster_asks == 0);

	/* a removal names the key and a reset names nothing; neither carries a position or a type */
	stopped = roster_event (CAMPAIGN_SCAN_PHASE_REMOVE, 4, NULL, "1", key);
	EXPECT (t, !stopped && roster_asks == 1 && key[0] == 0 && strcmp (roster_seen.host_id, "1") == 0 && (int) roster_seen.pose[CAMPAIGN_SCAN_PHASE] == CAMPAIGN_SCAN_PHASE_REMOVE);
	stopped = roster_event (CAMPAIGN_SCAN_PHASE_RESET, 0, NULL, NULL, key);
	EXPECT (t, !stopped && roster_asks == 1 && key[0] == 0 && roster_seen.host_id[0] == 0 && (int) roster_seen.pose[CAMPAIGN_SCAN_PHASE] == CAMPAIGN_SCAN_PHASE_RESET);
	stopped = roster_event (CAMPAIGN_SCAN_PHASE_REMOVE, 4, NULL, "007", key);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.key") != NULL && roster_asks == 0);
	stopped = roster_event (CAMPAIGN_SCAN_PHASE_REMOVE, 4, NULL, NULL, key);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.key") != NULL && roster_asks == 0);

	/* the answer is checked completely: the state the phase allows, a key for an insertion and for no other event, a key that is a roster key, no sector */
	roster_override = TRUE;
	roster_state = CAMPAIGN_MODEL_SCAN_PLACED;
	roster_name = "";
	stopped = roster_event (CAMPAIGN_SCAN_PHASE_INSERT, 4, position, NULL, key);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.key") != NULL);
	roster_name = "4x";
	stopped = roster_event (CAMPAIGN_SCAN_PHASE_INSERT, 4, position, NULL, key);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.key") != NULL);
	roster_state = CAMPAIGN_MODEL_SCAN_REMOVED;
	roster_name = "5";
	stopped = roster_event (CAMPAIGN_SCAN_PHASE_INSERT, 4, position, NULL, key);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.state") != NULL);
	roster_state = CAMPAIGN_MODEL_SCAN_PLACED;
	stopped = roster_event (CAMPAIGN_SCAN_PHASE_REMOVE, 4, NULL, "5", key);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.state") != NULL);
	roster_state = CAMPAIGN_MODEL_SCAN_REMOVED;
	stopped = roster_event (CAMPAIGN_SCAN_PHASE_REMOVE, 4, NULL, "5", key);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.key") != NULL);
	roster_state = CAMPAIGN_MODEL_SCAN_CLEARED;
	roster_name = "";
	stopped = roster_event (CAMPAIGN_SCAN_PHASE_INSERT, 4, position, NULL, key);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.state") != NULL);
	roster_override = FALSE;

	refuse_the_exchange = TRUE;
	stopped = roster_event (CAMPAIGN_SCAN_PHASE_INSERT, 4, position, NULL, key);
	EXPECT (t, stopped && strstr (fatal_text, "model-provider:callback") != NULL);
	refuse_the_exchange = FALSE;
}


/* ---- the campaign's placement table (mb_hostplacemap.h): the REAL code that keeps the host's key for each entity of a sector list, bound to scripted entities and the scripted host ---- */

struct pentity
{
	int slot, type, has_position;
	const char *name, *identity, *object;
	float position[3];
};

static struct pentity pheap[16];
static int pheap_size = 16, place_host_installed;

static int place_heap_check (void)
{
	if ((pheap_size <= 0) || (pheap_size > 16))
	{
		debug_fatal ("host roster: the entity heap holds %d slots, which is not a heap this run allocated", pheap_size);
	}

	return (pheap_size);
}

static int check_host_installed (void)
{
	return (place_host_installed);
}

static void check_roster_call (int phase, int entity_type, float side, const float *position, const char *identity, char *key)
{
	campaign_entity_roster_exchange (phase, entity_type, side, position, identity, key);
}

static void check_city_call (int phase, int entity_type, const char *identity, char *key)
{
	campaign_entity_city_exchange (phase, entity_type, identity, key);
}

#define campaign_model_host_installed check_host_installed
#define campaign_model_entity_roster check_roster_call
#define campaign_model_entity_city check_city_call
#define CAMPAIGN_PLACE_ENTITY struct pentity
#define CAMPAIGN_PLACE_HEAP() place_heap_check ()
#define CAMPAIGN_PLACE_SLOT(E) ((E)->slot)
#define CAMPAIGN_PLACE_AT(SLOT) (&pheap[(SLOT)])
#define CAMPAIGN_PLACE_TYPE(E) ((E)->type)
#define CAMPAIGN_PLACE_NAME(E) ((E)->name)
#define CAMPAIGN_PLACE_POSITION(E) ((E)->has_position ? (E)->position : NULL)
#define CAMPAIGN_PLACE_IDENTITY(E) ((E)->identity)
#define CAMPAIGN_PLACE_OBJECT_ID(E) ((E)->object)
#define CAMPAIGN_PLACE_SIDE() 100.0f

#include "entity/mobile/mb_hostplacemap.h"

#undef campaign_model_host_installed
#undef campaign_model_entity_roster
#undef campaign_model_entity_city

/* the entity of the offer that holds a placement heap slot (set by city_register) */
static struct sentity *pheap_owner[16];

/* the offer's city traversal bound to the REAL placement map: the host (the scripted lists) serves the first building and the one after each; the building it serves is resolved by its key and compared with the
   scripted entity's own list at the same place; the scripted entities' reads are logged as the real getters' are (the mirror read of the first child is the 'B') */
static struct pentity *so_place_of (struct sentity *e)
{
	return (e ? &pheap[e->pslot] : NULL);
}

static struct sentity *so_city_first (struct sentity *city)
{
	struct pentity *served;

	read_log ('B');

	served = place_child_first (so_place_of (city), so_place_of (city->first_building));

	return (served ? pheap_owner[served->slot] : NULL);
}

static struct sentity *so_city_next (struct sentity *city, struct sentity *building)
{
	struct pentity *served;

	served = place_child_next (so_place_of (city), so_place_of (building), so_place_of (building->next_building));

	return (served ? pheap_owner[served->slot] : NULL);
}

/* a scripted city and its buildings, placed in the host's lists by the real placement map in the order that gives the host the list the entity script states (head first: the buildings are listed from the last
   to the first, each at the head). The city holds a sector placement (the host names its list by it), the buildings none. Returns FALSE when the map stopped the engine. */
static int city_register (struct tc *t, struct sentity *city, struct sentity *b1, struct sentity *b2)
{
	struct sentity *entity[3];
	struct sentity *listed[2];
	int i, n = 0;

	entity[0] = city;
	entity[1] = b1;
	entity[2] = b2;

	for (i = 0; i < 3; i++)
	{
		if (entity[i])
		{
			struct pentity *p = &pheap[10 + i];

			memset (p, 0, sizeof (*p));
			p->slot = 10 + i;
			p->type = 40 + i;
			p->name = "SCRIPTED";
			p->has_position = TRUE;
			p->position[0] = 100.0f + (float) i;
			entity[i]->pslot = 10 + i;
			pheap_owner[10 + i] = entity[i];
		}
	}

	if (b2)
	{
		listed[n++] = b2;
	}

	if (b1)
	{
		listed[n++] = b1;
	}

	place_reset ();
	place_host_installed = TRUE;
	city_count = 0;
	roster_override = FALSE;
	city_override = FALSE;
	fatal_text[0] = '\0';

	if (setjmp (fatal_jump) == 0)
	{
		place_insert (&pheap[city->pslot]);

		for (i = 0; i < n; i++)
		{
			place_child_insert (&pheap[listed[i]->pslot], &pheap[city->pslot], NULL);
		}

		return (TRUE);
	}

	tc_expect (t, FALSE, "a scripted city could not be registered with the host", __LINE__);

	return (FALSE);
}

/* one operation of the table under a stop catcher: TRUE when the engine stopped; the host's questions of the operation are counted in roster_asks */
static int place_do (int what, struct pentity *e)
{
	roster_asks = 0;
	fatal_text[0] = '\0';

	if (setjmp (fatal_jump) == 0)
	{
		switch (what)
		{
			case 0: place_insert (e); break;
			case 1: place_remove (e); break;
			case 2: place_release_entity (e); break;
			default: place_reset (); break;
		}

		return (FALSE);
	}

	return (TRUE);
}

static void place_cases (struct tc *t)
{
	int i, stopped, ok;
	char key[24];

	memset (pheap, 0, sizeof (pheap));

	for (i = 0; i < 16; i++)
	{
		pheap[i].slot = i;
		pheap[i].type = 20 + i;
		pheap[i].name = "SCRIPTED";
		pheap[i].has_position = TRUE;
		pheap[i].position[0] = 1000.0f + (float) i;
		pheap[i].position[1] = 5.0f;
		pheap[i].position[2] = 2000.0f;
	}

	pheap[0].identity = "eech-world:7";
	roster_override = FALSE;
	roster_issue = 1;
	refuse_the_exchange = FALSE;
	pheap_size = 16;
	place_reset ();
	place_host_installed = FALSE;

	/* a build with no model provider has no roster: nothing is asked, nothing is kept, and nothing resolves */
	stopped = place_do (0, &pheap[0]);
	EXPECT (t, !stopped && roster_asks == 0 && place_entity ("1") == NULL);
	stopped = place_do (1, &pheap[0]);
	EXPECT (t, !stopped && roster_asks == 0);
	stopped = place_do (2, &pheap[0]);
	EXPECT (t, !stopped && roster_asks == 0);

	place_host_installed = TRUE;

	/* an insertion tells the host the entity's type, the sector side length, the position it holds and a hosted mobile's identity; the key the host issued resolves to the entity, and only to it */
	stopped = place_do (0, &pheap[0]);
	EXPECT (t, !stopped && roster_asks == 1 && (int) roster_seen.pose[CAMPAIGN_SCAN_PHASE] == CAMPAIGN_SCAN_PHASE_INSERT);
	EXPECT (t, roster_seen.pose[CAMPAIGN_ROSTER_TYPE] == 20.0f && roster_seen.pose[CAMPAIGN_SCAN_SIDE] == 100.0f && memcmp (&roster_seen.pose[CAMPAIGN_ROSTER_POSITION], pheap[0].position, 3 * sizeof (float)) == 0);
	EXPECT (t, strcmp (roster_seen.host_id, "eech-world:7") == 0);
	EXPECT (t, place_entity ("1") == &pheap[0] && place_entity ("2") == NULL);
	stopped = place_do (0, &pheap[1]);
	EXPECT (t, !stopped && roster_seen.host_id[0] == 0 && place_entity ("2") == &pheap[1] && place_entity ("1") == &pheap[0]);

	/* a removal names the key the host issued and the key resolves to nothing afterwards; the other key still resolves (the table's probe sequence is unbroken by a removed entry) */
	stopped = place_do (1, &pheap[0]);
	EXPECT (t, !stopped && roster_asks == 1 && (int) roster_seen.pose[CAMPAIGN_SCAN_PHASE] == CAMPAIGN_SCAN_PHASE_REMOVE && strcmp (roster_seen.host_id, "1") == 0);
	EXPECT (t, place_entity ("1") == NULL && place_entity ("2") == &pheap[1]);

	/* a relink is a removal and an insertion: the entity has a NEW key and the old one is not reissued */
	stopped = place_do (0, &pheap[0]);
	EXPECT (t, !stopped && place_entity ("3") == &pheap[0] && place_entity ("1") == NULL);

	/* an entity still placed when its heap slot is released is removed first; one that is not placed asks nothing */
	stopped = place_do (2, &pheap[1]);
	EXPECT (t, !stopped && roster_asks == 1 && strcmp (roster_seen.host_id, "2") == 0 && place_entity ("2") == NULL);
	stopped = place_do (2, &pheap[1]);
	EXPECT (t, !stopped && roster_asks == 0);
	stopped = place_do (2, &pheap[9]);
	EXPECT (t, !stopped && roster_asks == 0);

	/* what cannot be right stops the engine by name and asks the host nothing: an insertion twice without a removal, a removal of an entity that was never placed, an entity with no position, a heap that is not one */
	stopped = place_do (0, &pheap[0]);
	EXPECT (t, stopped && strstr (fatal_text, "still holds it") != NULL && roster_asks == 0);
	stopped = place_do (1, &pheap[5]);
	EXPECT (t, stopped && strstr (fatal_text, "never placed") != NULL && roster_asks == 0);
	pheap[5].has_position = FALSE;
	stopped = place_do (0, &pheap[5]);
	EXPECT (t, stopped && strstr (fatal_text, "no position") != NULL && roster_asks == 0);
	pheap[5].has_position = TRUE;
	pheap[5].slot = 16;
	stopped = place_do (0, &pheap[5]);
	EXPECT (t, stopped && strstr (fatal_text, "outside the entity heap") != NULL && roster_asks == 0);
	pheap[5].slot = 5;
	pheap_size = 0;
	stopped = place_do (0, &pheap[5]);
	EXPECT (t, stopped && strstr (fatal_text, "not a heap") != NULL && roster_asks == 0);
	pheap_size = 16;

	/* a host that issues a key it issued before is refused, and the entity is not recorded under it */
	roster_override = TRUE;
	roster_state = CAMPAIGN_MODEL_SCAN_PLACED;
	roster_name = "3";
	stopped = place_do (0, &pheap[5]);
	EXPECT (t, stopped && strstr (fatal_text, "second time") != NULL && place_entity ("3") == &pheap[0]);
	roster_override = FALSE;

	/* churn: thousands of relinks of one entity beside three that stay never lose a key, and the table does not grow with the keys the host has issued (it holds the live ones) */
	for (i = 0; i < 6000; i++)
	{
		stopped = place_do (0, &pheap[8]);

		if (stopped)
		{
			break;
		}

		stopped = place_do (1, &pheap[8]);

		if (stopped)
		{
			break;
		}
	}

	EXPECT (t, !stopped && place_entity ("3") == &pheap[0] && place_table_size <= 8192 && place_table_live == 1);
	ok = TRUE;
	stopped = place_do (0, &pheap[8]);
	snprintf (key, sizeof (key), "%d", roster_issue - 1);
	ok = ok && !stopped && (place_entity (key) == &pheap[8]);
	EXPECT (t, ok && place_table_live == 2);

	/* a new boot ends every key, asks the host to end its roster, and the table is usable again */
	stopped = place_do (3, NULL);
	EXPECT (t, !stopped && roster_asks == 1 && (int) roster_seen.pose[CAMPAIGN_SCAN_PHASE] == CAMPAIGN_SCAN_PHASE_RESET);
	EXPECT (t, place_entity ("3") == NULL && place_entity (key) == NULL);
	stopped = place_do (0, &pheap[2]);
	snprintf (key, sizeof (key), "%d", roster_issue - 1);
	EXPECT (t, !stopped && place_entity (key) == &pheap[2]);

	/* a host that refuses stops the engine with its reason and leaves the entity unplaced */
	place_reset ();
	refuse_the_exchange = TRUE;
	stopped = place_do (0, &pheap[3]);
	EXPECT (t, stopped && strstr (fatal_text, "model-provider:callback") != NULL);
	refuse_the_exchange = FALSE;
	stopped = place_do (1, &pheap[3]);
	EXPECT (t, stopped && strstr (fatal_text, "never placed") != NULL);

	place_reset ();
	place_host_installed = FALSE;
}

/* ---- the city lists: the real placement map and the real city exchange against the scripted host (phases 7 to 12) ---- */

/* one operation of the map under a stop catcher: TRUE when the engine stopped; the host's questions of the operation are counted in city_asks (and roster_asks for the sector events) */
static int city_do (int what, struct pentity *e, struct pentity *parent, struct pentity *pred)
{
	city_asks = 0;
	roster_asks = 0;
	fatal_text[0] = '\0';

	if (setjmp (fatal_jump) == 0)
	{
		switch (what)
		{
			case 0: place_child_insert (e, parent, pred); break;
			case 1: place_child_remove (e); break;
			case 2: place_release_entity (e); break;
			case 3: place_insert (e); break;
			case 4: place_remove (e); break;
			default: place_name (e, e->object); break;
		}

		return (FALSE);
	}

	return (TRUE);
}

/* what the host serves for the first building of a city (what 0) or the one after `building` (what 1), compared with `mirror`: TRUE when the engine stopped */
static int city_serve (int what, struct pentity *city, struct pentity *building, struct pentity *mirror, struct pentity **served)
{
	city_asks = 0;
	fatal_text[0] = '\0';
	*served = NULL;

	if (setjmp (fatal_jump) == 0)
	{
		*served = (what == 0) ? place_child_first (city, mirror) : place_child_next (city, building, mirror);

		return (FALSE);
	}

	return (TRUE);
}

/* a request of the city exchange itself, with the host's questions counted: TRUE when the engine stopped */
static int city_exchange_do (int phase, int entity_type, const char *identity, char *key)
{
	city_asks = 0;
	fatal_text[0] = '\0';
	key[0] = 0;

	if (setjmp (fatal_jump) == 0)
	{
		campaign_entity_city_exchange (phase, entity_type, identity, key);

		return (FALSE);
	}

	return (TRUE);
}

static void city_cases (struct tc *t)
{
	struct pentity *city = &pheap[1], *a = &pheap[2], *b = &pheap[3], *c = &pheap[4], *bare = &pheap[5], *loose = &pheap[6], *d = &pheap[7], *served;
	char key[160], pair[160];
	int i, stopped, kind;

	memset (pheap, 0, sizeof (pheap));

	for (i = 0; i < 16; i++)
	{
		pheap[i].slot = i;
		pheap[i].type = 20 + i;
		pheap[i].name = "SCRIPTED";
		pheap[i].has_position = TRUE;
		pheap[i].position[0] = 1000.0f + (float) i;
		pheap[i].position[2] = 2000.0f;
		pheap_owner[i] = NULL;
	}

	pheap_size = 16;
	roster_override = FALSE;
	roster_issue = 1;
	city_override = FALSE;
	city_count = 0;
	refuse_the_exchange = FALSE;
	place_reset ();
	place_host_installed = TRUE;

	/* the city holds its sector placement (the host names its list by that key); its buildings are listed under it, the newest at the head, a predecessor puts the new one right after it */
	stopped = city_do (3, city, NULL, NULL);
	EXPECT (t, !stopped && place_entity ("1") == city && city_asks == 0);
	stopped = city_do (0, a, city, NULL);
	EXPECT (t, !stopped && city_asks == 1 && roster_asks == 0 && (int) city_seen.pose[CAMPAIGN_SCAN_PHASE] == CAMPAIGN_SCAN_PHASE_CHILD_INSERT && strcmp (city_seen.host_id, "1") == 0);
	EXPECT (t, city_seen.pose[CAMPAIGN_ROSTER_TYPE] == (float) a->type && place_find ("2", &kind) == a->slot && kind == 1);
	EXPECT (t, place_entity ("2") == NULL);
	stopped = city_do (0, b, city, NULL);
	EXPECT (t, !stopped && strcmp (city_seen.host_id, "1") == 0 && place_find ("3", &kind) == b->slot && kind == 1);
	stopped = city_do (0, c, city, a);
	EXPECT (t, !stopped && strcmp (city_seen.host_id, "1 2") == 0 && place_find ("4", &kind) == c->slot && kind == 1);

	/* the host's order is the answer: b, a, c. The first building is asked by the city's key; the next by the city's and the building's; the campaign's own list at the same place is only the mirror */
	stopped = city_serve (0, city, NULL, b, &served);
	EXPECT (t, !stopped && served == b && city_asks == 1 && (int) city_seen.pose[CAMPAIGN_SCAN_PHASE] == CAMPAIGN_SCAN_PHASE_CHILD_FIRST && strcmp (city_seen.host_id, "1") == 0);
	stopped = city_serve (1, city, b, a, &served);
	EXPECT (t, !stopped && served == a && (int) city_seen.pose[CAMPAIGN_SCAN_PHASE] == CAMPAIGN_SCAN_PHASE_CHILD_NEXT && strcmp (city_seen.host_id, "1 3") == 0);
	stopped = city_serve (1, city, a, c, &served);
	EXPECT (t, !stopped && served == c && strcmp (city_seen.host_id, "1 2") == 0);
	stopped = city_serve (1, city, c, NULL, &served);
	EXPECT (t, !stopped && served == NULL && city_asks == 1);

	/* a disagreement is a named stop and never an end, a miss or a fallback: the campaign's list is at another building, at its end, or has one where the host serves the end */
	stopped = city_serve (0, city, NULL, a, &served);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.city_mirror") != NULL && served == NULL);
	stopped = city_serve (0, city, NULL, NULL, &served);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.city_mirror") != NULL);
	stopped = city_serve (1, city, c, b, &served);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.city_mirror") != NULL && strstr (fatal_text, "end of a city's list") != NULL);

	/* a key that names no building of the campaign is a named stop, whether it is a sector placement's key, a key the campaign never held or an ended one */
	city_override = TRUE;
	city_state = CAMPAIGN_MODEL_SCAN_NEXT;
	city_name = "1";
	stopped = city_serve (0, city, NULL, b, &served);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.city_key") != NULL);
	city_name = "99";
	stopped = city_serve (0, city, NULL, b, &served);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.city_key") != NULL);
	city_name = "01";
	stopped = city_serve (0, city, NULL, b, &served);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.key") != NULL);
	city_override = FALSE;

	/* a city with no sector placement has no list the host could name: nothing is asked */
	stopped = city_do (0, d, bare, NULL);
	EXPECT (t, stopped && strstr (fatal_text, "holds no sector placement") != NULL && city_asks == 0);
	stopped = city_serve (0, bare, NULL, NULL, &served);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.city_parent") != NULL && city_asks == 0);
	stopped = city_serve (1, bare, a, NULL, &served);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.city_parent") != NULL && city_asks == 0);

	/* what cannot be right stops the engine by name and asks the host nothing: a predecessor the host does not list, an entity listed twice, an entity that was never listed */
	stopped = city_do (0, d, city, loose);
	EXPECT (t, stopped && strstr (fatal_text, "not listed by the host") != NULL && city_asks == 0);
	stopped = city_do (0, a, city, NULL);
	EXPECT (t, stopped && strstr (fatal_text, "still holds it") != NULL && city_asks == 0);
	stopped = city_do (1, d, NULL, NULL);
	EXPECT (t, stopped && strstr (fatal_text, "never listed") != NULL && city_asks == 0);

	/* a removal names the city's key recorded at the insertion and the building's own, and the key names nothing afterwards (the others are served in order) */
	stopped = city_do (1, b, NULL, NULL);
	EXPECT (t, !stopped && city_asks == 1 && (int) city_seen.pose[CAMPAIGN_SCAN_PHASE] == CAMPAIGN_SCAN_PHASE_CHILD_REMOVE && strcmp (city_seen.host_id, "1 3") == 0);
	EXPECT (t, place_find ("3", &kind) < 0);
	stopped = city_serve (0, city, NULL, a, &served);
	EXPECT (t, !stopped && served == a);

	/* the same entity in two lists is declared one entity as soon as the second placement exists, in either order: a creation lists the building and then places it in its sector, and the other way round */
	stopped = city_do (3, a, NULL, NULL);
	EXPECT (t, !stopped && roster_asks == 1 && city_asks == 1 && (int) city_seen.pose[CAMPAIGN_SCAN_PHASE] == CAMPAIGN_SCAN_PHASE_LINK);
	snprintf (pair, sizeof (pair), "%s 2", place_key_of_slot[a->slot]);
	EXPECT (t, strcmp (city_last_link, pair) == 0);
	stopped = city_do (3, d, NULL, NULL);
	EXPECT (t, !stopped && city_asks == 0);
	stopped = city_do (0, d, city, NULL);
	EXPECT (t, !stopped && city_asks == 2 && (int) city_seen.pose[CAMPAIGN_SCAN_PHASE] == CAMPAIGN_SCAN_PHASE_LINK);
	snprintf (pair, sizeof (pair), "%s %s", place_key_of_slot[d->slot], place_child_of_slot[d->slot]);
	EXPECT (t, strcmp (city_last_link, pair) == 0);

	/* the object-report identity is named once it is issued (lazily): by the sector placement's key, or by the city child's when that is all the entity has; an entity with no placement, and an empty identity, ask nothing */
	a->object = "eech-world:object:7";
	stopped = city_do (5, a, NULL, NULL);
	snprintf (pair, sizeof (pair), "%s eech-world:object:7", place_key_of_slot[a->slot]);
	EXPECT (t, !stopped && city_asks == 1 && strcmp (city_last_name, pair) == 0 && (int) city_seen.pose[CAMPAIGN_SCAN_PHASE] == CAMPAIGN_SCAN_PHASE_NAME);
	c->object = "eech-world:object:8";
	stopped = city_do (5, c, NULL, NULL);
	snprintf (pair, sizeof (pair), "%s eech-world:object:8", place_child_of_slot[c->slot]);
	EXPECT (t, !stopped && city_asks == 1 && strcmp (city_last_name, pair) == 0);
	loose->object = "eech-world:object:9";
	stopped = city_do (5, loose, NULL, NULL);
	EXPECT (t, !stopped && city_asks == 0);
	a->object = "";
	stopped = city_do (5, a, NULL, NULL);
	EXPECT (t, !stopped && city_asks == 0);
	a->object = "eech-world:object:7";

	/* a relink of a structure that holds its object identity: with a city child the record is declared again (the child's record keeps the name), without one the new placement is named; neither allocates anything */
	stopped = city_do (4, a, NULL, NULL);
	EXPECT (t, !stopped && roster_asks == 1);
	stopped = city_do (3, a, NULL, NULL);
	EXPECT (t, !stopped && roster_asks == 1 && city_asks == 1 && (int) city_seen.pose[CAMPAIGN_SCAN_PHASE] == CAMPAIGN_SCAN_PHASE_LINK);
	loose->has_position = TRUE;
	stopped = city_do (3, loose, NULL, NULL);
	EXPECT (t, !stopped && city_asks == 1 && (int) city_seen.pose[CAMPAIGN_SCAN_PHASE] == CAMPAIGN_SCAN_PHASE_NAME);
	snprintf (pair, sizeof (pair), "%s eech-world:object:9", place_key_of_slot[loose->slot]);
	EXPECT (t, strcmp (city_last_name, pair) == 0);

	/* a heap slot released while still listed and placed removes the city child first, then the sector placement, by the keys recorded; one that holds neither asks nothing */
	stopped = city_do (2, a, NULL, NULL);
	EXPECT (t, !stopped && city_asks == 1 && roster_asks == 1 && place_child_of_slot[a->slot][0] == 0 && place_key_of_slot[a->slot][0] == 0);
	stopped = city_do (2, a, NULL, NULL);
	EXPECT (t, !stopped && city_asks == 0 && roster_asks == 0);

	/* the host's replies are checked completely: the state the phase allows, a key for an insertion and for no removal, the key it echoes for a LINK and a NAME, a key that is a key */
	city_override = TRUE;
	city_state = CAMPAIGN_MODEL_SCAN_PLACED;
	city_name = "";
	stopped = city_do (0, b, city, NULL);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.key") != NULL && strstr (fatal_text, "issued no key") != NULL && place_child_of_slot[b->slot][0] == 0);
	city_state = CAMPAIGN_MODEL_SCAN_REMOVED;
	city_name = "5";
	stopped = city_do (0, b, city, NULL);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.state") != NULL);
	city_state = CAMPAIGN_MODEL_SCAN_PLACED;
	city_name = "9x";
	stopped = city_do (0, b, city, NULL);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.key") != NULL);
	city_override = FALSE;
	stopped = city_do (0, b, city, NULL);
	EXPECT (t, !stopped);
	city_override = TRUE;
	city_state = CAMPAIGN_MODEL_SCAN_REMOVED;
	city_name = "77";
	stopped = city_do (1, b, NULL, NULL);
	EXPECT (t, stopped && strstr (fatal_text, "entity_scan.key") != NULL && strstr (fatal_text, "issued a key") != NULL);
	city_override = FALSE;

	/* the grammar of the identity place is checked BEFORE anything is sent: one key where two are needed, two where one is, a key that is not a key, an empty or control-character object identity, one that does not fit the field */
	stopped = city_exchange_do (CAMPAIGN_SCAN_PHASE_CHILD_NEXT, 0, "1", key);
	EXPECT (t, stopped && strstr (fatal_text, "needs two") != NULL && city_asks == 0);
	stopped = city_exchange_do (CAMPAIGN_SCAN_PHASE_CHILD_FIRST, 0, "1 2", key);
	EXPECT (t, stopped && strstr (fatal_text, "names the city only") != NULL && city_asks == 0);
	stopped = city_exchange_do (CAMPAIGN_SCAN_PHASE_CHILD_FIRST, 0, "01", key);
	EXPECT (t, stopped && strstr (fatal_text, "not a roster key") != NULL && city_asks == 0);
	stopped = city_exchange_do (CAMPAIGN_SCAN_PHASE_CHILD_NEXT, 0, "1 +2", key);
	EXPECT (t, stopped && strstr (fatal_text, "not a roster key") != NULL && city_asks == 0);
	stopped = city_exchange_do (CAMPAIGN_SCAN_PHASE_NAME, 0, "1 ", key);
	EXPECT (t, stopped && strstr (fatal_text, "empty object identity") != NULL && city_asks == 0);
	stopped = city_exchange_do (CAMPAIGN_SCAN_PHASE_NAME, 0, "1 eech\nobject", key);
	EXPECT (t, stopped && strstr (fatal_text, "control character") != NULL && city_asks == 0);
	{
		char longest[160], toolong[160];

		memset (longest, 'x', sizeof (longest));
		memset (toolong, 'x', sizeof (toolong));
		longest[128] = 0;
		toolong[129] = 0;
		longest[0] = '1';
		longest[1] = ' ';
		toolong[0] = '1';
		toolong[1] = ' ';
		stopped = city_exchange_do (CAMPAIGN_SCAN_PHASE_NAME, 0, longest, key);
		EXPECT (t, !stopped && city_asks == 1);
		stopped = city_exchange_do (CAMPAIGN_SCAN_PHASE_NAME, 0, toolong, key);
		EXPECT (t, stopped && strstr (fatal_text, "identity field") != NULL && city_asks == 0);
	}

	/* a host that refuses stops the engine with its reason and leaves the entity unlisted */
	refuse_the_exchange = TRUE;
	stopped = city_do (0, loose, city, NULL);
	EXPECT (t, stopped && strstr (fatal_text, "model-provider:callback") != NULL && place_child_of_slot[loose->slot][0] == 0);
	refuse_the_exchange = FALSE;

	/* a new boot ends every key of both kinds: the city's list names nothing afterwards */
	stopped = city_do (3, a, NULL, NULL);
	place_reset ();
	EXPECT (t, place_find ("1", &kind) < 0 && place_find ("2", &kind) < 0 && place_child_of_slot == NULL);

	place_host_installed = FALSE;
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

	roster_cases (&t);

	place_cases (&t);
	city_cases (&t);

	if (checks)
	{
		*checks = t.checks;
	}

	return (t.failures);
}

#endif
