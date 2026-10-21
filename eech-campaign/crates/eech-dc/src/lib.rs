//! `eech_dc`: the EECH dynamic campaign as a Lua 5.1 native module.
//!
//! The DLL contains the whole headless EECH engine (eech-engine): the
//! campaign parser and dynamic campaign, AI, pathfinding, flight, weapons,
//! damage, comms and entity system. A host's Lua state loads it with
//! `require ("eech_dc")` (the host must allow C modules) and drives it:
//!
//! ```lua
//! local dc = require ("eech_dc")
//! dc.prepare_installation (root)
//! local next_id = 0
//! local engine = dc.boot { install_root = root, map = "..\\common\\maps\\map16",
//!                          campaign_directory = "camp01", campaign = "georgia.chc",
//!                          gunship = "apache", seed = 1,
//!                          bind_mobile = function () next_id = next_id + 1; return "host:" .. next_id end }
//! while true do
//!     engine:frame (100)                -- milliseconds of simulated time
//!     for _, o in ipairs (engine:objects ()) do ... end
//! end
//! ```
//!
//! Errors are Lua errors whose message starts with `eech:`. An EECH fatal
//! error poisons the engine: every later call fails.

mod module;

pub use module::eech_dc;
