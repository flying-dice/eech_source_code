/*
 * EECH headless engine: the entry points a host calls. One engine per
 * process (EECH is global state); every entry sets the FPU to EECH's
 * round-toward-zero and restores the caller's environment on return.
 *
 * A debug_fatal inside EECH unwinds to the entry point, which returns
 * EECH_ENGINE_FATAL; the message is in eech_engine_fatal_message () and the
 * engine refuses every later call.
 */
#ifndef EECH_ENGINE_H
#define EECH_ENGINE_H

#include <stdint.h>

#define EECH_ENGINE_OK 0
#define EECH_ENGINE_FATAL 1
#define EECH_ENGINE_POISONED 2
#define EECH_ENGINE_STATE 3
#define EECH_ENGINE_BAD_ARGUMENT 4

struct eech_engine_config
{
	/* the installation root: contains cohokum/ (the working directory) and common/ */
	const char *install_root;
	/* EECH paths, relative to cohokum/: e.g. "..\\common\\maps\\map17", "camp01", "luxembourg.chc" */
	const char *map_path;
	const char *campaign_directory;
	const char *campaign_filename;
	/* GUNSHIP_TYPE_* of the (server) player; its side is the player's side */
	int gunship_type;
	uint32_t random_seed;
	/* extra EECH.INI-style command line arguments, NULL-terminated (may be NULL) */
	const char *const *arguments;
};

int eech_engine_boot (const struct eech_engine_config *config);

/* advance simulated time by milliseconds and run one iteration of EECH's flight loop */
int eech_engine_frame (uint32_t milliseconds);

const char *eech_engine_fatal_message (void);

/*
 * Writes the synthetic 3D object database (eech_synth3d.c) into directory
 * (an installation's cohokum/3ddata). Needs no booted engine.
 */
int eech_engine_write_3d_database (const char *directory);

#endif
