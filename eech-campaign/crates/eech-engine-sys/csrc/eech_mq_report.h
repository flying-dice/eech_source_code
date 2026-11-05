/*
 * TEST-ONLY (EEDCS-49, D3): the report grammar and bounds of the controlled moving ENTITY QUERY case (eech_wpn_fixture.c, feature `weapon-fixture`) as plain C, so the SAME bytes are
 * produced by the fixture and by its isolated control (eech_mq_report_check.c, feature `mq-report-check`). It has no engine type and no state, and it is not a model or a solver.
 *
 * A report is lines of `record key=value ...` terminated by a newline. A semantic host identity is 1 to MQ_ID_MAX bytes of ANY value (the roster's domain: whitespace, newlines and
 * UTF-8 included), so it is carried as an ENCODED identity: each byte that is not an ASCII letter, digit or one of `._~` becomes `%XX` (upper-case hexadecimal), so the encoding is
 * reversible, canonical (one text per identity) and holds no whitespace. `-` is always encoded, so the lone `-` of an absent identity is never an identity. The encoded identity is at most
 * MQ_ID_ENCODED_MAX bytes. The worst-case length of every report is computed by formatting a maximal sample through the very formatters below (never a separate estimate), and the case
 * refuses BEFORE its first query when the caller's buffer cannot hold it.
 */
#ifndef EECH_MQ_REPORT_H
#define EECH_MQ_REPORT_H

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define MQ_SEGMENTS 3
#define MQ_ID_MAX 128
#define MQ_ID_ENCODED_MAX (3 * MQ_ID_MAX)
/* the longest entity type name the report may carry (the compiled names are `ENTITY_TYPE_...` without whitespace) */
#define MQ_TYPE_NAME_MAX 40
/* the largest report any case entry produces, with every field at its limit, is far below this; it is also the buffer the Rust bridge passes */
#define MQ_REPORT_SCRATCH 4096

/* 1 when the byte is written as itself */
static int mq_plain_byte (unsigned char byte)
{
	return (byte >= 'A' && byte <= 'Z') || (byte >= 'a' && byte <= 'z') || (byte >= '0' && byte <= '9') || byte == '.' || byte == '_' || byte == '~';
}

/* the encoded identity of a 1 to MQ_ID_MAX byte text into `out` (capacity `capacity` including the NUL); the encoded length, or -1 when the domain or the capacity is exceeded */
static int mq_encode_id (const char *id, char *out, size_t capacity)
{
	static const char digits[] = "0123456789ABCDEF";
	size_t length = strlen (id), used = 0, k;

	if (length < 1 || length > MQ_ID_MAX)
	{
		return -1;
	}
	for (k = 0; k < length; k++)
	{
		unsigned char byte = (unsigned char) id[k];
		size_t need = mq_plain_byte (byte) ? 1 : 3;

		if (used + need + 1 > capacity)
		{
			return -1;
		}
		if (need == 1)
		{
			out[used++] = (char) byte;
		}
		else
		{
			out[used++] = '%';
			out[used++] = digits[byte >> 4];
			out[used++] = digits[byte & 15];
		}
	}
	out[used] = 0;
	return (int) used;
}

static int mq_print (char *out, size_t capacity, const char *format, ...)
{
	va_list args;
	int n;

	va_start (args, format);
	n = vsnprintf (out, capacity, format, args);
	va_end (args);
	return (n < 0 || (size_t) n >= capacity) ? -1 : n;
}

/* `wpn-moving-query-prepare` header: the encoded target, the segment count, the entity type and the radius word */
static int mq_format_prepare_header (char *out, size_t capacity, const char *encoded_id, const char *type_name, uint32_t radius)
{
	return mq_print (out, capacity, "wpn-moving-query-prepare host_id=%s segments=%d target_type=%s radius=0x%08x\n", encoded_id, MQ_SEGMENTS, type_name, (unsigned) radius);
}

static int mq_format_segment (char *out, size_t capacity, int seg, uint32_t bearing, const uint32_t a[3], const uint32_t b[3])
{
	return mq_print (out, capacity, "wpn-moving-query-segment seg=%d bearing=0x%08x a=0x%08x,0x%08x,0x%08x b=0x%08x,0x%08x,0x%08x\n", seg, (unsigned) bearing, (unsigned) a[0], (unsigned) a[1],
		(unsigned) a[2], (unsigned) b[0], (unsigned) b[1], (unsigned) b[2]);
}

static int mq_format_run_header (char *out, size_t capacity, const char *encoded_id)
{
	return mq_print (out, capacity, "wpn-moving-query-run host_id=%s segments=%d\n", encoded_id, MQ_SEGMENTS);
}

/* one result row. A MISS carries no output word and no identity (`hit_type=none`, `-` for the identity, point and normal); a HIT carries the three point and three normal words as printed
 * bit patterns, its entity type name and either the encoded roster identity of the hit or `-` when the hit entity has none */
static int mq_format_result (char *out, size_t capacity, int seg, int hit, const uint32_t a[3], const uint32_t b[3], int hit_is_target, const char *hit_type, const char *encoded_hit_id,
	const uint32_t point[3], const uint32_t normal[3])
{
	int n = mq_print (out, capacity, "wpn-moving-query-result seg=%d hit=%d a=0x%08x,0x%08x,0x%08x b=0x%08x,0x%08x,0x%08x", seg, hit ? 1 : 0, (unsigned) a[0], (unsigned) a[1], (unsigned) a[2],
		(unsigned) b[0], (unsigned) b[1], (unsigned) b[2]);
	int m;

	if (n < 0)
	{
		return -1;
	}
	if (hit)
	{
		m = mq_print (out + n, capacity - (size_t) n, " hit_is_target=%d hit_type=%s hit_host_id=%s point=0x%08x,0x%08x,0x%08x normal=0x%08x,0x%08x,0x%08x\n", hit_is_target ? 1 : 0, hit_type,
			encoded_hit_id ? encoded_hit_id : "-", (unsigned) point[0], (unsigned) point[1], (unsigned) point[2], (unsigned) normal[0], (unsigned) normal[1], (unsigned) normal[2]);
	}
	else
	{
		m = mq_print (out + n, capacity - (size_t) n, " hit_is_target=0 hit_type=none hit_host_id=- point=- normal=-\n");
	}
	return m < 0 ? -1 : n + m;
}

/* the case refusal: the NAMED reason and the encoded identity, in the buffer the engine entry returns with the non-poison argument status */
static int mq_format_error (char *out, size_t capacity, const char *reason, const char *encoded_id)
{
	return mq_print (out, capacity, "wpn-case-error reason=%s host_id=%s\n", reason, encoded_id);
}

/*
 * The worst-case length (without the NUL) of the PREPARE report and of the RUN report, by formatting a maximal sample through the formatters above: the longest encoded identity (every byte
 * escaped), the longest type name, every word 0xffffffff, the largest segment number, a HIT with a roster identity in every row. Returns the larger of the two, or -1 when the sample itself
 * does not fit MQ_REPORT_SCRATCH (a defect of the limits, never of the data).
 */
static int mq_worst_report_length (size_t type_name_max)
{
	static char scratch[MQ_REPORT_SCRATCH];
	char id[MQ_ID_MAX + 1], encoded[MQ_ID_ENCODED_MAX + 1], type[MQ_TYPE_NAME_MAX + 1];
	const uint32_t words[3] = { 0xffffffffu, 0xffffffffu, 0xffffffffu };
	size_t k;
	int prepare = 0, run = 0, n;

	if (type_name_max > MQ_TYPE_NAME_MAX)
	{
		return -1;
	}
	for (k = 0; k < MQ_ID_MAX; k++)
	{
		id[k] = '\n';
	}
	id[MQ_ID_MAX] = 0;
	for (k = 0; k < type_name_max; k++)
	{
		type[k] = 'T';
	}
	type[type_name_max] = 0;
	if (mq_encode_id (id, encoded, sizeof (encoded)) != MQ_ID_ENCODED_MAX)
	{
		return -1;
	}
	n = mq_format_prepare_header (scratch, sizeof (scratch), encoded, type, 0xffffffffu);
	if (n < 0)
	{
		return -1;
	}
	prepare = n;
	for (k = 0; k < MQ_SEGMENTS; k++)
	{
		n = mq_format_segment (scratch, sizeof (scratch), MQ_SEGMENTS - 1, 0xffffffffu, words, words);
		if (n < 0)
		{
			return -1;
		}
		prepare += n;
	}
	n = mq_format_run_header (scratch, sizeof (scratch), encoded);
	if (n < 0)
	{
		return -1;
	}
	run = n;
	for (k = 0; k < MQ_SEGMENTS; k++)
	{
		n = mq_format_result (scratch, sizeof (scratch), MQ_SEGMENTS - 1, 1, words, words, 1, type, encoded, words, words);
		if (n < 0)
		{
			return -1;
		}
		run += n;
	}
	return prepare > run ? prepare : run;
}

/* 1 when a caller's buffer of `capacity` bytes (including the NUL) holds every report this case can produce for entity type names of at most `type_name_max` bytes */
static int mq_capacity_holds (int capacity, size_t type_name_max)
{
	int worst = mq_worst_report_length (type_name_max);

	return worst >= 0 && capacity > 0 && worst + 1 <= capacity;
}

#endif
