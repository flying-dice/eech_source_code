// The campaign's report of the sector lists to the host's roster (EEDCS-49, D3, sixth slice): see mb_hostplace.h, mb_hostmodel.h (ENTITY_SCAN, the roster's events) and mb_hostplace.c.
//
// The host holds the roster: which entities stand in which sector, in what order (the head of a sector's list is its newest insertion and a scan visits the tail first), decided from ITS state.  The
// campaign tells it each insertion and removal and keeps, per entity, only the key the host issued for the entity's current placement, so that the entry the host serves can be resolved back to the campaign's
// entity for its decisions and its consequences.  A key is the host's: it is not an index of the entity heap (that is reused, a key never is), a relink is a removal and an insertion with a new key, and a key
// that was removed names nothing.
//
// What this does not do: it decides no sector (the host does, from the position it is told and, for a hosted mobile, from its own record of it), it does not order the entries, and it is not the answer of a
// scan: the campaign's list is only the mirror a scan compares the host's entry with (mb_hostscan.h).
//
// It is a header, included by mb_hostplace.c (bound to the real entity heap) and by the isolated control csrc/eech_entity_dialogue_check.c (bound to scripted entities and a scripted host), so the control
// exercises THIS code and not a copy.  The includer provides FALSE, TRUE, NULL, memcpy, strcmp, strlen, calloc, free, debug_fatal, campaign_model_host_installed and campaign_model_entity_roster
// (mb_hostmodel.h), and
//
//   CAMPAIGN_PLACE_ENTITY                  the entity type
//   CAMPAIGN_PLACE_HEAP ()                 the number of slots of the entity heap this run allocated (stops the engine for a heap that is not one)
//   CAMPAIGN_PLACE_SLOT (E)                the heap slot of an entity, and CAMPAIGN_PLACE_AT (SLOT) the entity in a slot
//   CAMPAIGN_PLACE_TYPE (E), CAMPAIGN_PLACE_NAME (E)   the entity's type (an int) and its type's name (text, for a refusal)
//   CAMPAIGN_PLACE_POSITION (E)            the entity's three position words, or NULL for an entity that has none
//   CAMPAIGN_PLACE_IDENTITY (E)            a hosted mobile's semantic host identity, or NULL for every other entity
//   CAMPAIGN_PLACE_SIDE ()                 the sector side length, as a float
#ifndef MB_HOSTPLACEMAP_H
#define MB_HOSTPLACEMAP_H

// the key the host issued for the placement of the entity in each heap slot ("" when it is not placed); the slots are those of the heap this run allocated
static char
	(*place_key_of_slot) [CAMPAIGN_ROSTER_KEY_MAX + 1] = NULL;

static int
	place_map_capacity = 0;

// the reverse map, by key: open addressing over slot numbers + 1 (0 empty, -1 a removed entry)
static int
	*place_table = NULL;

static size_t
	place_table_size = 0,
	place_table_used = 0,
	place_table_live = 0;

static int place_checked_slot (CAMPAIGN_PLACE_ENTITY *en)
{
	int
		index;

	index = CAMPAIGN_PLACE_SLOT (en);

	if ((index < 0) || (index >= CAMPAIGN_PLACE_HEAP ()))
	{
		debug_fatal ("host roster: entity %d is outside the entity heap of %d slots", index, CAMPAIGN_PLACE_HEAP ());
	}

	if (place_key_of_slot && (place_map_capacity != CAMPAIGN_PLACE_HEAP ()))
	{
		debug_fatal ("host roster: the entity heap changed from %d to %d slots while placements exist", place_map_capacity, CAMPAIGN_PLACE_HEAP ());
	}

	return (index);
}

static size_t place_hash (const char *key)
{
	// FNV-1a
	size_t hash = 2166136261u;

	while (*key)
	{
		hash ^= (unsigned char) *key++;

		hash *= 16777619u;
	}

	return (hash);
}

static void place_table_rebuild (size_t wanted)
{
	int
		slot;

	free (place_table);

	place_table = (int *) calloc (wanted, sizeof (int));

	if (!place_table)
	{
		debug_fatal ("host roster: cannot allocate a key table of %u entries", (unsigned int) wanted);
	}

	place_table_size = wanted;

	place_table_used = 0;

	place_table_live = 0;

	for (slot = 0; slot < place_map_capacity; slot++)
	{
		if (place_key_of_slot [slot][0])
		{
			size_t position = place_hash (place_key_of_slot [slot]) & (place_table_size - 1);

			while (place_table [position])
			{
				position = (position + 1) & (place_table_size - 1);
			}

			place_table [position] = slot + 1;

			place_table_used++;

			place_table_live++;
		}
	}
}

static void place_ensure_map (void)
{
	if (!place_key_of_slot)
	{
		place_map_capacity = CAMPAIGN_PLACE_HEAP ();

		place_key_of_slot = (char (*) [CAMPAIGN_ROSTER_KEY_MAX + 1]) calloc ((size_t) place_map_capacity, sizeof (*place_key_of_slot));

		if (!place_key_of_slot)
		{
			debug_fatal ("host roster: cannot allocate the placement map of the entity heap");
		}

		place_table_rebuild (4096);
	}
}

static int place_find_slot (const char *key)
{
	size_t
		position;

	if (!place_table)
	{
		return (-1);
	}

	position = place_hash (key) & (place_table_size - 1);

	while (place_table [position])
	{
		if ((place_table [position] > 0) && (strcmp (place_key_of_slot [place_table [position] - 1], key) == 0))
		{
			return (place_table [position] - 1);
		}

		position = (position + 1) & (place_table_size - 1);
	}

	return (-1);
}

static void place_forget_key (int slot)
{
	size_t
		position;

	position = place_hash (place_key_of_slot [slot]) & (place_table_size - 1);

	while (place_table [position] != slot + 1)
	{
		if (place_table [position] == 0)
		{
			debug_fatal ("host roster: the key of entity %d is not in the key table", slot);
		}

		position = (position + 1) & (place_table_size - 1);
	}

	// a removed entry stays a marker so the probe sequences of the others are unbroken
	place_table [position] = -1;

	place_table_live--;

	place_key_of_slot [slot][0] = 0;
}

// `en` was inserted at the head of a sector's list: the host places it and answers the key it issued.
static void place_insert (CAMPAIGN_PLACE_ENTITY *en)
{
	char
		key [CAMPAIGN_ROSTER_KEY_MAX + 1];

	const float
		*position;

	int
		slot;

	size_t
		at;

	// a build with no model provider has no roster and no scan (a scan asks the host and stops the engine without one)
	if (!campaign_model_host_installed ())
	{
		return;
	}

	place_ensure_map ();

	slot = place_checked_slot (en);

	if (place_key_of_slot [slot][0])
	{
		debug_fatal ("host roster: entity %d (type %s) is inserted into a sector list while the host still holds it in one", slot, CAMPAIGN_PLACE_NAME (en));
	}

	position = CAMPAIGN_PLACE_POSITION (en);

	if (!position)
	{
		debug_fatal ("host roster: entity %d (type %s) has no position, so it has no sector", slot, CAMPAIGN_PLACE_NAME (en));
	}

	key [0] = 0;

	// a hosted mobile names its semantic identity: the host decides its sector from its own record of it; every other entity states its position and the host keeps only the sector
	campaign_model_entity_roster (CAMPAIGN_SCAN_PHASE_INSERT, CAMPAIGN_PLACE_TYPE (en), CAMPAIGN_PLACE_SIDE (), position, CAMPAIGN_PLACE_IDENTITY (en), key);

	if (place_find_slot (key) >= 0)
	{
		debug_fatal ("host roster: the host issued the key %s a second time", key);
	}

	memcpy (place_key_of_slot [slot], key, strlen (key) + 1);

	if ((place_table_used + 1) * 2 > place_table_size)
	{
		// the removed markers are dropped by the rebuild: the table only grows when the live keys need it
		place_table_rebuild (((place_table_live + 1) * 4 > place_table_size) ? place_table_size * 2 : place_table_size);
	}
	else
	{
		at = place_hash (key) & (place_table_size - 1);

		while (place_table [at] > 0)
		{
			at = (at + 1) & (place_table_size - 1);
		}

		if (place_table [at] == 0)
		{
			place_table_used++;
		}

		place_table [at] = slot + 1;

		place_table_live++;
	}
}

// `en` is about to leave its sector's list: the host removes the key it issued for it.
static void place_remove (CAMPAIGN_PLACE_ENTITY *en)
{
	char
		answered [CAMPAIGN_ROSTER_KEY_MAX + 1];

	int
		slot;

	if (!campaign_model_host_installed () || !place_key_of_slot)
	{
		return;
	}

	slot = place_checked_slot (en);

	if (!place_key_of_slot [slot][0])
	{
		debug_fatal ("host roster: entity %d (type %s) leaves a sector list the host never placed it in", slot, CAMPAIGN_PLACE_NAME (en));
	}

	answered [0] = 0;

	campaign_model_entity_roster (CAMPAIGN_SCAN_PHASE_REMOVE, CAMPAIGN_PLACE_TYPE (en), 0.0f, NULL, place_key_of_slot [slot], answered);

	place_forget_key (slot);
}

// the heap slot of `en` is released: an entity that is still placed is removed first (a path that frees an entity without unlinking it from its list)
static void place_release_entity (CAMPAIGN_PLACE_ENTITY *en)
{
	int
		index;

	if (!campaign_model_host_installed () || !place_key_of_slot)
	{
		return;
	}

	index = CAMPAIGN_PLACE_SLOT (en);

	if ((index < 0) || (index >= place_map_capacity) || !place_key_of_slot [index][0])
	{
		return;
	}

	place_remove (en);
}

// a new boot: the host's roster and the campaign's keys both end
static void place_reset (void)
{
	char
		key [CAMPAIGN_ROSTER_KEY_MAX + 1];

	free (place_key_of_slot);

	free (place_table);

	place_key_of_slot = NULL;

	place_table = NULL;

	place_map_capacity = 0;

	place_table_size = 0;

	place_table_used = 0;

	place_table_live = 0;

	// the host's roster of an earlier boot ends too: a key of that boot can never name an entity of this one
	if (campaign_model_host_installed ())
	{
		key [0] = 0;

		campaign_model_entity_roster (CAMPAIGN_SCAN_PHASE_RESET, 0, 0.0f, NULL, NULL, key);
	}
}

// the entity the host's roster key names, or NULL when the campaign holds none for it
static CAMPAIGN_PLACE_ENTITY *place_entity (const char *key)
{
	int
		slot;

	slot = place_find_slot (key);

	return ((slot < 0) ? NULL : CAMPAIGN_PLACE_AT (slot));
}

#endif
