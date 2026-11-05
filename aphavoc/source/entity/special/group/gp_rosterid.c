#include "project.h"

#include <string.h>

#include "gp_roster.h"

#include "entity/mobile/mb_hostobj.h"

/*
 * The identity the host returned for a new group member, accepted or refused BEFORE the roster commits the binding (EEDCS-46, R3).  It is the same checks campaign_roster_link_member always made
 * (a length of 1 to 128 bytes, UTF-8, not an identity the roster issued to any incarnation), in one place that does not depend on the roster's records, plus the one the structures' registry needs
 * (mb_hostobj.c): the host's opaque identities form ONE namespace that is never reused, so a member may not take an identity that was issued for a structure, live or ended.  The structures' registry
 * enforces the converse when it issues one.  `roster_has` says whether the roster already issued the identity (to any incarnation).
 */

static int valid_utf8 (const unsigned char *text, size_t length)
{
	size_t i = 0;
	while (i < length)
	{
		unsigned char first = text[i++];
		unsigned int codepoint;
		int remaining;
		if (first < 0x80)
		{
			continue;
		}
		if ((first >= 0xC2) && (first <= 0xDF))
		{
			codepoint = first & 0x1F;
			remaining = 1;
		}
		else if ((first >= 0xE0) && (first <= 0xEF))
		{
			codepoint = first & 0x0F;
			remaining = 2;
		}
		else if ((first >= 0xF0) && (first <= 0xF4))
		{
			codepoint = first & 0x07;
			remaining = 3;
		}
		else
		{
			return FALSE;
		}
		if (i + (size_t) remaining > length)
		{
			return FALSE;
		}
		while (remaining--)
		{
			unsigned char continuation = text[i++];
			if ((continuation & 0xC0) != 0x80)
			{
				return FALSE;
			}
			codepoint = (codepoint << 6) | (continuation & 0x3F);
		}
		if (((codepoint < 0x800) && (first >= 0xE0)) ||
			((codepoint < 0x10000) && (first >= 0xF0)) ||
			((codepoint >= 0xD800) && (codepoint <= 0xDFFF)) ||
			(codepoint > 0x10FFFF))
		{
			return FALSE;
		}
	}
	return TRUE;
}

void campaign_roster_accept_host_id (const char *host_id, int (*roster_has) (const char *host_id))
{
	size_t length = strnlen (host_id, CAMPAIGN_HOST_ID_MAX + 1);

	if ((length == 0) || (length > CAMPAIGN_HOST_ID_MAX))
	{
		debug_fatal ("campaign roster: host bind_mobile returned an invalid identity length");
	}
	if (!valid_utf8 ((const unsigned char *) host_id, length))
	{
		debug_fatal ("campaign roster: host bind_mobile returned a non-UTF-8 identity");
	}
	if (roster_has (host_id))
	{
		debug_fatal ("campaign roster: host identity %s was reused for a new physical incarnation", host_id);
	}
	if (campaign_object_id_ever_issued (host_id))
	{
		debug_fatal ("campaign roster: host identity %s was already issued for a structure", host_id);
	}
}
