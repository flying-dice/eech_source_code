//
// Campaign screen notification.
//
// C provenance: ui_menu/ingame/campaign/ca_msgs.c :: notify_campaign_screen
//
// The guard is campaign behaviour and stays in the core: the campaign screen
// hears about changes only while a campaign or skirmish is running. The
// screen's responses (campaign_screen_message_responses) are the UI; the
// port reports the semantic event instead (CampaignEvents). DEMO_VERSION is 0
// in EECH builds. Only the message adopted slices send is ported.
//

import { getGameStatus } from "../../core/game-status";
import { getGameType } from "../../core/game-type";
import { GameStatusType, GameType } from "../../generated/c-enums";
import { getCampaignPorts, type Entity } from "../../entity/system/entity";

function campaignScreenListening(): boolean {
	if (getGameStatus() !== GameStatusType.GAME_STATUS_INITIALISED) {
		return false;
	}

	if (getGameType() !== GameType.GAME_TYPE_CAMPAIGN && getGameType() !== GameType.GAME_TYPE_SKIRMISH) {
		return false;
	}

	return true;
}

// C: notify_campaign_screen (CAMPAIGN_SCREEN_MISSION_CREATED, sender)
export function notifyCampaignScreenMissionCreated(sender: Entity): boolean {
	if (!campaignScreenListening()) {
		return false;
	}

	getCampaignPorts().campaignEvents.missionCreated(sender.index);

	return true;
}
