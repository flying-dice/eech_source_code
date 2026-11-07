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

/* the smallest path buffer of the ORIGINAL writer: store_session (ui_menu/session/uisession.c) fills data_filename [256] and script_filename [256] with sprintf and appends the suffix with strcat, so each
   complete saved path, its suffix and the terminating NUL must fit 256 bytes (tests/sector_height_witness_guard.rs pins both literals in that source) */
#define HW_WRITER_PATH_CAPACITY 256

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

/* the length of a string that must end within `capacity` bytes, or FALSE when it does not (an unterminated member could not be copied safely) */
static int bounded_length (const char *text, size_t capacity, size_t *length)
{
	const char *end = memchr (text, '\0', capacity);

	if (!end)
	{
		return FALSE;
	}

	*length = (size_t) (end - text);

	return TRUE;
}

/*
 * The ONE check of the complete saved paths, shared by the wrapper (before store_session or any file access) and by the real-C test that calls it directly: the data path `<data_path>\<campaign_directory>\<name>.sav` and the
 * script path `<data_path>\<campaign_directory>\<name><extension>` are built exactly as the original writer builds them and must each fit HW_WRITER_PATH_CAPACITY with their suffix and NUL. Both are written to
 * `data_out` and `script_out` (HW_WRITER_PATH_CAPACITY bytes each) only when BOTH fit; otherwise both are left empty and `reason` names what failed. Returns 0 or -1.
 */
int eech_hw_save_paths (const char *data_path, size_t data_path_capacity, const char *campaign_directory, size_t directory_capacity, const char *name, const char *extension, char *data_out, char *script_out, const char **reason)
{
	size_t data_length, directory_length, name_length, extension_length;

	data_out [0] = '\0';
	script_out [0] = '\0';

	if (!bounded_length (data_path, data_path_capacity, &data_length) || !bounded_length (campaign_directory, directory_capacity, &directory_length))
	{
		*reason = "the session's data path or campaign directory is not terminated within its own array";

		return -1;
	}

	name_length = strlen (name);
	extension_length = strlen (extension);

	/* data_path, a separator, campaign_directory, a separator, the name, the suffix: the bytes of the longest path before its NUL */
	if ((data_length + 1 + directory_length + 1 + name_length + strlen (".sav")) >= HW_WRITER_PATH_CAPACITY)
	{
		*reason = "the complete saved data path does not fit the original writer's 256-byte buffer";

		return -1;
	}

	if ((data_length + 1 + directory_length + 1 + name_length + extension_length) >= HW_WRITER_PATH_CAPACITY)
	{
		*reason = "the complete saved script path does not fit the original writer's 256-byte buffer";

		return -1;
	}

	snprintf (data_out, HW_WRITER_PATH_CAPACITY, "%s\\%s\\%s%s", data_path, campaign_directory, name, ".sav");
	snprintf (script_out, HW_WRITER_PATH_CAPACITY, "%s\\%s\\%s%s", data_path, campaign_directory, name, extension);

	return 0;
}

int eech_hw_session_save_run (const char *name, char *text, int capacity)
{
	session_list_data_type *session;
	char data_path [HW_WRITER_PATH_CAPACITY], script_path [HW_WRITER_PATH_CAPACITY], extension [8];
	const char *path_reason = "";
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

	/* the saved script's extension exactly as store_session builds it */
	snprintf (extension, sizeof (extension), ".%s", game_type_extensions [get_game_type ()]);

	if (strlen (extension) != 4)
	{
		return refuse (text, capacity, "the game type extension is not three characters");
	}

	extension [3] = extension [2];
	extension [2] = extension [1];
	extension [1] = 'S';

	/* BEFORE any file access or the original writer: both complete paths must fit the writer's own buffers */
	if (eech_hw_save_paths (session->data_path, sizeof (session->data_path), session->campaign_directory, sizeof (session->campaign_directory), name, extension, data_path, script_path, &path_reason) != 0)
	{
		return refuse (text, capacity, path_reason);
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
