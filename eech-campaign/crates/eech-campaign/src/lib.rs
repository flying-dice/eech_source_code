//! # eech-campaign
//!
//! The Enemy Engaged: Comanche vs Hokum dynamic campaign as an embeddable
//! native module.
//!
//! ```text
//!   host (DCS integration, a test harness, ...)
//!        │  Campaign, CampaignConfig, World, CampaignEvent, CampaignSnapshot
//!        ▼
//!   eech-campaign   public, safe API (this crate: no unsafe code)
//!        │  private FFI (eech-sys)
//!        ▼
//!   original EECH campaign C, compiled natively
//! ```
//!
//! The public API speaks campaign concepts. The implementation behind it is,
//! today, the original EECH C (entities, lists, dispatch tables): none of it
//! is visible here, and callers need not know which parts are legacy C and
//! which will be Rust later.
//!
//! ```no_run
//! use eech_campaign::*;
//! use std::time::Duration;
//!
//! # struct MyWorld;
//! # impl World for MyWorld {
//! #     fn position(&self, _: EntityId) -> Option<Position> { None }
//! #     fn object_bounds(&self, _: ObjectModel) -> Option<Bounds> { None }
//! # }
//! # fn config() -> CampaignConfig { unimplemented!() }
//! let mut world = MyWorld;
//! let mut campaign = Campaign::new(config())?;
//! let report = campaign.step(&mut world, Duration::from_millis(500))?;
//! for event in report.events {
//!     println!("{event:?}");
//! }
//! let state = campaign.snapshot();
//! # Ok::<(), CampaignError>(())
//! ```
//!
//! ## What is implemented
//!
//! The first vertical slice is the supply chain: keysite supply usage and
//! cargo, a force's low-on-supplies response, supply mission construction and
//! the assignment decision. Behaviour beyond the slice fails loudly
//! ([`CampaignError::Unported`], [`CampaignError::Boundary`]); nothing falls
//! back to an invented default. See `eech-campaign/README.md`.

#![forbid(unsafe_code)]

mod campaign;
mod error;
mod model;
mod world;

#[cfg(feature = "conformance")]
pub mod conformance;

pub use campaign::Campaign;
pub use error::{CampaignError, Result};
pub use model::*;
pub use world::World;
