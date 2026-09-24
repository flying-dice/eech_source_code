//! # eech-dc: the EECH campaign in DCS World
//!
//! A Lua 5.1 native module for DCS's mission scripting state:
//!
//! ```lua
//! package.cpath = package.cpath .. ";" .. lfs.writedir () .. [[Mods\tech\EECH\bin\?.dll]]
//! local eech = require ("eech_dc")
//! local campaign = eech.new ({ campaign = { ... }, origin = { x = ..., z = ... } })
//! local events = campaign:step (dt, { position = function (name) ... end, object_bounds = ... })
//! ```
//!
//! It is an adapter over the public `eech-campaign` API only: it never sees
//! the legacy C, and the C never sees Lua. Its jobs are the ones the
//! architecture gives an adapter:
//!
//! * **coordinates** — DCS points (x north, y up, z east, theatre origin) to
//!   EECH positions (x east, y up, z north, map origin): [`coords`];
//! * **names** — the campaign configuration names DCS units and airbases; ids
//!   cross into Lua as those names;
//! * **the world** — `World` answered by Lua functions, called synchronously
//!   during the step, under DCS's own floating-point environment;
//! * **no panics, no unwinding into DCS** — every failure is a Lua error
//!   (the crate denies the panic paths, as the reference DCS module crate
//!   does), and the embedded driver (`lua/driver.lua`) protects every DCS
//!   entry point it registers.
//!
//! Modelled on flying-dice/dcs-studio's `bridge` (mlua 0.10, `lua51` +
//! `module`, DCS's `lua.dll` through an import lib, PUC Lua 5.1 for tests).

pub mod coords;
mod module;
mod names;

pub use module::eech_dc;
