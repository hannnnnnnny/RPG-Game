## 存档系统 —— Godot 用 user:// 路径，自动落到 OS 适当位置
## Windows: %APPDATA%\Godot\app_userdata\TidesOfKhah\
## Linux:   ~/.local/share/godot/app_userdata/TidesOfKhah/
## macOS:   ~/Library/Application Support/Godot/app_userdata/TidesOfKhah/
##
## 用法：注册为 Autoload "SaveSystem"
##
## 自动存档：在 GameState 的关键 signal 上 connect 到 save()
## 手动存档：SaveSystem.save() / SaveSystem.load_save()

extends Node

const SAVE_PATH := "user://tides_of_khah_v1.cfg"

func save() -> void:
	var cfg := ConfigFile.new()
	cfg.set_value("meta", "version", 1)
	cfg.set_value("meta", "saved_at", Time.get_unix_time_from_system())
	cfg.set_value("game", "profile", GameState.profile)
	cfg.set_value("game", "world_state", GameState.world_state)
	cfg.set_value("game", "combat", GameState.combat)
	cfg.set_value("game", "inventory", GameState.inventory)
	cfg.set_value("game", "equipped", GameState.equipped)
	cfg.set_value("game", "log", GameState.log)

	var err := cfg.save(SAVE_PATH)
	if err != OK:
		push_error("Save failed: %s" % err)

func load_save() -> bool:
	var cfg := ConfigFile.new()
	var err := cfg.load(SAVE_PATH)
	if err != OK:
		return false

	GameState.profile = cfg.get_value("game", "profile", {})
	GameState.world_state = cfg.get_value("game", "world_state", Types.make_default_world_state())
	GameState.combat = cfg.get_value("game", "combat", Types.make_default_combat())
	GameState.inventory = cfg.get_value("game", "inventory", [])
	GameState.equipped = cfg.get_value("game", "equipped", {})
	GameState.log = cfg.get_value("game", "log", [])

	# Tell UI to refresh everything
	GameState.emit_signal("profile_changed", GameState.profile)
	GameState.emit_signal("world_state_changed", "*", null)
	GameState.emit_signal("combat_changed", GameState.combat)
	GameState.emit_signal("inventory_changed", GameState.inventory)
	GameState.emit_signal("equipped_changed", GameState.equipped)
	return true

func has_save() -> bool:
	return FileAccess.file_exists(SAVE_PATH)

func delete_save() -> void:
	if has_save():
		DirAccess.remove_absolute(ProjectSettings.globalize_path(SAVE_PATH))
