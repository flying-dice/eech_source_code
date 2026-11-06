// The campaign's report of the sector lists and the city lists to the host's roster (EEDCS-49, D3, sixth and seventh slices): see mb_hostplace.h, mb_hostmodel.h (ENTITY_SCAN, the roster's and the city events) and mb_hostplace.c.
//
// The host holds the roster: which entities stand in which sector, in what order (the head of a sector's list is its newest insertion and a scan visits the tail first), decided from ITS state; and, for a
// city, which buildings are listed under it in what order (LIST_TYPE_CITY_BUILDING: a building is inserted at the head or after a predecessor and the scan walks head to tail).  The campaign tells it each
// insertion and removal and keeps, per entity, only the keys the host issued for the entity's current placements (one in a sector list, one in a city's list), so that the entry the host serves can be resolved
// back to the campaign's entity for its decisions and its consequences.  A key is the host's: it is not an index of the entity heap (that is reused, a key never is), a relink is a removal and an insertion with a
// new key, and a key that was removed names nothing.  A building's two placements are the SAME entity: the campaign declares that to the host (LINK) as soon as the second exists, and names the record by the
// legacy object-report identity when (and only when) the campaign has asked for one (NAME), so the host holds one record for the building and nothing is issued earlier than before.  The identity is named ONCE per
// record, whichever placement exists when it is first possible (an identity issued before any placement is named at the first placement, a placement that joins a named record is only linked), and the name ends with
// the record's last placement.
//
// A city's list is not tied to the city's sector placement, which is transient: the fixed-position setter every city shares removes the city from its sector list and inserts it again when the position crosses a
// sector (a session restore does the same) while its buildings stay listed.  The list is named on the wire by the key it was FIRST named by (the city's sector placement key at its first building), which names the
// host's RECORD of the city: the city and each of its buildings keep it for as long as that record lives, that is while buildings are listed or the city holds a sector placement (it survives the last building leaving
// a placed city, and a building listed again before the next relink), and it is dropped when the record ends (the placement leaves with nothing listed, or the last building leaves a city that holds no placement).
// A city's new sector placement is declared to continue the record (LINK), and the host keeps the city's record and the list in between.
//
// What this does not do: it decides no sector (the host does, from the position it is told and, for a hosted mobile, from its own record of it), it does not order the entries, and it is not the answer of a
// scan: the campaign's lists are only the mirror a scan compares the host's entry with (mb_hostscan.h, and place_child_served below for a city).
//
// It is a header, included by mb_hostplace.c (bound to the real entity heap) and by the isolated control csrc/eech_entity_dialogue_check.c (bound to scripted entities and a scripted host), so the control
// exercises THIS code and not a copy.  The includer provides FALSE, TRUE, NULL, memcpy, strcmp, strlen, calloc, free, debug_fatal, campaign_model_host_installed, campaign_model_entity_roster and
// campaign_model_entity_city (mb_hostmodel.h), and
//
//   CAMPAIGN_PLACE_ENTITY                  the entity type
//   CAMPAIGN_PLACE_HEAP ()                 the number of slots of the entity heap this run allocated (stops the engine for a heap that is not one)
//   CAMPAIGN_PLACE_SLOT (E)                the heap slot of an entity, and CAMPAIGN_PLACE_AT (SLOT) the entity in a slot
//   CAMPAIGN_PLACE_TYPE (E), CAMPAIGN_PLACE_NAME (E)   the entity's type (an int) and its type's name (text, for a refusal)
//   CAMPAIGN_PLACE_POSITION (E)            the entity's three position words, or NULL for an entity that has none
//   CAMPAIGN_PLACE_IDENTITY (E)            a hosted mobile's semantic host identity, or NULL for every other entity
//   CAMPAIGN_PLACE_OBJECT_ID (E)           the legacy object-report identity already issued for the entity (NULL when none: it is never allocated here)
//   CAMPAIGN_PLACE_SIDE ()                 the sector side length, as a float
//   CAMPAIGN_PLACE_FIXED_FAMILY (TYPE)     whether the entity type carries the fixed record whose COLLISION state the host holds (a city is not one: it has its own state, below)
//   CAMPAIGN_PLACE_FIXED_READ (E, OUT)     the entity's fixed collision state as CAMPAIGN_FIXED_STATE_WORDS floats: position (3), heading, shape, sub type
//   CAMPAIGN_PLACE_CITY (TYPE)             whether the entity type is the city
//   CAMPAIGN_PLACE_CITY_READ (E, OUT)      the city's own state as CAMPAIGN_CITY_STATE_WORDS floats, the words as the entity holds them: position (3), bounding box minimum (3), bounding box maximum (3), sub type
// and campaign_model_entity_fixed and campaign_model_entity_city_state (mb_hostmodel.h).
//
// THE FIXED COLLISION STATE (Task49). The host's record of a fixed-family placement holds the placement's collision state, and a candidate question is decided from it (mb_hostmodel.h, ENTITY_CANDIDATE). The
// campaign states it with `place_fixed_state`, which states the state the entity holds NOW under the key of the placement the entity has (its sector placement's, else its place in a city's list). It is
// called at the end of both insertions (a creator states the shape before it links the entity, a relink writes the position first, an unpack writes the record first, so the state is complete there) and by every
// writer of a field of the state after the entity is placed: a setter (position, heading, sub type, shape), a kill's destroyed shape, a restore, an unpack of a session's alive and damaged shapes and a bridge's
// damage, fall and rest (fx_*.c, *_dstry.c, *_pack.c, segment.c, sg_updt.c). An entity that is not placed (a write during its construction, before the insertion) states nothing: its insertion does.
//
// THE CITY'S OWN STATE (Task49). A city carries the fixed record too, but what a candidate question about it reads is its position and its bounding box (en_cllsn.c: the city, when its first building was served), and it has
// no setter, no pack data and no shape of its own. The same funnel states them, under the city's SECTOR placement (a city is never listed in another's list): the insertion states them as its last step (a relink repeats
// it under the new key; the host's record, which the city's list hangs from, continues across it) and the shared fixed-position setter states a position written in the same sector. The BOX is the host's: the expansion
// each building's link makes (ct_msgs.c) is built by the host from the records it holds (`place_city_expand`) and applied to the entity's mirror, never stated back. The box words are the original's INVERTED initial box (1.0e6 and -1.0e6) until the first building links: a valid statement. The words are copied as the entity
// holds them and never loaded as floats here.
//
// THE CITY'S BOX OUTLIVES ITS PLACEMENT (Task49, REQUIRED_CITY_BOX_LIFECYCLE_RETENTION). A city with no building left holds no list, and its sector placement is transient, so the box the HOST built would have ended
// with the record and the next placement would have been stated from the campaign's mirror alone. The host now keeps the record while it holds a box it built, and names it in the answer of the removal that leaves
// it (a placement's, or the last building's under a placement-less city): the campaign keeps that name as the city's list name (place_retain_record), the next placement of the city is declared to continue the
// record by it (place_declare: a LINK, and the box the insertion then states is only the mirror of the host's), and the city entity's release (place_release_entity) ends the record. A city the host built nothing for
// is unchanged: its record, and its name, end with its last placement.
#ifndef MB_HOSTPLACEMAP_H
#define MB_HOSTPLACEMAP_H

// the keys the host issued for the placements of the entity in each heap slot ("" when it has none): its place in a sector's list, and its place in a city's list (and the name of that city's list, which the building
// was inserted into, so that its removal names the same list even when the city has been relinked or its entity data is already gone); the slots are those of the heap this run allocated
static char
	(*place_key_of_slot) [CAMPAIGN_ROSTER_KEY_MAX + 1] = NULL,
	(*place_child_of_slot) [CAMPAIGN_ROSTER_KEY_MAX + 1] = NULL,
	(*place_parent_of_slot) [CAMPAIGN_ROSTER_KEY_MAX + 1] = NULL,
	(*place_list_of_slot) [CAMPAIGN_ROSTER_KEY_MAX + 1] = NULL;

// per slot, for a building: the heap slot of its city plus one (0 for none); for a city: how many buildings are listed under it, and, while there are any, the name of its list (place_list_of_slot: the key the list
// was first named by, which stays its name when the city's sector placement is relinked); and whether the host's record of the entity is named by its object-report identity
static int
	*place_city_of_slot = NULL,
	*place_listed_of_slot = NULL;

static char
	*place_named_of_slot = NULL;

static int
	place_map_capacity = 0;

// the reverse map, by key, over both kinds of placement: open addressing over (slot * 2 + kind) + 1 (0 empty, -1 a removed entry); kind 0 is a sector placement and kind 1 a city child
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

// the key a table code names: the sector placement of slot code / 2, or its city child when the low bit is set
static char *place_code_key (int code)
{
	return ((code & 1) ? place_child_of_slot [code / 2] : place_key_of_slot [code / 2]);
}

static void place_table_rebuild (size_t wanted)
{
	int
		slot,
		kind;

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
		for (kind = 0; kind < 2; kind++)
		{
			if (place_code_key (slot * 2 + kind)[0])
			{
				size_t position = place_hash (place_code_key (slot * 2 + kind)) & (place_table_size - 1);

				while (place_table [position])
				{
					position = (position + 1) & (place_table_size - 1);
				}

				place_table [position] = slot * 2 + kind + 1;

				place_table_used++;

				place_table_live++;
			}
		}
	}
}

static void place_ensure_map (void)
{
	if (!place_key_of_slot)
	{
		place_map_capacity = CAMPAIGN_PLACE_HEAP ();

		place_key_of_slot = (char (*) [CAMPAIGN_ROSTER_KEY_MAX + 1]) calloc ((size_t) place_map_capacity, sizeof (*place_key_of_slot));

		place_child_of_slot = (char (*) [CAMPAIGN_ROSTER_KEY_MAX + 1]) calloc ((size_t) place_map_capacity, sizeof (*place_child_of_slot));

		place_parent_of_slot = (char (*) [CAMPAIGN_ROSTER_KEY_MAX + 1]) calloc ((size_t) place_map_capacity, sizeof (*place_parent_of_slot));

		place_list_of_slot = (char (*) [CAMPAIGN_ROSTER_KEY_MAX + 1]) calloc ((size_t) place_map_capacity, sizeof (*place_list_of_slot));

		place_city_of_slot = (int *) calloc ((size_t) place_map_capacity, sizeof (int));

		place_listed_of_slot = (int *) calloc ((size_t) place_map_capacity, sizeof (int));

		place_named_of_slot = (char *) calloc ((size_t) place_map_capacity, sizeof (char));

		if (!place_key_of_slot || !place_child_of_slot || !place_parent_of_slot || !place_list_of_slot || !place_city_of_slot || !place_listed_of_slot || !place_named_of_slot)
		{
			debug_fatal ("host roster: cannot allocate the placement map of the entity heap");
		}

		place_table_rebuild (4096);
	}
}

// the slot a key names and, in `kind`, whether it is that slot's sector placement (0) or its city child (1); -1 for a key the campaign holds none for
static int place_find (const char *key, int *kind)
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
		if ((place_table [position] > 0) && (strcmp (place_code_key (place_table [position] - 1), key) == 0))
		{
			*kind = (place_table [position] - 1) & 1;

			return ((place_table [position] - 1) / 2);
		}

		position = (position + 1) & (place_table_size - 1);
	}

	return (-1);
}

static void place_forget_key (int slot, int kind)
{
	size_t
		position;

	char
		*key;

	key = place_code_key (slot * 2 + kind);

	position = place_hash (key) & (place_table_size - 1);

	while (place_table [position] != slot * 2 + kind + 1)
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

	key [0] = 0;
}

// the key the host issued is the entity's from now on: it is entered in the slot's place and in the reverse table (a key the host issued twice stops the engine)
static void place_register (int slot, int kind, const char *key)
{
	char
		*held;

	int
		other;

	size_t
		at;

	if (place_find (key, &other) >= 0)
	{
		debug_fatal ("host roster: the host issued the key %s a second time", key);
	}

	held = place_code_key (slot * 2 + kind);

	memcpy (held, key, strlen (key) + 1);

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

		place_table [at] = slot * 2 + kind + 1;

		place_table_live++;
	}
}

// two keys, or a key and an object identity, in the identity place: `first`, one space, `second`.  The place is CAMPAIGN_MODEL_SCAN_IDENTITY_MAX bytes: a key and an object identity each at their own maximum fit
// together, so a refusal here is a part outside its own domain, never a join that does not fit
static void place_pair (char *out, size_t capacity, const char *first, const char *second)
{
	size_t
		a,
		b;

	a = strlen (first);

	b = strlen (second);

	if (a + 1 + b + 1 > capacity)
	{
		debug_fatal ("host roster: the identity place of a city event cannot hold %d and %d bytes (its field is %d bytes)", (int) a, (int) b, (int) capacity - 1);
	}

	memcpy (out, first, a);

	out [a] = ' ';

	memcpy (out + a + 1, second, b + 1);
}

// the name the host's list of the city in `slot` is addressed by: the key it was first named by while buildings are listed under the city (its sector placement may have been relinked since), otherwise the key of
// the city's own sector placement
static const char *place_list_key (int slot)
{
	return (place_list_of_slot [slot][0] ? place_list_of_slot [slot] : place_key_of_slot [slot]);
}

// the host's record of the entity in `slot` ends with its last placement, and its list: its object-report name ends with it
static void place_unname_ended (int slot)
{
	if (!place_key_of_slot [slot][0] && !place_child_of_slot [slot][0] && !place_list_of_slot [slot][0])
	{
		place_named_of_slot [slot] = 0;
	}
}

// the entity's placements are one entity and one record: the host is told as soon as the second placement exists (a creation puts the building in its city's list first and its sector's second, a relink puts the
// sector placement back, and a city's relinked sector placement continues the record its list hangs from), and the record is named by the object-report identity the entity already holds, once, whichever placement
// exists (an identity issued before any placement is named at the first one).  Nothing is allocated here: an identity is named only if one was issued.
static void place_declare (CAMPAIGN_PLACE_ENTITY *en, int slot)
{
	char
		identity [CAMPAIGN_MODEL_SCAN_IDENTITY_MAX + 1],
		answered [CAMPAIGN_ROSTER_KEY_MAX + 1];

	const char
		*object;

	answered [0] = 0;

	if (place_key_of_slot [slot][0] && place_child_of_slot [slot][0])
	{
		place_pair (identity, sizeof (identity), place_key_of_slot [slot], place_child_of_slot [slot]);

		campaign_model_entity_city (CAMPAIGN_SCAN_PHASE_LINK, CAMPAIGN_PLACE_TYPE (en), identity, answered);
	}
	else if (place_key_of_slot [slot][0] && place_list_of_slot [slot][0])
	{
		place_pair (identity, sizeof (identity), place_key_of_slot [slot], place_list_of_slot [slot]);

		campaign_model_entity_city (CAMPAIGN_SCAN_PHASE_LINK, CAMPAIGN_PLACE_TYPE (en), identity, answered);
	}

	if (place_named_of_slot [slot] || (!place_key_of_slot [slot][0] && !place_child_of_slot [slot][0]))
	{
		return;
	}

	object = CAMPAIGN_PLACE_OBJECT_ID (en);

	if (object && object [0])
	{
		place_pair (identity, sizeof (identity), place_key_of_slot [slot][0] ? place_key_of_slot [slot] : place_child_of_slot [slot], object);

		campaign_model_entity_city (CAMPAIGN_SCAN_PHASE_NAME, CAMPAIGN_PLACE_TYPE (en), identity, answered);

		place_named_of_slot [slot] = 1;
	}
}

// the collision state of `en` as it is NOW is stated to the host's record of the placement the entity has: its sector placement's key, else its place in a city's list. A fixed-family entity states its fixed state and a
// city its own position and box. A type that has none, an entity that is not placed (a write during its construction, before its insertion, which then states it) and a build with no host state nothing. The state is
// read here, never kept: the host's record is the only one.
static void place_fixed_state (CAMPAIGN_PLACE_ENTITY *en)
{
	float
		state [CAMPAIGN_FIXED_STATE_WORDS],
		city [CAMPAIGN_CITY_STATE_WORDS];

	const char
		*key;

	int
		slot,
		of_city;

	if (!campaign_model_host_installed () || !place_key_of_slot)
	{
		return;
	}

	of_city = CAMPAIGN_PLACE_CITY (CAMPAIGN_PLACE_TYPE (en));

	if (!of_city && !CAMPAIGN_PLACE_FIXED_FAMILY (CAMPAIGN_PLACE_TYPE (en)))
	{
		return;
	}

	slot = place_checked_slot (en);

	key = place_key_of_slot [slot][0] ? place_key_of_slot [slot] : place_child_of_slot [slot];

	if (!key [0])
	{
		return;
	}

	if (of_city)
	{
		CAMPAIGN_PLACE_CITY_READ (en, city);

		campaign_model_entity_city_state (key, city);

		return;
	}

	CAMPAIGN_PLACE_FIXED_READ (en, state);

	campaign_model_entity_fixed (CAMPAIGN_PLACE_TYPE (en), key, state);
}

// the key the host issued for the placement `en` has now: its sector placement's, or (`child` TRUE) its place in a city's list; empty when it has none or there is no host. A candidate names it on every staged
// question (mb_hostoffer.h). It is read here, never kept: a relink is a removal and a new insertion with a new key.
static const char *place_key (CAMPAIGN_PLACE_ENTITY *en, int child)
{
	int
		slot;

	if (!campaign_model_host_installed () || !place_key_of_slot)
	{
		return "";
	}

	slot = place_checked_slot (en);

	return (child ? place_child_of_slot [slot] : place_key_of_slot [slot]);
}

// The host named the record it keeps for the bounding box it built when it answered the removal of a city's placement (or of the last building listed under a placement-less city): the city has nothing else
// that would hold the box, and the campaign's mirror must never be its only copy. Only a city can hold one, and the name is the city's own list name when it already holds one; the name stays until the next
// placement of the city is declared to continue the record (place_declare) and until the city entity is released (place_release_entity).
static void place_retain_record (CAMPAIGN_PLACE_ENTITY *en, int slot, const char *name)
{
	if (!CAMPAIGN_PLACE_CITY (CAMPAIGN_PLACE_TYPE (en)))
	{
		debug_fatal ("host roster: the host named the record %.40s for the removal of entity %d (type %s), which is no city and has no bounding box to keep", name, slot, CAMPAIGN_PLACE_NAME (en));
	}

	if (place_list_of_slot [slot][0])
	{
		if (strcmp (place_list_of_slot [slot], name) != 0)
		{
			debug_fatal ("host roster: the host named the record %.40s for the city %d, which holds the record %.40s", name, slot, place_list_of_slot [slot]);
		}

		return;
	}

	memcpy (place_list_of_slot [slot], name, strlen (name) + 1);
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

	place_register (slot, 0, key);

	place_declare (en, slot);

	// the state of a fixed-family entity, complete here (see the head of this file): stated under the placement just made, so the record that placement joined holds it
	place_fixed_state (en);
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

	place_forget_key (slot, 0);

	// the city's placement leaves: with buildings listed the host's record stays (the next placement continues it); the host may keep it for the bounding box it built and then names it (the name stays, and the next
	// placement continues the record); with neither the record ends with this placement, and so does its name
	if (answered [0])
	{
		place_retain_record (en, slot, answered);
	}
	else if (place_listed_of_slot [slot] <= 0)
	{
		place_list_of_slot [slot][0] = 0;
	}

	place_unname_ended (slot);
}

// `en` was inserted into the list of the buildings of the city `parent`, at the head (`pred` NULL) or after `pred`: the host lists it and answers the key it issued. The city must already hold a sector placement
// (the host names the list by the key of it that the city's first building was listed under): a city is created, and placed, before any of its buildings.
static void place_child_insert (CAMPAIGN_PLACE_ENTITY *en, CAMPAIGN_PLACE_ENTITY *parent, CAMPAIGN_PLACE_ENTITY *pred)
{
	char
		key [CAMPAIGN_ROSTER_KEY_MAX + 1],
		identity [CAMPAIGN_MODEL_SCAN_IDENTITY_MAX + 1];

	const char
		*list;

	int
		slot,
		parent_slot,
		pred_slot;

	if (!campaign_model_host_installed ())
	{
		return;
	}

	place_ensure_map ();

	slot = place_checked_slot (en);

	if (place_child_of_slot [slot][0])
	{
		debug_fatal ("host roster: entity %d (type %s) is inserted into a city list while the host still holds it in one", slot, CAMPAIGN_PLACE_NAME (en));
	}

	parent_slot = place_checked_slot (parent);

	if (!place_key_of_slot [parent_slot][0])
	{
		debug_fatal ("host roster: the city %d (type %s) of a building's list holds no sector placement, so the host has no key to name its list by", parent_slot, CAMPAIGN_PLACE_NAME (parent));
	}

	list = place_list_key (parent_slot);

	if (pred)
	{
		pred_slot = place_checked_slot (pred);

		if (!place_child_of_slot [pred_slot][0])
		{
			debug_fatal ("host roster: the predecessor %d (type %s) of a city list entry is not listed by the host", pred_slot, CAMPAIGN_PLACE_NAME (pred));
		}

		place_pair (identity, sizeof (identity), list, place_child_of_slot [pred_slot]);
	}
	else
	{
		// no predecessor: the identity is the list's name alone
		memcpy (identity, list, strlen (list) + 1);
	}

	key [0] = 0;

	campaign_model_entity_city (CAMPAIGN_SCAN_PHASE_CHILD_INSERT, CAMPAIGN_PLACE_TYPE (en), identity, key);

	place_register (slot, 1, key);

	memcpy (place_parent_of_slot [slot], list, strlen (list) + 1);

	place_city_of_slot [slot] = parent_slot + 1;

	// the city's first building names its list by the key of the city's sector placement: the city keeps that name while buildings are listed under it
	if (!place_list_of_slot [parent_slot][0])
	{
		memcpy (place_list_of_slot [parent_slot], place_parent_of_slot [slot], strlen (place_parent_of_slot [slot]) + 1);
	}

	place_listed_of_slot [parent_slot]++;

	place_declare (en, slot);

	// a building is listed in its city before its sector (cb_creat.c): its state is stated under whichever placement it has, here the city's list, and again under its sector placement
	place_fixed_state (en);
}

// A building's link has expanded its city's bounding box (ct_msgs.c; Task49): the HOST builds the box from the records it already holds, and the campaign applies the checked answer to its mirror in `box`
// (CAMPAIGN_CITY_EXPAND_WORDS floats, written only once the whole answer held). The question names the city's sector placement and the building's place in the city's list; both must be placed (the creator listed
// the building in its city before this link: en_list.c), and the building must be listed under THIS city. A pair that is not (no host installed, a city with no sector placement, an unlisted building, a building
// of another city) is the campaign's fault and stops the engine by name BEFORE anything is asked: the campaign has no physical construction of its own to fall back on.
static void place_city_expand (CAMPAIGN_PLACE_ENTITY *city, CAMPAIGN_PLACE_ENTITY *building, float *box)
{
	int
		city_slot,
		building_slot;

	if (!campaign_model_host_installed ())
	{
		debug_fatal ("host roster: the bounding box of a city is built by the host, and no host is installed");
	}

	place_ensure_map ();

	city_slot = place_checked_slot (city);

	building_slot = place_checked_slot (building);

	if (!place_key_of_slot [city_slot][0])
	{
		debug_fatal ("host roster: the city %d (type %s) of a building's link holds no sector placement, so the host has no key to build its box under", city_slot, CAMPAIGN_PLACE_NAME (city));
	}

	if (!place_child_of_slot [building_slot][0])
	{
		debug_fatal ("host roster: the building %d (type %s) of a city's box expansion is not listed by the host in a city's list", building_slot, CAMPAIGN_PLACE_NAME (building));
	}

	if (place_city_of_slot [building_slot] != city_slot + 1)
	{
		debug_fatal ("host roster: the building %d (type %s) of a city's box expansion is listed under another city than %d", building_slot, CAMPAIGN_PLACE_NAME (building), city_slot);
	}

	campaign_model_entity_city_expand (place_key_of_slot [city_slot], place_child_of_slot [building_slot], box);
}

// `en` is about to leave the list of the buildings of its city: the host unlists the key it issued for it, naming the list by the name recorded at the insertion (the city's current sector placement may not be the key
// it was inserted under, or may not exist at the moment).
static void place_child_remove (CAMPAIGN_PLACE_ENTITY *en)
{
	char
		answered [CAMPAIGN_ROSTER_KEY_MAX + 1],
		identity [CAMPAIGN_MODEL_SCAN_IDENTITY_MAX + 1];

	int
		slot,
		city;

	if (!campaign_model_host_installed () || !place_key_of_slot)
	{
		return;
	}

	slot = place_checked_slot (en);

	if (!place_child_of_slot [slot][0])
	{
		debug_fatal ("host roster: entity %d (type %s) leaves a city list the host never listed it in", slot, CAMPAIGN_PLACE_NAME (en));
	}

	place_pair (identity, sizeof (identity), place_parent_of_slot [slot], place_child_of_slot [slot]);

	answered [0] = 0;

	campaign_model_entity_city (CAMPAIGN_SCAN_PHASE_CHILD_REMOVE, CAMPAIGN_PLACE_TYPE (en), identity, answered);

	place_forget_key (slot, 1);

	// the city loses a building (a city that was released with buildings still listed no longer holds the list, and a slot that holds another city's list is not this one's).  The name is the host's RECORD's, which
	// lives while the city holds a sector placement or a building is listed: with the last building gone the city keeps the name while it stays placed (a building listed again, or a relink, still names that
	// record) and drops it when it holds no placement either, which is when the host's record ends
	city = place_city_of_slot [slot] - 1;

	if ((city >= 0) && place_list_of_slot [city][0] && (strcmp (place_list_of_slot [city], place_parent_of_slot [slot]) == 0))
	{
		place_listed_of_slot [city]--;

		if (place_listed_of_slot [city] <= 0)
		{
			place_listed_of_slot [city] = 0;

			// a record the host keeps for the bounding box it built keeps its name too (the host named it in this answer)
			if (!place_key_of_slot [city][0] && !answered [0])
			{
				place_list_of_slot [city][0] = 0;
			}

			place_unname_ended (city);
		}
	}

	if (answered [0] && !((city >= 0) && place_list_of_slot [city][0] && (strcmp (place_list_of_slot [city], answered) == 0)))
	{
		debug_fatal ("host roster: the host named the record %.40s for the removal of the building %d, which the campaign's city does not hold", answered, slot);
	}

	place_parent_of_slot [slot][0] = 0;

	place_city_of_slot [slot] = 0;

	place_unname_ended (slot);
}

// the heap slot of `en` is released: an entity that is still placed is removed first (a path that frees an entity without unlinking it from its lists)
static void place_release_entity (CAMPAIGN_PLACE_ENTITY *en)
{
	char
		answered [CAMPAIGN_ROSTER_KEY_MAX + 1];

	int
		index;

	if (!campaign_model_host_installed () || !place_key_of_slot)
	{
		return;
	}

	index = CAMPAIGN_PLACE_SLOT (en);

	if ((index < 0) || (index >= place_map_capacity))
	{
		return;
	}

	if (place_child_of_slot [index][0])
	{
		place_child_remove (en);
	}

	if (place_key_of_slot [index][0])
	{
		place_remove (en);
	}

	// the city entity ends: the host ends the record it still holds for it (the one it kept for the bounding box it built, or the list of a city released with buildings still listed). This is the real end of the
	// entity's lifetime; the host's own withdrawal of a state ends nothing
	if (place_list_of_slot [index][0] && CAMPAIGN_PLACE_CITY (CAMPAIGN_PLACE_TYPE (en)))
	{
		answered [0] = 0;

		campaign_model_entity_city (CAMPAIGN_SCAN_PHASE_CITY_RELEASE, CAMPAIGN_PLACE_TYPE (en), place_list_of_slot [index], answered);
	}

	// a city released with buildings still listed no longer holds a list: the buildings keep the name they were listed under and end the host's list as they are released
	place_list_of_slot [index][0] = 0;

	place_listed_of_slot [index] = 0;

	place_named_of_slot [index] = 0;
}

// a new boot: the host's roster and the campaign's keys both end
static void place_reset (void)
{
	char
		key [CAMPAIGN_ROSTER_KEY_MAX + 1];

	free (place_key_of_slot);

	free (place_child_of_slot);

	free (place_parent_of_slot);

	free (place_list_of_slot);

	free (place_city_of_slot);

	free (place_listed_of_slot);

	free (place_named_of_slot);

	free (place_table);

	place_key_of_slot = NULL;

	place_child_of_slot = NULL;

	place_parent_of_slot = NULL;

	place_list_of_slot = NULL;

	place_city_of_slot = NULL;

	place_listed_of_slot = NULL;

	place_named_of_slot = NULL;

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

// the entity the host's roster key names as a SECTOR placement, or NULL when the campaign holds none for it (a key of a city child names none here)
static CAMPAIGN_PLACE_ENTITY *place_entity (const char *key)
{
	int
		slot,
		kind;

	kind = 0;

	slot = place_find (key, &kind);

	return (((slot < 0) || (kind != 0)) ? NULL : CAMPAIGN_PLACE_AT (slot));
}

// the building the host served, resolved to the campaign's entity and compared with the campaign's own list at the same place (`mirror`: the entry the campaign's list holds there, or NULL at its end): the host's
// list is the answer and the campaign's only the mirror, so an entry the campaign's list does not agree with, and a key that names no building, stop the engine by name; neither is ever an end or a fallback
static CAMPAIGN_PLACE_ENTITY *place_child_served (const char *key, CAMPAIGN_PLACE_ENTITY *mirror)
{
	CAMPAIGN_PLACE_ENTITY
		*entry;

	int
		slot,
		kind;

	if (key [0] == 0)
	{
		if (mirror)
		{
			debug_fatal ("host model: [model-provider:entity_scan.city_mirror] the host served the end of a city's list but the campaign's own list has another building: the host's list and the campaign's disagree");
		}

		return (NULL);
	}

	kind = 0;

	slot = place_find (key, &kind);

	if ((slot < 0) || (kind != 1))
	{
		debug_fatal ("host model: [model-provider:entity_scan.city_key] the host served the building \"%.40s\", which names no building of the campaign's cities", key);
	}

	entry = CAMPAIGN_PLACE_AT (slot);

	if (entry != mirror)
	{
		debug_fatal ("host model: [model-provider:entity_scan.city_mirror] the host served a building but the campaign's own list is %s: the host's list and the campaign's disagree", mirror ? "at another building" : "at its end");
	}

	return (entry);
}

// the first building of the city `parent`, from the host's list; `mirror` is the first child of the campaign's list
static CAMPAIGN_PLACE_ENTITY *place_child_first (CAMPAIGN_PLACE_ENTITY *parent, CAMPAIGN_PLACE_ENTITY *mirror)
{
	char
		key [CAMPAIGN_ROSTER_KEY_MAX + 1];

	int
		slot;

	place_ensure_map ();

	slot = place_checked_slot (parent);

	if (!place_key_of_slot [slot][0])
	{
		debug_fatal ("host model: [model-provider:entity_scan.city_parent] the city %d (type %s) a scan reads the buildings of holds no sector placement, so the host has no list for it", slot, CAMPAIGN_PLACE_NAME (parent));
	}

	key [0] = 0;

	campaign_model_entity_city (CAMPAIGN_SCAN_PHASE_CHILD_FIRST, CAMPAIGN_PLACE_TYPE (parent), place_list_key (slot), key);

	return (place_child_served (key, mirror));
}

// the building after `building` in the list of the city `parent`, from the host's list; `mirror` is the successor in the campaign's list
static CAMPAIGN_PLACE_ENTITY *place_child_next (CAMPAIGN_PLACE_ENTITY *parent, CAMPAIGN_PLACE_ENTITY *building, CAMPAIGN_PLACE_ENTITY *mirror)
{
	char
		key [CAMPAIGN_ROSTER_KEY_MAX + 1],
		identity [CAMPAIGN_MODEL_SCAN_IDENTITY_MAX + 1];

	int
		slot,
		child_slot;

	place_ensure_map ();

	slot = place_checked_slot (parent);

	child_slot = place_checked_slot (building);

	if (!place_key_of_slot [slot][0] || !place_child_of_slot [child_slot][0])
	{
		debug_fatal ("host model: [model-provider:entity_scan.city_parent] the scan stands at a building (entity %d) of a city (entity %d) and the host holds no key for one of them", child_slot, slot);
	}

	place_pair (identity, sizeof (identity), place_list_key (slot), place_child_of_slot [child_slot]);

	key [0] = 0;

	campaign_model_entity_city (CAMPAIGN_SCAN_PHASE_CHILD_NEXT, CAMPAIGN_PLACE_TYPE (building), identity, key);

	return (place_child_served (key, mirror));
}

// the campaign has asked the host for the legacy object-report identity of `en` (lazily, once) and the host issued `id`: the placements the host holds for the entity are named by it, so that the host's
// record of the entity and the identity the campaign reports about are the same record
static void place_name (CAMPAIGN_PLACE_ENTITY *en, const char *id)
{
	char
		identity [CAMPAIGN_MODEL_SCAN_IDENTITY_MAX + 1],
		answered [CAMPAIGN_ROSTER_KEY_MAX + 1];

	int
		slot;

	if (!campaign_model_host_installed () || !place_key_of_slot || !id || !id [0])
	{
		return;
	}

	slot = place_checked_slot (en);

	// a record is named once, by either placement; an entity with none is named when it is placed (place_declare)
	if (place_named_of_slot [slot] || (!place_key_of_slot [slot][0] && !place_child_of_slot [slot][0]))
	{
		return;
	}

	place_pair (identity, sizeof (identity), place_key_of_slot [slot][0] ? place_key_of_slot [slot] : place_child_of_slot [slot], id);

	answered [0] = 0;

	campaign_model_entity_city (CAMPAIGN_SCAN_PHASE_NAME, CAMPAIGN_PLACE_TYPE (en), identity, answered);

	place_named_of_slot [slot] = 1;
}

#endif
