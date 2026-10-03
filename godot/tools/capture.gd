## Dev tool: boot a scene, optionally stage it, render real frames and save
## a screenshot. Used to eyeball UI/world changes without clicking through.
##
## godot --path godot res://tools/Capture.tscn -- \
##   scene=res://scenes/world/TownAshlight.tscn setup=forge out=C:/tmp/x.png
extends Node

const WAIT_FRAMES := 45

var _args := {}

func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		var kv := a.split("=", true, 1)
		if kv.size() == 2:
			_args[kv[0]] = kv[1]
	_seed_run()
	var scene: PackedScene = load(_args.get("scene", "res://scenes/world/TownAshlight.tscn"))
	var root := scene.instantiate()
	add_child(root)
	for i in range(WAIT_FRAMES):
		await get_tree().process_frame
	_stage(root, _args.get("setup", ""))
	for i in range(10):
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	var out: String = _args.get("out", "user://capture.png")
	get_viewport().get_texture().get_image().save_png(out)
	print("captured ", out)
	get_tree().quit()

## A believable mid-game run so panels have something to show.
func _seed_run() -> void:
	SaveSystem.save_path = "user://capture_sandbox.cfg"
	GameState.profile = {"name": "迪丝"}
	GameState.world_state.gold = 640
	GameState.add_materials({Forge.MAT_COMMON: 7, Forge.MAT_RARE: 2, Forge.MAT_RESIDUE: 1})
	for src in ["elite", "enemy", "totem", "enemy"]:
		GameState.inventory.push_back(LootGenerator.generate_loot(src, 1))
	GameState.inventory[0].upgrade_level = 3

func _stage(root: Node, setup: String) -> void:
	GameState.close_dialogue()
	match setup:
		"forge":
			get_tree().call_group("forge_panel", "open")
		"forge_reroll":
			get_tree().call_group("forge_panel", "open")
			var panel := get_tree().get_first_node_in_group("forge_panel")
			panel._reroll(0)
		"anvil":
			var p := root.get_node_or_null("Entities/Player")
			if p:
				p.global_position = Vector2(1040, 760)
