export type { Clock } from "./clock";
export type { EntityReplication, ReplicatedEntityAttribute } from "./entity-replication";
export type { MobilePhysicalState } from "./mobile-physical-state";

import type { Clock } from "./clock";
import type { EntityReplication } from "./entity-replication";
import type { MobilePhysicalState } from "./mobile-physical-state";

// Every environmental dependency of the adopted campaign slices.
export interface CampaignPorts {
	mobilePhysicalState: MobilePhysicalState;
	entityReplication: EntityReplication;
	clock: Clock;
}
