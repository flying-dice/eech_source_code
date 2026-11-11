/*
 * TEST-ONLY (EEDCS-49, D3, Task49; features `physical-flags-check` and `physical-flags-consumer-check`, HOSTED profile only): what the two real-C flag controls share to run their UNCHANGED scenarios against the HOST's flag store.
 *
 * In the hosted profile (EECH_CAMPAIGN_HOST_ENTITY, the default build) the three per-entity UPDATED / MOVED / ROTATED words are the HOST's: the engine's own getter, setter and reset (mobile/mb_int.c, special/update/up_update.c)
 * ask the model provider (mb_hostmodel.c, mb_hostentity.h), and the legacy arrays do not exist in the build. The controls therefore cannot start a scenario by clearing an array or read a word from one. They do exactly the
 * two things the arrays allowed, through the host:
 *   - a scenario starts from a FRESH HOST BINDING: the host replaces its store with a new one (`fresh`) and the campaign forgets the group keys it held (`campaign_model_flag_forget_keys_for_test`, which only empties the
 *     table of identities). This is the process start the original's zero-initialised static arrays give a scenario that calls clear_flags (); it is NOT a reset of the store through the production reset;
 *   - a word is OBSERVED by asking the HOST for the group's whole word (`observe`, with the key the campaign holds for the slot's group: `campaign_model_flag_key_for_test`): the host's word, never a campaign copy.
 * The provider (`model`), `fresh` and `observe` are installed by the test that runs the control (the real store of eech-world, `eech-engine-sys/tests/physical_flags_hosted.rs`); the control installs the model provider as the
 * engine's host and leaves it installed (a test process ends with it). Nothing here answers a flag: the control's expectations are the ORIGINAL profile's, each over the same real getter, setter and reset.
 */
#ifndef EECH_PHYSICAL_FLAGS_HOSTED_H
#define EECH_PHYSICAL_FLAGS_HOSTED_H

#ifdef EECH_CAMPAIGN_HOST_ENTITY

#include "entity/mobile/mb_hostmodel.h"

extern const char *campaign_model_flag_key_for_test (int slot);
extern void campaign_model_flag_forget_keys_for_test (void);

typedef void (*flags_host_fresh_fn) (void);
typedef unsigned int (*flags_host_observe_fn) (const char *key, int array);

static flags_host_fresh_fn
	hosted_fresh_fn = NULL;

static flags_host_observe_fn
	hosted_observe_fn = NULL;

static void hosted_install (campaign_model_fn model, flags_host_fresh_fn fresh, flags_host_observe_fn observe)
{
	campaign_model_host
		host;

	memset (&host, 0, sizeof (host));

	host.model = model;
	hosted_fresh_fn = fresh;
	hosted_observe_fn = observe;

	campaign_model_set_host (&host);
}

static void hosted_clear (void)
{
	hosted_fresh_fn ();

	campaign_model_flag_forget_keys_for_test ();
}

/* the HOST's whole word of one array for the group a slot belongs to: 0 for a group the campaign never opened (the host holds a clear word for it by definition) */
static unsigned int hosted_word (int type, int slot)
{
	const char
		*key;

	key = campaign_model_flag_key_for_test (slot);

	if (key == NULL)
	{
		return 0;
	}

	return hosted_observe_fn (key, (type == INT_TYPE_UPDATED) ? CAMPAIGN_FLAG_ARRAY_UPDATED : ((type == INT_TYPE_MOVED) ? CAMPAIGN_FLAG_ARRAY_MOVED : CAMPAIGN_FLAG_ARRAY_ROTATED));
}

#endif

#endif
