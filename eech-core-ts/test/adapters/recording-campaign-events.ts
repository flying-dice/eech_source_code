//
// Deterministic CampaignEvents: records every event in order.
//

import type { CampaignEvents } from "../../src/ports";

export class RecordingCampaignEvents implements CampaignEvents {
	public readonly missionsCreated: number[] = [];

	public missionCreated(taskIndex: number): void {
		this.missionsCreated.push(taskIndex);
	}
}
