//
// Port: semantic campaign events a host may present to the player.
//
// C provenance: ui_menu/ingame/campaign/ca_msgs.c :: notify_campaign_screen
//               (the campaign screen's response table, campaign_screen_message_responses)
//
// EECH tells its campaign screen about campaign changes by message
// (CAMPAIGN_SCREEN_MISSION_CREATED, ...), and the screen's UI objects respond.
// The campaign core keeps the decision whether to tell (notify_campaign_screen's
// guard: a running campaign or skirmish) and the event itself; how a host
// shows it is outside the core. Only the events of adopted slices are here.
//

export interface CampaignEvents {
	// C: notify_campaign_screen (CAMPAIGN_SCREEN_MISSION_CREATED, task): a
	// primary task joined its start keysite's unassigned task list
	missionCreated(taskIndex: number): void;

	// C: notify_campaign_screen (CAMPAIGN_SCREEN_MISSION_ASSIGNED, task): a
	// primary task moved to its keysite's assigned task list (slice 6b)
	missionAssigned(taskIndex: number): void;
}
