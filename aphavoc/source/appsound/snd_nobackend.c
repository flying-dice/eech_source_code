//
// 	 Enemy Engaged RAH-66 Comanche Versus KA-52 Hokum
// 	 Copyright (C) 2000 Empire Interactive (Europe) Ltd,
// 	 677 High Road, North Finchley, London N12 0DA
//
//
// EECH_CAMPAIGN_HOST_ENV (EEDCS-51, D3): THE CAMPAIGN'S NO-BACKEND SOUND BOUNDARY.  The campaign Core does not contain the legacy physical sound backend (modules/sound: DirectSound buffers, the
// 3D listener, the sample loader and the handle table; the original game still builds them).  The physical sound lifetime, the sample metadata and the backend AVAILABILITY are the host's
// (ss_hostenv.h); the availability the host gives is always "not available" (campaign_sound_backend), so the campaign has no native sound handle and loads no sample.  This file is the whole
// of what the Core offers in place of that backend, under the declarations of modules/sound/soundman.h, so every caller in the sources keeps compiling and linking unchanged:
//
//   - the no-backend ANSWERS the legacy code already handled when its sound system was not initialised: initialisation and device enumeration fail (FALSE), creating a handle gives NULL, the set of
//     handles is empty, the sound system is not paused (the campaign's own pause and continue still pause and unpause the CD music, as the legacy functions did when uninitialised), and there is
//     nothing to deinitialise or to ask whether a sample is valid;
//   - everything else asks a native handle, the listener or the sample store to do something physical.  The campaign cannot do that, and it never answers it with a made-up status or
//     success: it stops the engine BY NAME, before any native query, playback, destruction or state change.  A live native handle is therefore an unsupported attempt, not a drop and not a no-op.
//
// The handle pointer and layout stay private: no function here reads or writes one.  This file adds no physical simulation and no second backend.
//
// These refusals are the last BACKSTOP, not the evidence that the physical mixer left the Core: a caller that computes physical state before it reaches the adapter has not left it.  The hosted SoundEffect
// playback request (se_snd.c play_local_sound) is therefore decided at its own entry, before its range, volume and entropy work, with the same handle name.
//

#include "project.h"

#ifdef EECH_CAMPAIGN_HOST_ENV

//
// the campaign's own state of the sound system: never initialised (there is no backend to initialise) and, therefore, never paused by the backend
//

int
	sound_system_initialised = FALSE,
	sound_system_paused = FALSE;

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static void refuse_native_handle (const char *operation)
{
	debug_fatal ("host environment: [env-sound:unsupported_native_handle] the campaign was asked to %s a native sound handle; the legacy sound backend is not part of the campaign Core, so a live native handle is unsupported (no native query, playback, destruction or state change was made)", operation);
}

static void refuse_native_backend (const char *operation)
{
	debug_fatal ("host environment: [env-sound:unsupported_native_backend] the campaign was asked to %s the native sound backend; it is not part of the campaign Core, so native samples, the listener and the handle table are unsupported (no native query, load, playback or state change was made)", operation);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

//
// the no-backend answers
//

int initialise_sound_system ( void )
{
	return FALSE;
}

void deinitialise_sound_system ( void )
{
}

int get_sound_system_devices ( const char ***devices, const char **default_device )
{
	return FALSE;
}

int is_source_sound_sample_valid ( int sample_index )
{
	return FALSE;
}

system_sound_effect * create_single_system_sound_effect ( int sound_sample_index, int volume, int looping, void *user_data, int context )
{
	return NULL;
}

system_sound_effect * create_sequenced_system_sound_effect ( int number_of_samples, sound_sequence_information *samples, int volume, void *user_data, int context )
{
	return NULL;
}

system_sound_effect * get_first_system_sound_effect ( void )
{
	return NULL;
}

void pause_sound_system ( void )
{
	pause_music ();
}

void continue_sound_system ( void )
{
	unpause_music ();
}

int get_sound_system_paused ( void )
{
	return sound_system_paused;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

//
// a native handle is asked to do something: unsupported, by name
//

int get_system_sound_effect_playing ( system_sound_effect *effect )
{
	refuse_native_handle ("query the playing status of");

	return FALSE;
}

void destroy_system_sound_effect ( system_sound_effect *effect )
{
	refuse_native_handle ("destroy");
}

void set_system_sound_effect_user_data ( system_sound_effect *effect, void *user_data )
{
	refuse_native_handle ("set the user data of");
}

void * get_system_sound_effect_user_data ( system_sound_effect *effect )
{
	refuse_native_handle ("read the user data of");

	return NULL;
}

void play_sequenced_system_sound_effect ( system_sound_effect *effect, int sequence_index, float time_position )
{
	refuse_native_handle ("play a sequence on");
}

void play_system_sound_effect ( system_sound_effect *effect, float time_position )
{
	refuse_native_handle ("play");
}

void pause_system_sound_effect ( system_sound_effect *effect )
{
	refuse_native_handle ("pause");
}

void continue_system_sound_effect ( system_sound_effect *effect )
{
	refuse_native_handle ("continue");
}

system_sound_effect * get_next_system_sound_effect ( system_sound_effect *effect )
{
	refuse_native_handle ("walk on from");

	return NULL;
}

void set_system_sound_effect_volume ( system_sound_effect *effect, int volume )
{
	refuse_native_handle ("set the volume of");
}

void set_system_sound_effect_position ( system_sound_effect *effect, vec3d position )
{
	refuse_native_handle ("set the position of");
}

void set_system_sound_effect_pitch ( system_sound_effect *effect, float pitch )
{
	refuse_native_handle ("set the pitch of");
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

//
// the sample store, the listener and the handle table are asked to do something: unsupported, by name
//

void allocate_source_sound_samples ( int number )
{
	refuse_native_backend ("allocate the sample store of");
}

void deallocate_source_sound_samples ( void )
{
	refuse_native_backend ("deallocate the sample store of");
}

void create_source_sound_sample ( int sample_index, sample_types type, int default_rate, int size )
{
	refuse_native_backend ("create a sample in");
}

void destroy_source_sound_sample ( int sample_index )
{
	refuse_native_backend ("destroy a sample in");
}

void load_source_sound_sample ( int sample_index, void *data )
{
	refuse_native_backend ("load a sample into");
}

void destroy_all_system_sound_effects ( void )
{
	refuse_native_backend ("destroy every handle of");
}

void update_system_sound_effect_system ( void )
{
	refuse_native_backend ("update the handles of");
}

void set_system_sound_listener_orientation ( vec3d forward, vec3d up, vec3d right )
{
	refuse_native_backend ("orient the listener of");
}

#endif
