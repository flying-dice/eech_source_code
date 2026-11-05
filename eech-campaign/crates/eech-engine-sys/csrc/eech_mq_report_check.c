/*
 * TEST-ONLY (EEDCS-49, D3, feature `mq-report-check`): the isolated control of the controlled moving ENTITY QUERY case's report codec and bounds (eech_mq_report.h), the SAME bytes the
 * `weapon-fixture` case (eech_wpn_fixture.c) formats. It loads no model, boots no engine and asks no query: it proves the finite part of the case that a plain C function can prove.
 *   - the identity encoding is reversible, canonical, whitespace-free and bounded for EVERY roster identity (1 to 128 bytes of any value, newlines, spaces and UTF-8 included);
 *   - a HIT row with a 128-byte identity is longer than the previous 320-byte local buffer (the defect this replaces) and fits its report;
 *   - the worst-case report length is computed from the formatters themselves and the capacity predicate accepts exactly the buffers that hold it, so the case refuses before its first query.
 * The order of those checks inside the engine entry (before the first production query) is by construction (the first statement of both entries) and is exercised natively by Root.
 * eech_mq_report_check_run returns the number of failed expectations (0: all held) and writes one line per failure; *checks is the number of expectations made.
 */
#ifdef EECH_MQ_REPORT_CHECK

#include "eech_mq_report.h"

static int mq_check_failures, mq_check_count;
static char *mq_check_text;
static int mq_check_capacity;

#define MQ_EXPECT(condition) \
	do \
	{ \
		mq_check_count++; \
		if (!(condition)) \
		{ \
			mq_check_failures++; \
			(void) mq_note (__LINE__, #condition); \
		} \
	} while (0)

static int mq_note (int line, const char *what)
{
	size_t used = strlen (mq_check_text);

	if (used + 1 < (size_t) mq_check_capacity)
	{
		snprintf (mq_check_text + used, (size_t) mq_check_capacity - used, "line %d: %s\n", line, what);
	}
	return 0;
}

/* the test-only inverse of mq_encode_id: -1 unless the text is a canonical encoding */
static int mq_check_decode (const char *encoded, char *out, size_t capacity)
{
	size_t used = 0, k = 0, length = strlen (encoded);

	while (k < length)
	{
		unsigned char c = (unsigned char) encoded[k];

		if (used + 1 >= capacity)
		{
			return -1;
		}
		if (c == '%')
		{
			unsigned value = 0;
			int d;

			if (k + 3 > length)
			{
				return -1;
			}
			for (d = 1; d <= 2; d++)
			{
				unsigned char h = (unsigned char) encoded[k + d];

				value <<= 4;
				if (h >= '0' && h <= '9')
				{
					value |= (unsigned) (h - '0');
				}
				else if (h >= 'A' && h <= 'F')
				{
					value |= (unsigned) (h - 'A' + 10);
				}
				else
				{
					return -1;
				}
			}
			out[used++] = (char) value;
			k += 3;
		}
		else if (mq_plain_byte (c))
		{
			out[used++] = (char) c;
			k++;
		}
		else
		{
			return -1;
		}
	}
	out[used] = 0;
	{
		/* canonical only: the decoded identity must encode back to exactly this text (`%41` for `A`, lower-case hexadecimal and an embedded NUL are not) */
		char again[MQ_ID_ENCODED_MAX + 1];

		if (strlen (out) != used || mq_encode_id (out, again, sizeof (again)) < 0 || strcmp (again, encoded) != 0)
		{
			return -1;
		}
	}
	return (int) used;
}

/* an id of `length` bytes drawn from a repeating cycle that includes space, tab, newline, carriage return, `%`, `-`, `=` and UTF-8 bytes */
static void mq_check_id (char *id, size_t length, int seed)
{
	static const unsigned char cycle[] = { ' ', '\n', 'a', '\t', '%', 0xc3, 0xa9, '-', '=', '\r', 'Z', 0xe2, 0x82, 0xac, '~', '.', '_', '9' };
	size_t k;

	for (k = 0; k < length; k++)
	{
		id[k] = (char) cycle[(k + (size_t) seed) % sizeof (cycle)];
	}
	id[length] = 0;
}

static void mq_check_encoding (void)
{
	char id[MQ_ID_MAX + 2], encoded[MQ_ID_ENCODED_MAX + 1], decoded[MQ_ID_MAX + 2], small[8];
	size_t length;
	int seed, n, byte, k;

	/* plain bytes stay, everything else (including `-`, space, newline, `%` and UTF-8) is escaped */
	MQ_EXPECT (mq_encode_id ("abcXYZ019._~", encoded, sizeof (encoded)) == 12 && strcmp (encoded, "abcXYZ019._~") == 0);
	MQ_EXPECT (mq_encode_id ("-", encoded, sizeof (encoded)) == 3 && strcmp (encoded, "%2D") == 0);
	MQ_EXPECT (mq_encode_id (" ", encoded, sizeof (encoded)) == 3 && strcmp (encoded, "%20") == 0);
	MQ_EXPECT (mq_encode_id ("a\nb", encoded, sizeof (encoded)) == 5 && strcmp (encoded, "a%0Ab") == 0);
	MQ_EXPECT (mq_encode_id ("%", encoded, sizeof (encoded)) == 3 && strcmp (encoded, "%25") == 0);
	MQ_EXPECT (mq_encode_id ("\xc3\xa9", encoded, sizeof (encoded)) == 6 && strcmp (encoded, "%C3%A9") == 0);
	/* the lone `-` of an absent identity is never an encoded identity */
	for (byte = 1; byte < 256; byte++)
	{
		char one[2];

		one[0] = (char) byte;
		one[1] = 0;
		n = mq_encode_id (one, encoded, sizeof (encoded));
		MQ_EXPECT (n >= 1 && strcmp (encoded, "-") != 0);
		MQ_EXPECT (mq_check_decode (encoded, decoded, sizeof (decoded)) == 1 && decoded[0] == (char) byte);
		for (k = 0; k < n; k++)
		{
			if ((unsigned char) encoded[k] <= ' ' || (unsigned char) encoded[k] > '~')
			{
				MQ_EXPECT (0);
			}
		}
	}
	/* the domain: empty and 129 bytes refuse, 1 and 128 bytes encode */
	id[0] = 0;
	MQ_EXPECT (mq_encode_id (id, encoded, sizeof (encoded)) == -1);
	mq_check_id (id, MQ_ID_MAX + 1, 0);
	MQ_EXPECT (mq_encode_id (id, encoded, sizeof (encoded)) == -1);
	/* every length from 1 to the maximum round-trips exactly, for several byte mixes; the encoded text never exceeds its bound */
	for (seed = 0; seed < 5; seed++)
	{
		for (length = 1; length <= MQ_ID_MAX; length++)
		{
			mq_check_id (id, length, seed);
			n = mq_encode_id (id, encoded, sizeof (encoded));
			MQ_EXPECT (n >= (int) length && n <= MQ_ID_ENCODED_MAX);
			MQ_EXPECT (mq_check_decode (encoded, decoded, sizeof (decoded)) == (int) length && memcmp (decoded, id, length) == 0);
		}
	}
	/* the longest case: every byte escaped */
	memset (id, '\n', MQ_ID_MAX);
	id[MQ_ID_MAX] = 0;
	MQ_EXPECT (mq_encode_id (id, encoded, sizeof (encoded)) == MQ_ID_ENCODED_MAX);
	/* the capacity of the destination is exact: the encoded length plus the NUL fits, one byte less does not */
	MQ_EXPECT (mq_encode_id ("a\nb", small, 6) == 5);
	MQ_EXPECT (mq_encode_id ("a\nb", small, 5) == -1);
	/* a non-canonical or malformed encoding is not decoded */
	MQ_EXPECT (mq_check_decode ("%41", decoded, sizeof (decoded)) == -1);
	MQ_EXPECT (mq_check_decode ("%c3%a9", decoded, sizeof (decoded)) == -1);
	MQ_EXPECT (mq_check_decode ("%00", decoded, sizeof (decoded)) == -1);
	MQ_EXPECT (mq_check_decode ("%C3%A9", decoded, sizeof (decoded)) == 2);
	MQ_EXPECT (mq_check_decode ("%4", decoded, sizeof (decoded)) == -1);
	MQ_EXPECT (mq_check_decode ("%zz", decoded, sizeof (decoded)) == -1);
	MQ_EXPECT (mq_check_decode ("a b", decoded, sizeof (decoded)) == -1);
}

static void mq_check_rows (void)
{
	static char row[MQ_REPORT_SCRATCH];
	char id[MQ_ID_MAX + 1], encoded[MQ_ID_ENCODED_MAX + 1], decoded[MQ_ID_MAX + 2];
	const uint32_t a[3] = { 0x3f800000u, 0xc0000000u, 0x7fc00001u }, b[3] = { 0x80000000u, 0x00000001u, 0xffffffffu };
	const uint32_t point[3] = { 0x11111111u, 0x22222222u, 0x33333333u }, normal[3] = { 0x44444444u, 0x55555555u, 0x66666666u };
	const char *miss = "wpn-moving-query-result seg=1 hit=0 a=0x3f800000,0xc0000000,0x7fc00001 b=0x80000000,0x00000001,0xffffffff hit_is_target=0 hit_type=none hit_host_id=- point=- normal=-\n";
	int n, newlines;
	size_t k;

	/* the POSITIVE control: a 128-byte identity mixing whitespace, newlines and UTF-8, a HIT with the six output words */
	mq_check_id (id, MQ_ID_MAX, 3);
	MQ_EXPECT (mq_encode_id (id, encoded, sizeof (encoded)) > 0);
	n = mq_format_result (row, sizeof (row), 2, 1, a, b, 1, "ENTITY_TYPE_ROUTED_VEHICLE", encoded, point, normal);
	/* longer than the previous 320-byte local buffer and than the 378-byte row the review computed for the raw identity */
	MQ_EXPECT (n > 378 && n < (int) sizeof (row));
	MQ_EXPECT (strlen (row) == (size_t) n && row[n - 1] == '\n');
	newlines = 0;
	for (k = 0; k < (size_t) n; k++)
	{
		newlines += row[k] == '\n';
	}
	MQ_EXPECT (newlines == 1);
	MQ_EXPECT (strstr (row, " hit_host_id=") != NULL && strstr (row, encoded) != NULL);
	MQ_EXPECT (strstr (row, "point=0x11111111,0x22222222,0x33333333 normal=0x44444444,0x55555555,0x66666666\n") != NULL);
	MQ_EXPECT (mq_check_decode (encoded, decoded, sizeof (decoded)) == MQ_ID_MAX && memcmp (decoded, id, MQ_ID_MAX) == 0);
	/* the DEFECT this replaces, characterised: the same row with the RAW identity (what the previous case interpolated) is longer than its 320-byte buffer and splits into several lines */
	n = mq_format_result (row, sizeof (row), 2, 1, a, b, 1, "ENTITY_TYPE_ROUTED_VEHICLE", id, point, normal);
	newlines = 0;
	for (k = 0; k < (size_t) n; k++)
	{
		newlines += row[k] == '\n';
	}
	MQ_EXPECT (n > 320 && newlines > 1);
	/* a hit whose entity has no roster identity carries `-` (never an identity: `-` itself is encoded) */
	n = mq_format_result (row, sizeof (row), 0, 1, a, b, 0, "ENTITY_TYPE_CITY_BUILDING", NULL, point, normal);
	MQ_EXPECT (n > 0 && strstr (row, " hit_is_target=0 hit_type=ENTITY_TYPE_CITY_BUILDING hit_host_id=- point=0x11111111") != NULL);
	/* the MISS row is exactly the miss grammar and carries no output word */
	n = mq_format_result (row, sizeof (row), 1, 0, a, b, 1, "ENTITY_TYPE_ROUTED_VEHICLE", encoded, point, normal);
	MQ_EXPECT (n == (int) strlen (miss) && strcmp (row, miss) == 0);
	/* a destination that is one byte short refuses (never a silent truncation) */
	n = mq_format_result (row, sizeof (row), 1, 0, a, b, 0, "none", NULL, point, normal);
	MQ_EXPECT (n > 0 && mq_format_result (row, (size_t) n, 1, 0, a, b, 0, "none", NULL, point, normal) == -1);
	MQ_EXPECT (mq_format_result (row, (size_t) n + 1, 1, 0, a, b, 0, "none", NULL, point, normal) == n);
	/* the header, segment and error records */
	n = mq_format_run_header (row, sizeof (row), "a%0Ab");
	MQ_EXPECT (n > 0 && strcmp (row, "wpn-moving-query-run host_id=a%0Ab segments=3\n") == 0);
	n = mq_format_prepare_header (row, sizeof (row), "a%0Ab", "ENTITY_TYPE_ROUTED_VEHICLE", 0x41200000u);
	MQ_EXPECT (n > 0 && strcmp (row, "wpn-moving-query-prepare host_id=a%0Ab segments=3 target_type=ENTITY_TYPE_ROUTED_VEHICLE radius=0x41200000\n") == 0);
	n = mq_format_segment (row, sizeof (row), 0, 0x3f060a92u, a, b);
	MQ_EXPECT (n > 0 && strcmp (row, "wpn-moving-query-segment seg=0 bearing=0x3f060a92 a=0x3f800000,0xc0000000,0x7fc00001 b=0x80000000,0x00000001,0xffffffff\n") == 0);
	n = mq_format_error (row, sizeof (row), "target_not_alive", "a%0Ab");
	MQ_EXPECT (n > 0 && strcmp (row, "wpn-case-error reason=target_not_alive host_id=a%0Ab\n") == 0);
}

static void mq_check_bounds (void)
{
	int worst = mq_worst_report_length (MQ_TYPE_NAME_MAX), longest_real = mq_worst_report_length (27);

	/* the worst case is computed (not estimated), is finite, and holds three rows of the maximal identity */
	MQ_EXPECT (worst > 0 && longest_real > 0 && longest_real <= worst);
	MQ_EXPECT (worst > 3 * 378);
	MQ_EXPECT (worst + 1 < MQ_REPORT_SCRATCH);
	/* the capacity predicate accepts exactly the buffers that hold it, for the longest compiled type name (27 bytes) and for the declared maximum */
	MQ_EXPECT (mq_capacity_holds (longest_real + 1, 27) && !mq_capacity_holds (longest_real, 27));
	MQ_EXPECT (mq_capacity_holds (worst + 1, MQ_TYPE_NAME_MAX) && !mq_capacity_holds (worst, MQ_TYPE_NAME_MAX));
	/* the 4096-byte buffer the Rust bridge passes holds it; the previous 3072 minimum was not proved to */
	MQ_EXPECT (mq_capacity_holds (4096, MQ_TYPE_NAME_MAX));
	MQ_EXPECT (!mq_capacity_holds (0, 27) && !mq_capacity_holds (-1, 27) && !mq_capacity_holds (64, 27));
	/* a type name longer than the report allows is refused, never truncated */
	MQ_EXPECT (mq_worst_report_length (MQ_TYPE_NAME_MAX + 1) == -1 && !mq_capacity_holds (4096, MQ_TYPE_NAME_MAX + 1));
}

int eech_mq_report_check_run (char *text, int capacity, int *checks)
{
	mq_check_failures = 0;
	mq_check_count = 0;
	mq_check_text = text;
	mq_check_capacity = capacity;
	if (capacity > 0)
	{
		text[0] = 0;
	}
	mq_check_encoding ();
	mq_check_rows ();
	mq_check_bounds ();
	*checks = mq_check_count;
	return mq_check_failures;
}

#endif
