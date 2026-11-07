/*
 * TEST-ONLY (EEDCS-49, D3, Task49, feature `sector-height-witness`): the fixture METHOD of the native server-session witness (ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1). Never part of the shipping module: with the
 * feature off this file defines nothing and no entry exists.
 *
 * WHAT IT DOES: it makes the campaign perform its OWN server-session save, once, on the live session. It calls the original `store_session` (ui_menu/session/uisession.c, the function the in-game save runs),
 * which writes the campaign's saved SCRIPT (`<name>.S<ext>`, whose FILE_TAG_SAVED_CAMPAIGN names the data) and the saved DATA (`<name>.sav`: the session version int, then `pack_session` in PACK_MODE_SERVER_SESSION into a
 * buffer that store_session itself grows until the pack no longer overflows). Packing a sector asks the HOST for its word (scan phase 17, through the sector getter): that is the save of the host-owned
 * high-water height. The matching LOAD is the campaign's own, not a method here: a later boot of the saved script reaches FILE_TAG_SAVED_CAMPAIGN in `parse_campaign_script` (ai/parser/parsgen.c), which reads
 * the data and calls `unpack_session` (PACK_MODE_SERVER_SESSION); unpacking a sector states its saved word RAW to the host (phase 18) and the host acknowledges the bits it holds.
 *
 * WHAT IT DOES NOT DO: it never calls the provider's restore, packs by hand, unpacks anything or loads into a populated world (the campaign's load is a fresh boot of the saved script, in its own process: an unpack
 * onto live entities is not a load and is never attempted). It reads no sector word itself and writes none. Raw histories the host answers are the HOST's fixture inputs (lua/sector-height-witness.lua).
 *
 * SAFE BY CONSTRUCTION (every refusal is a bad argument that names its reason in the report and changes nothing): the name is 1 to 24 of [A-Za-z0-9_-] (store_session keeps 31 characters and builds paths from it); the
 * session must be the booted campaign's own server session of the campaign game type; NEITHER the saved data NOR the saved script may already exist (store_session rotates numbered backups and deletes the oldest when
 * both exist, and rewrites a script that exists: this method never overwrites, rotates or deletes a file); after the call the data file must exist and begin with the session version int the load checks. A failure
 * inside store_session or the pack stops the engine through its own fatal path (the entry's longjmp), as any campaign fatal does: nothing here catches it and nothing continues after it.
 *
 * eech_hw_session_save_run writes `sh-witness-save name=<name> bytes=<size of the data file> version=<int>` on success. Return 0: saved. Non-zero: refused (the reason is in the text).
 */

#ifdef EECH_SECTOR_HEIGHT_WITNESS

#include <stdio.h>
#include <string.h>

#include "project.h"

#define HW_NAME_MAX 24

static int refuse (char *text, int capacity, const char *reason)
{
	snprintf (text, (size_t) capacity, "sh-witness-refused reason=%s\n", reason);

	return -1;
}

static int name_is_valid (const char *name)
{
	size_t length, i;

	if (!name)
	{
		return FALSE;
	}

	length = strlen (name);

	if ((length < 1) || (length > HW_NAME_MAX))
	{
		return FALSE;
	}

	for (i = 0; i < length; i++)
	{
		char c = name [i];

		if (!(((c >= 'a') && (c <= 'z')) || ((c >= 'A') && (c <= 'Z')) || ((c >= '0') && (c <= '9')) || (c == '_') || (c == '-')))
		{
			return FALSE;
		}
	}

	return TRUE;
}

static int file_exists (const char *path)
{
	FILE *file = fopen (path, "rb");

	if (file)
	{
		fclose (file);

		return TRUE;
	}

	return FALSE;
}

int eech_hw_session_save_run (const char *name, char *text, int capacity)
{
	session_list_data_type *session;
	char data_path [320], script_path [320], extension [8];
	FILE *file;
	long size;
	int version, saved_version;

	if (!text || (capacity <= 0))
	{
		return -1;
	}

	text [0] = '\0';

	if (!name_is_valid (name))
	{
		return refuse (text, capacity, "the name must be 1 to 24 characters of A-Z a-z 0-9 _ -");
	}

	session = get_current_game_session ();

	if (!session)
	{
		return refuse (text, capacity, "there is no current game session");
	}

	if (get_comms_model () != COMMS_MODEL_SERVER)
	{
		return refuse (text, capacity, "only a server session is saved");
	}

	if (get_game_type () != GAME_TYPE_CAMPAIGN)
	{
		return refuse (text, capacity, "only a campaign session is saved");
	}

	if (snprintf (data_path, sizeof (data_path), "%s\\%s\\%s.sav", session->data_path, session->campaign_directory, name) >= (int) sizeof (data_path))
	{
		return refuse (text, capacity, "the saved data path does not fit");
	}

	/* the saved script's extension exactly as store_session builds it */
	snprintf (extension, sizeof (extension), ".%s", game_type_extensions [get_game_type ()]);

	if (strlen (extension) != 4)
	{
		return refuse (text, capacity, "the game type extension is not three characters");
	}

	extension [3] = extension [2];
	extension [2] = extension [1];
	extension [1] = 'S';

	if (snprintf (script_path, sizeof (script_path), "%s\\%s\\%s%s", session->data_path, session->campaign_directory, name, extension) >= (int) sizeof (script_path))
	{
		return refuse (text, capacity, "the saved script path does not fit");
	}

	if (file_exists (data_path) || file_exists (script_path))
	{
		return refuse (text, capacity, "a saved file of that name exists: this method never overwrites, rotates or deletes");
	}

	version = get_local_entity_int_value (get_session_entity (), INT_TYPE_VERSION_NUMBER);

	if (!store_session (session, name))
	{
		return refuse (text, capacity, "store_session did not save");
	}

	file = fopen (data_path, "rb");

	if (!file)
	{
		return refuse (text, capacity, "the saved data file was not written");
	}

	saved_version = 0;

	if ((fread (&saved_version, sizeof (saved_version), 1, file) != 1) || (fseek (file, 0, SEEK_END) != 0) || ((size = ftell (file)) < (long) sizeof (saved_version)))
	{
		fclose (file);

		return refuse (text, capacity, "the saved data file is not readable or holds less than its version int");
	}

	fclose (file);

	if (saved_version != version)
	{
		return refuse (text, capacity, "the saved data does not begin with the session version");
	}

	snprintf (text, (size_t) capacity, "sh-witness-save name=%s bytes=%ld version=%d\n", name, size, version);

	return 0;
}

#endif
