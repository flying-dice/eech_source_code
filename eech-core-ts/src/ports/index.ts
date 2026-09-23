export type { CampaignEvents } from "./campaign-events";
export type { Clock } from "./clock";
export type { EntityReplication, ReplicatedEntityAttribute, ReplicatedTaskRoute } from "./entity-replication";
export type { MobilePhysicalState } from "./mobile-physical-state";
export type { Object3DBounds, Object3DMetadata } from "./object-3d-metadata";

import type { CampaignEvents } from "./campaign-events";
import type { Clock } from "./clock";
import type { EntityReplication } from "./entity-replication";
import type { MobilePhysicalState } from "./mobile-physical-state";
import type { Object3DMetadata } from "./object-3d-metadata";

// Every environmental dependency of the adopted campaign slices.
export interface CampaignPorts {
	mobilePhysicalState: MobilePhysicalState;
	entityReplication: EntityReplication;
	clock: Clock;
	object3DMetadata: Object3DMetadata;
	campaignEvents: CampaignEvents;
}
