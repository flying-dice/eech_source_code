/* TEST-ONLY, included by eech_engine.c after its real ENTER/LEAVE definitions.
 * No boot, entity heap, replacement host, copied entry macro or recovery API.
 * This entry deliberately permits the real flag functions before boot: it tests
 * the entry/callback/store join, not a shipping public operation or session.
 * One dedicated process: the fatal case last, no global or FPU restoration claim.
 */
#ifdef EECH_PHYSICAL_FLAGS_LIFETIME_CHECK
int eech_engine_flags_lifetime_check (int mode, int *word, const struct eech_engine_host *host)
{
	ENTER (OPERATION_OBSERVE, host);
	if (!word || (mode < 0) || (mode > 2))
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	if (mode == 0)
	{
		campaign_model_flag_set (CAMPAIGN_FLAG_ARRAY_MOVED, 37);
	}
	else if (mode == 2)
	{
		LEAVE (EECH_ENGINE_BAD_ARGUMENT);
	}
	*word = campaign_model_flag_read (CAMPAIGN_FLAG_ARRAY_MOVED, 37);
	LEAVE (EECH_ENGINE_OK);
}
#endif
