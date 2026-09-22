export type { EntityReplication } from "./entity-replication";
export type { MobilePhysicalState } from "./mobile-physical-state";

import type { EntityReplication } from "./entity-replication";
import type { MobilePhysicalState } from "./mobile-physical-state";

// Every environmental dependency of the adopted campaign slices.
export interface CampaignPorts {
	mobilePhysicalState: MobilePhysicalState;
	entityReplication: EntityReplication;
}
