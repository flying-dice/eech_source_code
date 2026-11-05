-- Prints which engine methods the loaded eech_dc module exposes to Lua; used to show the shipping (default-feature) DLL has no fixtures.
-- usage: eech-world.exe api-surface.lua root=<abs root>
local args = host.args
local root = assert (args.root, "root=<installation root>")
local dc = require ("eech_dc")
dc.prepare_installation (root)
local engine = dc.boot ({
	install_root = root,
	map = "..\\common\\maps\\map5",
	campaign_directory = "camp01",
	campaign = "LEBANON.CHC",
	gunship = "apache",
	seed = 1,
	bind_mobile = host.bind_mobile,
	move_vehicle = host.move_vehicle,
	observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
})
engine:frame (100)
print ("module", dc.name)
for _, name in ipairs ({ "frame", "report_loss", "members", "objects", "loss_fixture", "loss_state", "loss_shape", "loss_release", "loss_group", "dependent_fixture" }) do
	print (string.format ("method\t%s\t%s", name, type (engine [name])))
end
